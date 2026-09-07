use crate::utils::*;
// ssh.rs - SSH connection management with PTY shell support

use serde::{Deserialize, Serialize};
use ssh2::{CheckResult, ErrorCode, HashType, HostKeyType, KnownHostFileKind, KnownHostKeyFormat, Session};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

use crate::utils;

// ─── Types ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshSession {
    pub id: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub connected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorData {
    pub cpu_usage: f64,
    pub memory_total: u64,
    pub memory_used: u64,
    pub memory_percent: f64,
    pub disk_total: u64,
    pub disk_used: u64,
    pub disk_percent: f64,
    pub load_1: f64,
    pub load_5: f64,
    pub load_15: f64,
    pub uptime_seconds: u64,
}

// ─── State ───

/// Exec-based sessions (legacy, one-shot commands)
static SESSIONS: std::sync::LazyLock<Mutex<HashMap<String, Session>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

static SESSION_INFO: std::sync::LazyLock<Mutex<HashMap<String, SshSession>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

/// PTY shell sessions: session_id → PtyShell handle
struct PtyShell {
    input_tx: crossbeam_channel::Sender<String>,
    resize_tx: crossbeam_channel::Sender<(u16, u16)>,
    _reader: std::thread::JoinHandle<()>,
}

static PTY_SHELLS: std::sync::LazyLock<Mutex<HashMap<String, PtyShell>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

/// Authenticated SSH Session objects backing interactive PTY shells.
/// Additional exec/SFTP channels reuse the same SSH transport and credentials.
static PTY_SESSIONS: std::sync::LazyLock<Mutex<HashMap<String, Session>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

// ─── Helpers ───

fn known_host_entry(host: &str, port: u16) -> String {
    if port == 22 {
        host.to_string()
    } else {
        format!("[{}]:{}", host, port)
    }
}

fn host_key_algorithm_and_format(
    key_type: HostKeyType,
) -> Result<(&'static str, KnownHostKeyFormat), String> {
    match key_type {
        HostKeyType::Rsa => Ok(("ssh-rsa", KnownHostKeyFormat::SshRsa)),
        HostKeyType::Dss => Ok(("ssh-dss", KnownHostKeyFormat::SshDss)),
        HostKeyType::Ecdsa256 => Ok((
            "ecdsa-sha2-nistp256",
            KnownHostKeyFormat::Ecdsa256,
        )),
        HostKeyType::Ecdsa384 => Ok((
            "ecdsa-sha2-nistp384",
            KnownHostKeyFormat::Ecdsa384,
        )),
        HostKeyType::Ecdsa521 => Ok((
            "ecdsa-sha2-nistp521",
            KnownHostKeyFormat::Ecdsa521,
        )),
        HostKeyType::Ed25519 => Ok(("ssh-ed25519", KnownHostKeyFormat::Ed25519)),
        HostKeyType::Unknown => Err("不支持的 SSH 主机密钥类型".to_string()),
    }
}

fn format_sha256_fingerprint(hash: &[u8]) -> String {
    let encoded = base64_encode(hash);
    format!("SHA256:{}", encoded.trim_end_matches('='))
}

fn host_key_error(
    code: &str,
    host: &str,
    port: u16,
    algorithm: &str,
    fingerprint: &str,
) -> String {
    let detail = serde_json::json!({
        "host": host,
        "port": port,
        "algorithm": algorithm,
        "fingerprint": fingerprint,
    });
    format!("{}:{}", code, detail)
}

fn is_ssh_eagain(error: &ssh2::Error) -> bool {
    matches!(error.code(), ErrorCode::Session(-37))
}

fn retry_nonblocking_ssh<T>(
    operation_name: &str,
    mut operation: impl FnMut() -> Result<T, ssh2::Error>,
) -> Result<T, String> {
    let started = std::time::Instant::now();
    loop {
        match operation() {
            Ok(value) => return Ok(value),
            Err(error)
                if is_ssh_eagain(&error)
                    && started.elapsed() < std::time::Duration::from_secs(READ_TIMEOUT_SECS) =>
            {
                std::thread::sleep(std::time::Duration::from_millis(PTY_POLL_FAST_MS));
            }
            Err(error) if is_ssh_eagain(&error) => {
                return Err(format!("{}超时", operation_name));
            }
            Err(error) => {
                return Err(format!("{}失败: {}", operation_name, error));
            }
        }
    }
}

fn session_handle(session_id: &str) -> Option<Session> {
    if let Some(session) = lock!(SESSIONS).get(session_id).cloned() {
        return Some(session);
    }
    lock!(PTY_SESSIONS).get(session_id).cloned()
}

fn execute_blocking_session(session: &Session, command: &str) -> Result<String, String> {
    let mut channel = session.channel_session().map_err(|e| e.to_string())?;
    channel.exec(command).map_err(|e| e.to_string())?;
    let mut output = String::new();
    channel
        .read_to_string(&mut output)
        .map_err(|e| format!("读取远程命令输出失败: {}", e))?;
    channel.wait_close().map_err(|e| e.to_string())?;
    let exit = channel.exit_status().unwrap_or(-1);
    Ok(format!("{}\n[exit: {}]", output, exit))
}

fn execute_nonblocking_session(session: &Session, command: &str) -> Result<String, String> {
    let mut channel = retry_nonblocking_ssh("创建远程命令通道", || session.channel_session())?;
    retry_nonblocking_ssh("启动远程命令", || channel.exec(command))?;

    let started = std::time::Instant::now();
    let mut output = Vec::new();
    let mut buffer = [0u8; PTY_BUF_SIZE];

    loop {
        match channel.read(&mut buffer) {
            Ok(0) if channel.eof() => break,
            Ok(0) => {
                if started.elapsed() >= std::time::Duration::from_secs(READ_TIMEOUT_SECS) {
                    return Err("读取远程命令输出超时".to_string());
                }
                std::thread::sleep(std::time::Duration::from_millis(PTY_POLL_FAST_MS));
            }
            Ok(read) => output.extend_from_slice(&buffer[..read]),
            Err(error)
                if error.kind() == std::io::ErrorKind::WouldBlock
                    || error.kind() == std::io::ErrorKind::TimedOut =>
            {
                if channel.eof() {
                    break;
                }
                if started.elapsed() >= std::time::Duration::from_secs(READ_TIMEOUT_SECS) {
                    return Err("读取远程命令输出超时".to_string());
                }
                std::thread::sleep(std::time::Duration::from_millis(PTY_POLL_FAST_MS));
            }
            Err(error) => return Err(format!("读取远程命令输出失败: {}", error)),
        }
    }

    retry_nonblocking_ssh("关闭远程命令通道", || channel.wait_close())?;
    let exit = retry_nonblocking_ssh("读取远程命令退出状态", || channel.exit_status())
        .unwrap_or(-1);
    let output = String::from_utf8_lossy(&output).into_owned();
    Ok(format!("{}\n[exit: {}]", output, exit))
}

/// Create an SSH session from a TCP stream and verify the remote host key
/// against ~/.ssh/known_hosts.
fn create_session(
    tcp: TcpStream,
    host: &str,
    port: u16,
    trust_new_host_key: bool,
) -> Result<Session, String> {
    tcp.set_nonblocking(false)
        .map_err(|e| format!("设置阻塞模式失败: {}", e))?;

    let mut session = Session::new().map_err(|e| e.to_string())?;
    session.set_tcp_stream(tcp);
    session
        .handshake()
        .map_err(|e| format!("握手失败: [{}] {}", e.code(), e.message()))?;

    let (key, key_type) = session
        .host_key()
        .ok_or_else(|| "无法获取服务器主机密钥".to_string())?;
    let (algorithm, key_format) = host_key_algorithm_and_format(key_type)?;
    let fingerprint = session
        .host_key_hash(HashType::Sha256)
        .map(format_sha256_fingerprint)
        .ok_or_else(|| "无法计算服务器主机密钥指纹".to_string())?;

    let known_hosts_path = dirs::home_dir()
        .ok_or_else(|| "无法确定用户主目录".to_string())?
        .join(".ssh")
        .join("known_hosts");

    if let Some(parent) = known_hosts_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("创建 ~/.ssh 目录失败: {}", e))?;
    }

    let mut known_hosts = session
        .known_hosts()
        .map_err(|e| format!("无法初始化 known_hosts 检查: {}", e))?;

    if known_hosts_path.exists() {
        known_hosts
            .read_file(&known_hosts_path, KnownHostFileKind::OpenSSH)
            .map_err(|e| format!("读取 known_hosts 失败: {}", e))?;
    }

    match known_hosts.check_port(host, port, key) {
        CheckResult::Match => {}
        CheckResult::NotFound if !trust_new_host_key => {
            return Err(host_key_error(
                "HOST_KEY_UNKNOWN",
                host,
                port,
                algorithm,
                &fingerprint,
            ));
        }
        CheckResult::NotFound => {
            let entry = known_host_entry(host, port);
            known_hosts
                .add(&entry, key, host, key_format)
                .map_err(|e| format!("添加主机密钥到 known_hosts 失败: {}", e))?;
            known_hosts
                .write_file(&known_hosts_path, KnownHostFileKind::OpenSSH)
                .map_err(|e| format!("写入 known_hosts 失败: {}", e))?;
        }
        CheckResult::Mismatch => {
            return Err(host_key_error(
                "HOST_KEY_MISMATCH",
                host,
                port,
                algorithm,
                &fingerprint,
            ));
        }
        CheckResult::Failure => {
            return Err("主机密钥验证失败".to_string());
        }
    }

    Ok(session)
}

// ─── Exec-based API (legacy, one-shot commands) ───

pub async fn connect(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
    trust_new_host_key: bool,
) -> Result<String, String> {
    let addr: std::net::SocketAddr = format!("{}:{}", host, port)
        .parse()
        .map_err(|e| format!("地址无效: {}", e))?;
    let tcp = TcpStream::connect_timeout(
        &addr,
        std::time::Duration::from_secs(CONNECT_TIMEOUT_SECS),
    )
    .map_err(|e| format!("连接失败: {}", e))?;

    let session = create_session(tcp, host, port, trust_new_host_key)?;

    let mut auth_err = String::new();
    for attempt in 1..=5 {
        match session.userauth_password(username, password) {
            Ok(()) => break,
            Err(e) => {
                auth_err = format!("认证失败: {}", e);
                let msg = e.message().to_lowercase();
                if msg.contains("would block")
                    || msg.contains("again")
                    || msg.contains("busy")
                    || msg.contains("too many")
                {
                    std::thread::sleep(std::time::Duration::from_millis(200 * attempt));
                    continue;
                }
                return Err(auth_err);
            }
        }
    }
    if !session.authenticated() {
        let detail = if auth_err.is_empty() {
            "用户名或密码错误".to_string()
        } else {
            auth_err
        };
        return Err(detail);
    }

    let id = format!("ssh-exec-{}-{}", host.replace('.', "_"), utils::unix_now());
    lock!(SESSION_INFO).insert(
        id.clone(),
        SshSession {
            id: id.clone(),
            host: host.into(),
            port,
            username: username.into(),
            connected: true,
        },
    );
    lock!(SESSIONS).insert(id.clone(), session);
    Ok(id)
}

pub async fn connect_with_key(
    host: &str,
    port: u16,
    username: &str,
    key_path: &str,
    passphrase: &str,
    trust_new_host_key: bool,
) -> Result<String, String> {
    let addr: std::net::SocketAddr = format!("{}:{}", host, port)
        .parse()
        .map_err(|e| format!("地址无效: {}", e))?;
    let tcp = TcpStream::connect_timeout(
        &addr,
        std::time::Duration::from_secs(CONNECT_TIMEOUT_SECS),
    )
    .map_err(|e| format!("连接失败: {}", e))?;
    let session = create_session(tcp, host, port, trust_new_host_key)?;
    if passphrase.is_empty() {
        session
            .userauth_pubkey_file(
                username,
                None,
                std::path::Path::new(key_path),
                None,
            )
            .map_err(|e| format!("密钥认证失败: {}", e))?;
    } else {
        session
            .userauth_pubkey_file(
                username,
                None,
                std::path::Path::new(key_path),
                Some(passphrase),
            )
            .map_err(|e| format!("密钥认证失败: {}", e))?;
    }
    if !session.authenticated() {
        return Err("密钥认证失败".into());
    }
    let id = format!("ssh-exec-{}-{}", host.replace('.', "_"), utils::unix_now());
    lock!(SESSION_INFO).insert(
        id.clone(),
        SshSession {
            id: id.clone(),
            host: host.into(),
            port,
            username: username.into(),
            connected: true,
        },
    );
    lock!(SESSIONS).insert(id.clone(), session);
    Ok(id)
}

pub async fn execute(session_id: &str, command: &str) -> Result<String, String> {
    let session = session_handle(session_id).ok_or_else(|| "会话不存在".to_string())?;
    if session.is_blocking() {
        execute_blocking_session(&session, command)
    } else {
        execute_nonblocking_session(&session, command)
    }
}

pub fn open_sftp(session_id: &str) -> Result<ssh2::Sftp, String> {
    let session = session_handle(session_id).ok_or_else(|| "会话不存在".to_string())?;
    if session.is_blocking() {
        session.sftp().map_err(|e| format!("初始化 SFTP 失败: {}", e))
    } else {
        retry_nonblocking_ssh("初始化 SFTP", || session.sftp())
    }
}

pub fn disconnect(session_id: &str) -> Result<(), String> {
    lock!(SESSIONS).remove(session_id);
    lock!(SESSION_INFO).remove(session_id);
    Ok(())
}

pub fn list_sessions() -> Vec<SshSession> {
    lock!(SESSION_INFO).values().cloned().collect()
}

// ─── PTY Shell API (real interactive shell) ───

pub fn start_shell(
    app: AppHandle,
    host: &str,
    port: u16,
    username: &str,
    password: &str,
    key_path: Option<&str>,
    passphrase: Option<&str>,
    trust_new_host_key: bool,
    cols: u16,
    rows: u16,
) -> Result<String, String> {
    let addr: std::net::SocketAddr = format!("{}:{}", host, port)
        .parse()
        .map_err(|e| format!("地址无效: {}", e))?;
    let tcp = TcpStream::connect_timeout(
        &addr,
        std::time::Duration::from_secs(CONNECT_TIMEOUT_SECS),
    )
    .map_err(|e| format!("连接失败: {}", e))?;

    let session = create_session(tcp, host, port, trust_new_host_key)?;

    let mut auth_err = String::new();
    if let Some(kp) = key_path {
        let pp = passphrase.unwrap_or("");
        for attempt in 1..=5 {
            let result = if pp.is_empty() {
                session.userauth_pubkey_file(
                    username,
                    None,
                    std::path::Path::new(kp),
                    None,
                )
            } else {
                session.userauth_pubkey_file(
                    username,
                    None,
                    std::path::Path::new(kp),
                    Some(pp),
                )
            };
            match result {
                Ok(()) => break,
                Err(e) => {
                    auth_err = format!("密钥认证失败: {}", e);
                    let msg = e.message().to_lowercase();
                    if msg.contains("would block")
                        || msg.contains("again")
                        || msg.contains("busy")
                        || msg.contains("too many")
                    {
                        std::thread::sleep(std::time::Duration::from_millis(200 * attempt));
                        continue;
                    }
                    return Err(auth_err);
                }
            }
        }
    } else {
        for attempt in 1..=5 {
            match session.userauth_password(username, password) {
                Ok(()) => break,
                Err(e) => {
                    auth_err = format!("认证失败: {}", e);
                    let msg = e.message().to_lowercase();
                    if msg.contains("would block")
                        || msg.contains("again")
                        || msg.contains("busy")
                        || msg.contains("too many")
                    {
                        std::thread::sleep(std::time::Duration::from_millis(200 * attempt));
                        continue;
                    }
                    return Err(auth_err);
                }
            }
        }
    }

    if !session.authenticated() {
        let detail = if auth_err.is_empty() {
            "SSH 认证失败".to_string()
        } else {
            auth_err
        };
        return Err(detail);
    }

    let mut channel = session
        .channel_session()
        .map_err(|e| format!("创建通道失败: [{}] {}", e.code(), e.message()))?;
    channel
        .request_pty_size(cols as u32, rows as u32, None, None)
        .map_err(|e| format!("PTY 失败: {}", e))?;
    channel
        .shell()
        .map_err(|e| format!("启动 shell 失败: {}", e))?;

    session.set_blocking(false);

    let session_id = format!("ssh-shell-{}-{}", host.replace('.', "_"), utils::unix_now());
    let sid = session_id.clone();

    let (input_tx, input_rx) = crossbeam_channel::unbounded::<String>();
    let (resize_tx, resize_rx) = crossbeam_channel::unbounded::<(u16, u16)>();

    let app_handle = app.clone();
    let reader_sid = session_id.clone();
    let reader = std::thread::spawn(move || {
        let mut buf = [0u8; PTY_BUF_SIZE];
        let mut consecutive_empty = 0u32;

        'shell: loop {
            match channel.read(&mut buf) {
                Ok(0) => {
                    let _ = app_handle.emit(
                        &format!("ssh-output:{}", reader_sid),
                        "\r\n\x1b[1;33m[Shell 已退出]\x1b[0m\r\n",
                    );
                    break;
                }
                Ok(n) => {
                    consecutive_empty = 0;
                    let output = String::from_utf8_lossy(&buf[..n]).into_owned();
                    let _ = app_handle.emit(&format!("ssh-output:{}", reader_sid), output);
                }
                Err(ref e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut =>
                {
                    if channel.eof() {
                        let _ = app_handle.emit(
                            &format!("ssh-output:{}", reader_sid),
                            "\r\n\x1b[1;33m[Shell 已退出]\x1b[0m\r\n",
                        );
                        break;
                    }
                    consecutive_empty += 1;
                }
                Err(_) => {
                    let _ = app_handle.emit(
                        &format!("ssh-output:{}", reader_sid),
                        "\r\n\x1b[1;31m[连接断开]\x1b[0m\r\n",
                    );
                    break;
                }
            }

            while let Ok(input) = input_rx.try_recv() {
                if input == "\x04" {
                    channel.send_eof().ok();
                    let _ = app_handle.emit(
                        &format!("ssh-output:{}", reader_sid),
                        "\r\n\x1b[1;33m[Shell 已退出]\x1b[0m\r\n",
                    );
                    break 'shell;
                }

                let data = input.as_bytes();
                let mut written = 0;
                for _ in 0..10 {
                    match channel.write(&data[written..]) {
                        Ok(n) => {
                            written += n;
                            if written >= data.len() {
                                break;
                            }
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            std::thread::sleep(std::time::Duration::from_millis(PTY_POLL_FAST_MS));
                        }
                        Err(_) => break,
                    }
                }
            }

            while let Ok((cols, rows)) = resize_rx.try_recv() {
                channel
                    .request_pty_size(cols as u32, rows as u32, None, None)
                    .ok();
            }

            if consecutive_empty > PTY_IDLE_THRESHOLD {
                std::thread::sleep(std::time::Duration::from_millis(PTY_POLL_SLOW_MS));
            } else {
                std::thread::sleep(std::time::Duration::from_millis(PTY_POLL_FAST_MS));
            }
        }

        drop(channel);
        lock!(PTY_SESSIONS).remove(&reader_sid);
        lock!(PTY_SHELLS).remove(&reader_sid);
    });

    lock!(PTY_SHELLS).insert(
        session_id.clone(),
        PtyShell {
            input_tx,
            resize_tx,
            _reader: reader,
        },
    );
    lock!(PTY_SESSIONS).insert(session_id.clone(), session);

    Ok(sid)
}

pub fn shell_input(session_id: &str, data: &str) -> Result<(), String> {
    let shells = lock!(PTY_SHELLS);
    let shell = shells.get(session_id).ok_or("Shell 会话不存在")?;
    shell
        .input_tx
        .send(data.to_string())
        .map_err(|e| format!("发送输入失败: {}", e))?;
    Ok(())
}

pub fn shell_resize(session_id: &str, cols: u16, rows: u16) -> Result<(), String> {
    let shells = lock!(PTY_SHELLS);
    let shell = shells.get(session_id).ok_or("Shell 会话不存在")?;
    shell
        .resize_tx
        .send((cols, rows))
        .map_err(|e| format!("发送 resize 失败: {}", e))?;
    Ok(())
}

pub fn close_shell(session_id: &str) -> Result<(), String> {
    let shell = {
        let mut shells = lock!(PTY_SHELLS);
        shells.remove(session_id)
    };

    if let Some(shell) = shell {
        let _ = shell.input_tx.send("\x04".to_string());
    }
    lock!(PTY_SESSIONS).remove(session_id);
    Ok(())
}

pub fn list_shells() -> Vec<String> {
    lock!(PTY_SHELLS).keys().cloned().collect()
}

// ─── Remote Command Execution (for monitoring) ───

pub async fn get_monitor_data(session_id: &str) -> Result<MonitorData, String> {
    let output = execute(
        session_id,
        "cat /proc/stat /proc/meminfo /proc/loadavg /proc/uptime && df -B1 / | tail -1",
    )
    .await?;
    parse_monitor_data(&output)
}

fn parse_monitor_data(output: &str) -> Result<MonitorData, String> {
    let mut cpu_total = 0u64;
    let mut cpu_idle = 0u64;
    let mut mem_total = 0u64;
    let mut mem_available = 0u64;
    let mut load_1 = 0.0f64;
    let mut load_5 = 0.0f64;
    let mut load_15 = 0.0f64;
    let mut uptime_seconds = 0u64;
    let mut disk_total = 0u64;
    let mut disk_used = 0u64;

    for line in output.lines() {
        let line = line.trim();

        if line.starts_with("cpu ") && cpu_total == 0 {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 6 {
                let user: u64 = parts[1].parse().unwrap_or(0);
                let nice: u64 = parts[2].parse().unwrap_or(0);
                let system: u64 = parts[3].parse().unwrap_or(0);
                cpu_idle = parts[4].parse().unwrap_or(0);
                let iowait: u64 = parts[5].parse().unwrap_or(0);
                cpu_total = user + nice + system + cpu_idle + iowait;
            }
        }

        if line.starts_with("MemTotal:") {
            mem_total = line
                .split_whitespace()
                .nth(1)
                .and_then(|v| v.parse().ok())
                .unwrap_or(0)
                * 1024;
        }
        if line.starts_with("MemAvailable:") {
            mem_available = line
                .split_whitespace()
                .nth(1)
                .and_then(|v| v.parse().ok())
                .unwrap_or(0)
                * 1024;
        }

        if line.contains('.') && line.split_whitespace().count() >= 3 {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let (Ok(l1), Ok(l5), Ok(l15)) = (
                parts[0].parse::<f64>(),
                parts[1].parse::<f64>(),
                parts[2].parse::<f64>(),
            ) {
                if load_1 == 0.0 && l1 > 0.0 {
                    load_1 = l1;
                    load_5 = l5;
                    load_15 = l15;
                }
            }
        }

        if line.contains('.') && line.split_whitespace().count() == 2 {
            if let Some(first) = line.split_whitespace().next() {
                if let Ok(uptime) = first.parse::<f64>() {
                    if uptime > 1.0 && uptime_seconds == 0 {
                        uptime_seconds = uptime as u64;
                    }
                }
            }
        }

        if line.starts_with('/') {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                disk_total = parts[1].parse().unwrap_or(0);
                disk_used = parts[2].parse().unwrap_or(0);
            }
        }
    }

    let cpu_usage = if cpu_total > 0 {
        ((cpu_total - cpu_idle) as f64 / cpu_total as f64) * 100.0
    } else {
        0.0
    };

    let memory_used = mem_total.saturating_sub(mem_available);
    let memory_percent = if mem_total > 0 {
        (memory_used as f64 / mem_total as f64) * 100.0
    } else {
        0.0
    };

    let disk_percent = if disk_total > 0 {
        (disk_used as f64 / disk_total as f64) * 100.0
    } else {
        0.0
    };

    Ok(MonitorData {
        cpu_usage,
        memory_total: mem_total,
        memory_used,
        memory_percent,
        disk_total,
        disk_used,
        disk_percent,
        load_1,
        load_5,
        load_15,
        uptime_seconds,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_host_entry_default_port() {
        assert_eq!(known_host_entry("example.com", 22), "example.com");
    }

    #[test]
    fn test_known_host_entry_custom_port() {
        assert_eq!(known_host_entry("example.com", 2222), "[example.com]:2222");
    }

    #[test]
    fn test_sha256_fingerprint_format() {
        assert_eq!(format_sha256_fingerprint(b"abc"), "SHA256:YWJj");
    }

    #[test]
    fn test_host_key_error_is_machine_readable() {
        let err = host_key_error(
            "HOST_KEY_UNKNOWN",
            "example.com",
            2222,
            "ssh-ed25519",
            "SHA256:YWJj",
        );
        let payload = err
            .strip_prefix("HOST_KEY_UNKNOWN:")
            .expect("error prefix");
        let json: serde_json::Value = serde_json::from_str(payload).expect("valid JSON payload");
        assert_eq!(json["host"], "example.com");
        assert_eq!(json["port"], 2222);
        assert_eq!(json["algorithm"], "ssh-ed25519");
        assert_eq!(json["fingerprint"], "SHA256:YWJj");
    }

    #[test]
    fn test_eagain_detection() {
        let eagain = ssh2::Error::new(ErrorCode::Session(-37), "would block");
        let other = ssh2::Error::new(ErrorCode::Session(-1), "other");
        assert!(is_ssh_eagain(&eagain));
        assert!(!is_ssh_eagain(&other));
    }

    #[test]
    fn test_unix_now_returns_positive() {
        let t = utils::unix_now();
        assert!(t > 0, "utils::unix_now() should return a value > 0");
        assert!(t > 1_577_836_800, "utils::unix_now() should be after 2020");
    }

    #[test]
    fn test_ssh_session_serialize_deserialize() {
        let session = SshSession {
            id: "ssh_192_168_1_1_12345".into(),
            host: "192.168.1.1".into(),
            port: 22,
            username: "root".into(),
            connected: true,
        };
        let json = serde_json::to_string(&session).expect("serialize SshSession");
        assert!(json.contains("192.168.1.1"));
        assert!(json.contains("root"));

        let back: SshSession = serde_json::from_str(&json).expect("deserialize SshSession");
        assert_eq!(back.id, "ssh_192_168_1_1_12345");
        assert_eq!(back.host, "192.168.1.1");
        assert_eq!(back.port, 22);
        assert_eq!(back.username, "root");
        assert!(back.connected);
    }

    #[test]
    fn test_monitor_data_deserialize() {
        let json = r#"{
            "cpuUsage": 25.5,
            "memoryTotal": 8589934592,
            "memoryUsed": 4294967296,
            "memoryPercent": 50.0,
            "diskTotal": 107374182400,
            "diskUsed": 53687091200,
            "diskPercent": 50.0,
            "load1": 1.5,
            "load5": 1.2,
            "load15": 0.8,
            "uptimeSeconds": 86400
        }"#;
        let data: MonitorData = serde_json::from_str(json).expect("deserialize MonitorData");
        assert!((data.cpu_usage - 25.5).abs() < 0.01);
        assert_eq!(data.memory_total, 8_589_934_592);
        assert_eq!(data.uptime_seconds, 86400);
    }

    #[test]
    fn test_parse_monitor_data() {
        let output = r#"cpu  1000 100 500 3000 200 0 0 0 0 0
cpu0 500 50 250 1500 100 0 0 0 0 0
MemTotal:       16384000 kB
MemAvailable:    8192000 kB
0.50 0.30 0.20 1/500 12345
123456.78 234567.89
/dev/sda1 107374182400 53687091200 53687091200 50% /"#;
        let data = parse_monitor_data(output).expect("parse_monitor_data");
        assert!(data.cpu_usage > 0.0);
        assert!(data.cpu_usage < 100.0);
        assert_eq!(data.memory_total, 16_384_000 * 1024);
        assert_eq!(data.memory_used, (16_384_000 - 8_192_000) * 1024);
        assert!((data.load_1 - 0.50).abs() < 0.01);
        assert!((data.load_5 - 0.30).abs() < 0.01);
        assert!((data.load_15 - 0.20).abs() < 0.01);
        assert!(data.uptime_seconds > 0);
        assert_eq!(data.disk_total, 107_374_182_400);
    }

    #[test]
    fn test_parse_monitor_data_empty() {
        let data = parse_monitor_data("").expect("empty input should still return valid data");
        assert_eq!(data.cpu_usage, 0.0);
        assert_eq!(data.memory_total, 0);
        assert_eq!(data.load_1, 0.0);
    }
}
