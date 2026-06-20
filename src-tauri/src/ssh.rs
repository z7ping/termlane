use crate::utils::*;
// ssh.rs - SSH connection management with PTY shell support

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Mutex;
use ssh2::Session;
use tauri::{AppHandle, Emitter};
use ssh2::{CheckResult, KnownHostFileKind};


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

/// Session objects owned by PTY threads (for disconnect/cleanup)
static PTY_SESSIONS: std::sync::LazyLock<Mutex<HashMap<String, Session>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

// ─── Helpers ───

/// Create an SSH session from a TCP stream, then verify the remote host key
/// against ~/.ssh/known_hosts.  If the host is unknown the key is trusted on
/// first use (TOFU) and persisted; if the key has changed the connection is
/// rejected with a clear error.
fn create_session(tcp: TcpStream, host: &str) -> Result<Session, String> {
    // Step 0: ensure blocking mode
    eprintln!("[DEBUG ssh] Step 0: setting blocking mode...");
    tcp.set_nonblocking(false)
        .map_err(|e| {
            eprintln!("[DEBUG ssh] FAIL set_nonblocking: {}", e);
            format!("设置阻塞模式失败: {}", e)
        })?;
    eprintln!("[DEBUG ssh] Step 0 OK: blocking mode set");

    // Step 1: create session
    eprintln!("[DEBUG ssh] Step 1: Session::new()...");
    let mut session = Session::new().map_err(|e| {
        eprintln!("[DEBUG ssh] FAIL Session::new: {}", e);
        e.to_string()
    })?;
    eprintln!("[DEBUG ssh] Step 1 OK");

    // Step 2: set TCP stream
    eprintln!("[DEBUG ssh] Step 2: set_tcp_stream...");
    session.set_tcp_stream(tcp);
    eprintln!("[DEBUG ssh] Step 2 OK");

    // Step 3: handshake (the tricky part on Windows)
    eprintln!("[DEBUG ssh] Step 3: handshake...");
    session.handshake().map_err(|e| {
        let code = e.code();
        let msg = e.message();
        eprintln!(
            "[DEBUG ssh] FAIL handshake: code={} msg={}",
            code, msg
        );
        format!("握手失败: [{}] {}", code, msg)
    })?;
    eprintln!("[DEBUG ssh] Step 3 OK: handshake succeeded");

    // ── Host-key verification (TOFU style) ──
    let (key, key_type) = session.host_key()
        .ok_or_else(|| "无法获取服务器主机密钥".to_string())?;

    let known_hosts_path = dirs::home_dir()
        .ok_or_else(|| "无法确定用户主目录".to_string())?
        .join(".ssh")
        .join("known_hosts");

    // Ensure ~/.ssh directory exists
    if let Some(parent) = known_hosts_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let mut known_hosts = session.known_hosts()
        .map_err(|e| format!("无法初始化 known_hosts 检查: {}", e))?;

    if known_hosts_path.exists() {
        known_hosts.read_file(&known_hosts_path, KnownHostFileKind::OpenSSH)
            .map_err(|e| format!("读取 known_hosts 失败: {}", e))?;
    }

    let key_format = match key_type {
        ssh2::HostKeyType::Rsa      => ssh2::KnownHostKeyFormat::SshRsa,
        ssh2::HostKeyType::Dss      => ssh2::KnownHostKeyFormat::SshDss,
        ssh2::HostKeyType::Ecdsa256 => ssh2::KnownHostKeyFormat::Ecdsa256,
        ssh2::HostKeyType::Ecdsa384 => ssh2::KnownHostKeyFormat::Ecdsa384,
        ssh2::HostKeyType::Ecdsa521 => ssh2::KnownHostKeyFormat::Ecdsa521,
        ssh2::HostKeyType::Ed25519  => ssh2::KnownHostKeyFormat::Ed25519,
        _                          => ssh2::KnownHostKeyFormat::SshRsa,
    };
    match known_hosts.check(host, key) {
        CheckResult::Match => {
            // Key is trusted — proceed
        }
        CheckResult::NotFound => {
            // TOFU — trust on first use
            known_hosts.add(host, key, host, key_format)
                .map_err(|e| format!("添加主机密钥到 known_hosts 失败: {}", e))?;
            known_hosts.write_file(&known_hosts_path, KnownHostFileKind::OpenSSH)
                .map_err(|e| format!("写入 known_hosts 失败: {}", e))?;
        }
        CheckResult::Mismatch => {
            return Err(format!(
                "⚠️ 主机密钥不匹配！服务器 {} 的密钥已变更，可能是中间人攻击或服务器重装了系统。\
                  如需继续连接请手动编辑 ~/.ssh/known_hosts 删除该主机的条目。", host
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
) -> Result<String, String> {
    let addr: std::net::SocketAddr = format!("{}:{}", host, port)
        .parse()
        .map_err(|e| format!("地址无效: {}", e))?;
    let tcp = TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(CONNECT_TIMEOUT_SECS))
        .map_err(|e| format!("连接失败: {}", e))?;

    let session = create_session(tcp, host)?;

    // Session is blocking by default; libssh2 on Windows has a quirk where
    // EAGAIN (-37) can surface even in blocking mode when the socket buffer
    // hasn't received the server's auth response yet.  Retry a few times.
    eprintln!("[DEBUG ssh] Step 4: userauth_password...");
    let mut auth_err = String::new();
    for attempt in 1..=5 {
        match session.userauth_password(username, password) {
            Ok(()) => {
                eprintln!("[DEBUG ssh] Step 4 OK (attempt {})", attempt);
                break;
            }
            Err(e) => {
                auth_err = format!("认证失败: {}", e);
                let msg = e.message().to_lowercase();
                eprintln!(
                    "[DEBUG ssh] Step 4 attempt {} FAIL: code={} msg={}",
                    attempt, e.code(), e.message()
                );
                // EAGAIN = retry; server_busy/too many auth = retry
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
        // On first failure, auth_err might still be empty (unlikely, but guard).
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
) -> Result<String, String> {
    let addr: std::net::SocketAddr = format!("{}:{}", host, port)
        .parse()
        .map_err(|e| format!("地址无效: {}", e))?;
    let tcp = TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(CONNECT_TIMEOUT_SECS))
        .map_err(|e| format!("连接失败: {}", e))?;
    let session = create_session(tcp, host)?;
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
    let sessions = lock!(SESSIONS);
    let session = sessions.get(session_id).ok_or("会话不存在")?;
    let mut ch = session.channel_session().map_err(|e| e.to_string())?;
    ch.request_pty("xterm-256color", None, None).ok();
    ch.exec(command).map_err(|e| e.to_string())?;
    let mut output = String::new();
    ch.read_to_string(&mut output).ok();
    ch.wait_close().ok();
    let exit = ch.exit_status().unwrap_or(-1);
    Ok(format!("{}\n[exit: {}]", output, exit))
}

pub fn disconnect(session_id: &str) -> Result<(), String> {
    lock!(SESSIONS).remove(session_id);
    lock!(SESSION_INFO).remove(session_id);
    Ok(())
}

pub fn list_sessions() -> Vec<SshSession> {
    lock!(SESSION_INFO)
        .values()
        .cloned()
        .collect()
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
    cols: u16,
    rows: u16,
) -> Result<String, String> {
    // 1. Connect (with 10s timeout)
    let addr: std::net::SocketAddr = format!("{}:{}", host, port)
        .parse()
        .map_err(|e| format!("地址无效: {}", e))?;
    let tcp = TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(CONNECT_TIMEOUT_SECS))
        .map_err(|e| format!("连接失败: {}", e))?;

    let session = create_session(tcp, host)?;

    // 2. Authenticate (with EAGAIN retry for Windows Winsock quirk)
    eprintln!("[DEBUG ssh] Step PTY auth: authenticating...");
    let mut auth_err = String::new();
    if let Some(kp) = key_path {
        let pp = passphrase.unwrap_or("");
        for attempt in 1..=5 {
            let r = if pp.is_empty() {
                session.userauth_pubkey_file(username, None, std::path::Path::new(kp), None)
            } else {
                session.userauth_pubkey_file(username, None, std::path::Path::new(kp), Some(pp))
            };
            match r {
                Ok(()) => { eprintln!("[DEBUG ssh] Step PTY auth OK (key, attempt {})", attempt); break; }
                Err(e) => {
                    auth_err = format!("密钥认证失败: {}", e);
                    let msg = e.message().to_lowercase();
                    eprintln!("[DEBUG ssh] Step PTY auth key attempt {} FAIL: code={} msg={}", attempt, e.code(), e.message());
                    if msg.contains("would block") || msg.contains("again") || msg.contains("busy") || msg.contains("too many") {
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
                Ok(()) => { eprintln!("[DEBUG ssh] Step PTY auth OK (password, attempt {})", attempt); break; }
                Err(e) => {
                    auth_err = format!("认证失败: {}", e);
                    let msg = e.message().to_lowercase();
                    eprintln!("[DEBUG ssh] Step PTY auth password attempt {} FAIL: code={} msg={}", attempt, e.code(), e.message());
                    if msg.contains("would block") || msg.contains("again") || msg.contains("busy") || msg.contains("too many") {
                        std::thread::sleep(std::time::Duration::from_millis(200 * attempt));
                        continue;
                    }
                    return Err(auth_err);
                }
            }
        }
    }

    eprintln!("[DEBUG ssh] Step PTY auth OK (all attempts passed, authenticated={})",
             session.authenticated());

    // IMPORTANT: Keep session BLOCKING when creating the channel.
    // set_blocking(false) on Windows makes channel_session() return EAGAIN (-37)
    // immediately.  We'll switch the channel (not the session) to non-blocking
    // once everything is set up.

    // 3. Request shell channel (keep session blocking here)
    eprintln!("[DEBUG ssh] Step PTY: channel_session...");
    let mut channel = session.channel_session()
        .map_err(|e| format!("创建通道失败: [{}] {}", e.code(), e.message()))?;
    eprintln!("[DEBUG ssh] Step PTY: channel_session OK");
    channel
        .request_pty_size(cols as u32, rows as u32, None, None)
        .map_err(|e| format!("PTY 失败: {}", e))?;
    channel
        .shell()
        .map_err(|e| format!("启动 shell 失败: {}", e))?;
    eprintln!("[DEBUG ssh] Step PTY: shell started OK");

    // Now that channel+shell are created, switch session to non-blocking
    // for the reader thread (so channel.read() returns WouldBlock instead
    // of blocking forever while we also need to check input_rx.try_recv()).
    eprintln!("[DEBUG ssh] Step PTY: setting session to non-blocking...");
    session.set_blocking(false);
    eprintln!("[DEBUG ssh] Step PTY: session now non-blocking");

    let session_id = format!("ssh-shell-{}-{}", host.replace('.', "_"), utils::unix_now());
    let sid = session_id.clone();

    // 4. Create crossbeam channels for I/O + resize
    let (input_tx, input_rx) = crossbeam_channel::unbounded::<String>();
    let (resize_tx, resize_rx) = crossbeam_channel::unbounded::<(u16, u16)>();

    // 5. Spawn reader thread (owns the channel, does all I/O)
    let app_handle = app.clone();
    let reader_sid = session_id.clone();
    let reader = std::thread::spawn(move || {
        let mut buf = [0u8; PTY_BUF_SIZE];
        let mut consecutive_empty = 0u32;

        loop {
            // ── Read output from SSH ──
            match channel.read(&mut buf) {
                Ok(0) => {
                    // EOF — shell exited
                    let _ = app_handle.emit(
                        &format!("ssh-output:{}", reader_sid),
                        "\r\n\x1b[1;33m[Shell 已退出]\x1b[0m\r\n",
                    );
                    break;
                }
                Ok(n) => {
                    consecutive_empty = 0;
                    let output = String::from_utf8_lossy(&buf[..n]).into_owned();
                    let _ = app_handle
                        .emit(&format!("ssh-output:{}", reader_sid), output);
                }
                Err(ref e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut =>
                {
                    // No data available, check for EOF
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
                    // Connection error
                    let _ = app_handle.emit(
                        &format!("ssh-output:{}", reader_sid),
                        "\r\n\x1b[1;31m[连接断开]\x1b[0m\r\n",
                    );
                    break;
                }
            }

            // ── Write user input to SSH ──
            while let Ok(input) = input_rx.try_recv() {
                eprintln!("[DEBUG reader] input_rx got: len={}", input.len());
                if input == "\x04" {
                    // Ctrl+D → send EOF
                    channel.send_eof().ok();
                } else {
                    // In non-blocking mode, write may also return WouldBlock.
                    // Retry a few times before giving up.
                    let data = input.as_bytes();
                    let mut written = 0;
                    for retry in 0..10 {
                        match channel.write(&data[written..]) {
                            Ok(n) => {
                                written += n;
                                eprintln!("[DEBUG reader] channel.write OK: {} bytes (total {}/{})", n, written, data.len());
                                if written >= data.len() { break; }
                            }
                            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                eprintln!("[DEBUG reader] channel.write WouldBlock retry {}", retry);
                                std::thread::sleep(std::time::Duration::from_millis(5));
                                continue;
                            }
                            Err(e) => {
                                eprintln!("[DEBUG reader] channel.write FAIL: {:?}", e);
                                break;
                            }
                        }
                    }
                }
            }

            // ── Handle resize requests ──
            while let Ok((cols, rows)) = resize_rx.try_recv() {
                channel.request_pty_size(cols as u32, rows as u32, None, None).ok();
            }

            // ── Yield CPU ──
            if consecutive_empty > PTY_IDLE_THRESHOLD {
                // After 10 empty reads, sleep longer to avoid busy-spinning
                std::thread::sleep(std::time::Duration::from_millis(PTY_POLL_SLOW_MS));
            } else {
                // Tight poll for low latency (5ms keeps latency imperceptible for terminal I/O)
                std::thread::sleep(std::time::Duration::from_millis(PTY_POLL_FAST_MS));
            }
        }

        // Channel is dropped here, session cleanup happens separately
        drop(channel);
    });

    // 6. Store state (BEFORE spawning reader thread so shell_input can find it)
    lock!(PTY_SHELLS).insert(
        session_id.clone(),
        PtyShell {
            input_tx,
            resize_tx,
            _reader: reader,
        },
    );
    lock!(PTY_SESSIONS)
        .insert(session_id.clone(), session);

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
    lock!(PTY_SHELLS).remove(session_id);
    // Dropping the session closes the TCP connection → reader thread exits
    lock!(PTY_SESSIONS).remove(session_id);
    Ok(())
}

pub fn list_shells() -> Vec<String> {
    lock!(PTY_SHELLS).keys().cloned().collect()
}

// ─── Jump Host (Proxy Jump) ───
// LIMITATION: ssh2-rs (libssh2 bindings) does NOT support creating a Session
// over a Channel tunnel. ssh2::Session::new() requires a TcpStream, not a
// Channel stream, so true SSH ProxyJump (tunneling through an intermediate host)
// is not possible with this API.
//
// This function:
// 1. Verifies the jump host is reachable and can establish a tunnel to the target
//    (pre-validation step - useful for catching misconfigurations early)
// 2. Connects DIRECTLY to the target (assumes network reachability from the
//    client machine, NOT through the jump host tunnel)
//
// For TRUE ProxyJump support, use one of:
//   - OpenSSH ProxyJump (-J flag) via a local PTY shell session
//   - A library that supports session-over-channel (e.g., ssh2 Tokio bindings)
//   - libssh2 native ProxyJump (when available)
//
// This function is effectively: "verify jump host can reach target, then connect
// directly." The jump host connectivity check is kept as a useful pre-validation.

pub async fn connect_jump(
    jump_host: &str,
    jump_port: u16,
    jump_user: &str,
    jump_pass: &str,
    target_host: &str,
    target_port: u16,
    target_user: &str,
    target_pass: &str,
) -> Result<String, String> {
    // Step 1: Verify jump host is reachable and can tunnel to target
    // This is pre-validation only - we don't use the tunnel for the actual connection
    let jump_addr: std::net::SocketAddr = format!("{}:{}", jump_host, jump_port)
        .parse()
        .map_err(|e| format!("跳板机地址无效: {}", e))?;
    let jump_tcp = TcpStream::connect_timeout(&jump_addr, std::time::Duration::from_secs(CONNECT_TIMEOUT_SECS))
        .map_err(|e| format!("跳板机连接失败: {}", e))?;
    let jump_session = create_session(jump_tcp, jump_host)?;
    jump_session
        .userauth_password(jump_user, jump_pass)
        .map_err(|e| format!("跳板机认证失败: {}", e))?;

    // Verify the jump host can reach the target (test channel, then drop it)
    // This catches configuration errors before we attempt the direct connection
    let test_channel = jump_session
        .channel_direct_tcpip(target_host, target_port, None)
        .map_err(|e| format!("跳板机隧道建立失败（目标不可达）: {}", e))?;
    drop(test_channel);
    // Note: We cannot use this channel for Session::new() because ssh2-rs requires
    // a TcpStream, not a Channel. The tunnel test is purely for validation.

    // Step 2: Connect directly to target (NOT through the jump host tunnel)
    // This assumes the target is network-reachable from the client machine
    let id = connect(target_host, target_port, target_user, target_pass).await?;

    // TODO: True ProxyJump support requires either:
    //   - Using OpenSSH's -J flag via a PTY shell: ssh -J user@jumphost user@target
    //   - A library that supports Session::new() over a Channel stream
    //   - libssh2 native ProxyJump support (not yet available in ssh2-rs)
    // For now, the jump session is dropped here since it's not used for tunneling.
    drop(jump_session);

    Ok(id)
}

// ─── Remote Command Execution (for monitoring) ───

pub async fn get_monitor_data(session_id: &str) -> Result<MonitorData, String> {
    // Try PTY shell sessions first, then exec sessions
    // For monitoring, we use exec (one-shot commands)
    let output = execute(session_id, "cat /proc/stat /proc/meminfo /proc/loadavg /proc/uptime && df -B1 / | tail -1").await?;
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

        // CPU from /proc/stat (first cpu line)
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

        // Memory from /proc/meminfo
        if line.starts_with("MemTotal:") {
            mem_total = line
                .split_whitespace()
                .nth(1)
                .and_then(|v| v.parse().ok())
                .unwrap_or(0)
                * 1024; // kB → bytes
        }
        if line.starts_with("MemAvailable:") {
            mem_available = line
                .split_whitespace()
                .nth(1)
                .and_then(|v| v.parse().ok())
                .unwrap_or(0)
                * 1024;
        }

        // Load average from /proc/loadavg
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

        // Uptime from /proc/uptime
        if line.contains('.') && line.split_whitespace().count() == 2 {
            if let Some(first) = line.split_whitespace().next() {
                if let Ok(uptime) = first.parse::<f64>() {
                    if uptime > 1.0 && uptime_seconds == 0 {
                        uptime_seconds = uptime as u64;
                    }
                }
            }
        }

        // Disk from df output
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

    let memory_used = mem_total - mem_available;
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
    fn test_unix_now_returns_positive() {
        let t = utils::unix_now();
        assert!(t > 0, "utils::unix_now() should return a value > 0");
        // Should be reasonable (after 2020-01-01 1577836800)
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
