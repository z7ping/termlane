use crate::utils::*;
// ssh.rs - SSH connection management with PTY shell support

use ssh2::{CheckResult, ErrorCode, HashType, HostKeyType, KnownHostFileKind, KnownHostKeyFormat, Session};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

// ─── State ───

/// Short-lived authenticated sessions used by the connection-test flow.
static SESSIONS: std::sync::LazyLock<Mutex<HashMap<String, Session>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

/// PTY shell sessions: session_id → PtyShell handle.
struct PtyShell {
    input_tx: crossbeam_channel::Sender<String>,
    resize_tx: crossbeam_channel::Sender<(u16, u16)>,
    _reader: std::thread::JoinHandle<()>,
}

static PTY_SHELLS: std::sync::LazyLock<Mutex<HashMap<String, PtyShell>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

/// Authenticated SSH Session objects backing interactive PTY shells.
/// SFTP channels reuse the same transport and credentials.
static PTY_SESSIONS: std::sync::LazyLock<Mutex<HashMap<String, Session>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

static SESSION_SEQUENCE: AtomicU64 = AtomicU64::new(0);

// ─── Helpers ───

fn new_session_id(kind: &str) -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let sequence = SESSION_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!("ssh-{}-{:x}-{:x}", kind, timestamp, sequence)
}

fn validate_shell_session_id(session_id: &str) -> Result<(), String> {
    if session_id.is_empty() || session_id.len() > 128 {
        return Err("无效的 SSH Shell Session ID".to_string());
    }
    if !session_id
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return Err("SSH Shell Session ID 只能包含字母、数字、- 和 _".to_string());
    }
    if lock!(PTY_SHELLS).contains_key(session_id) || lock!(PTY_SESSIONS).contains_key(session_id) {
        return Err("SSH Shell Session ID 已存在".to_string());
    }
    Ok(())
}

fn emit_shell_lifecycle(app: &AppHandle, session_id: &str, kind: &str, message: Option<&str>) {
    let _ = app.emit(
        &format!("ssh-lifecycle:{}", session_id),
        serde_json::json!({
            "kind": kind,
            "message": message,
        }),
    );
}

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

fn write_channel_all(channel: &mut ssh2::Channel, data: &[u8]) -> Result<(), String> {
    let started = std::time::Instant::now();
    let mut written = 0usize;

    while written < data.len() {
        match channel.write(&data[written..]) {
            Ok(0) => return Err("SSH 输入通道未写入任何数据".to_string()),
            Ok(count) => written += count,
            Err(error)
                if (error.kind() == std::io::ErrorKind::WouldBlock
                    || error.kind() == std::io::ErrorKind::TimedOut)
                    && started.elapsed() < std::time::Duration::from_secs(READ_TIMEOUT_SECS) =>
            {
                std::thread::sleep(std::time::Duration::from_millis(PTY_POLL_FAST_MS));
            }
            Err(error)
                if error.kind() == std::io::ErrorKind::WouldBlock
                    || error.kind() == std::io::ErrorKind::TimedOut =>
            {
                return Err("SSH 输入写入超时".to_string());
            }
            Err(error) => return Err(format!("SSH 输入写入失败: {}", error)),
        }
    }

    Ok(())
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

// ─── Connection test API ───

pub async fn connect(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
    trust_new_host_key: bool,
) -> Result<String, String> {
    let tcp = connect_tcp(host, port, CONNECT_TIMEOUT_SECS)?;
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

    let id = new_session_id("test");
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
    let tcp = connect_tcp(host, port, CONNECT_TIMEOUT_SECS)?;
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

    let id = new_session_id("test");
    lock!(SESSIONS).insert(id.clone(), session);
    Ok(id)
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
    Ok(())
}

// ─── PTY Shell API ───

pub fn start_shell(
    app: AppHandle,
    session_id: &str,
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
    validate_shell_session_id(session_id)?;

    let tcp = connect_tcp(host, port, CONNECT_TIMEOUT_SECS)?;
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

    let session_id = session_id.to_string();
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
                    emit_shell_lifecycle(&app_handle, &reader_sid, "exited", None);
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
                        emit_shell_lifecycle(&app_handle, &reader_sid, "exited", None);
                        break;
                    }
                    consecutive_empty += 1;
                }
                Err(error) => {
                    emit_shell_lifecycle(
                        &app_handle,
                        &reader_sid,
                        "disconnected",
                        Some(&error.to_string()),
                    );
                    break;
                }
            }

            while let Ok(input) = input_rx.try_recv() {
                if input == "\x04" {
                    let _ = channel.send_eof();
                    emit_shell_lifecycle(&app_handle, &reader_sid, "exited", None);
                    break 'shell;
                }

                if let Err(error) = write_channel_all(&mut channel, input.as_bytes()) {
                    emit_shell_lifecycle(
                        &app_handle,
                        &reader_sid,
                        "disconnected",
                        Some(&error),
                    );
                    break 'shell;
                }
            }

            while let Ok((cols, rows)) = resize_rx.try_recv() {
                let _ = channel.request_pty_size(cols as u32, rows as u32, None, None);
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
    lock!(PTY_SESSIONS).insert(session_id, session);

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
    fn test_known_host_entry_ipv6_custom_port() {
        assert_eq!(known_host_entry("2001:db8::1", 2222), "[2001:db8::1]:2222");
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
    fn session_ids_do_not_collide() {
        let first = new_session_id("test");
        let second = new_session_id("test");
        assert_ne!(first, second);
    }

    #[test]
    fn shell_session_id_validation_is_strict() {
        assert!(validate_shell_session_id("ssh-shell-abc_123").is_ok());
        assert!(validate_shell_session_id("").is_err());
        assert!(validate_shell_session_id("bad/id").is_err());
    }
}