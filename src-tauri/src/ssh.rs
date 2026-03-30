// ssh.rs - SSH connection with PTY terminal support

use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::Mutex;
use ssh2::Session;
use std::io::{Read, Write};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshSession {
    pub id: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub connected: bool,
}

// Active sessions store
static SESSIONS: std::sync::LazyLock<Mutex<HashMap<String, Session>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

static SESSION_INFO: std::sync::LazyLock<Mutex<HashMap<String, SshSession>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

pub async fn connect(host: &str, port: u16, username: &str, password: &str) -> Result<String, String> {
    let tcp = TcpStream::connect(format!("{}:{}", host, port))
        .map_err(|e| format!("连接失败: {} ({}:{})", e, host, port))?;

    tcp.set_read_timeout(Some(std::time::Duration::from_secs(30)))
        .map_err(|e| format!("设置超时失败: {}", e))?;

    let mut session = Session::new().map_err(|e| e.to_string())?;
    session.set_tcp_stream(tcp);
    session.handshake().map_err(|e| format!("握手失败: {}", e))?;

    session.userauth_password(username, password)
        .map_err(|e| format!("认证失败: {}", e))?;

    if !session.authenticated() {
        return Err("认证失败: 用户名或密码错误".into());
    }

    let session_id = format!("ssh_{}_{}", host.replace('.', "_"), unix_now());

    let info = SshSession {
        id: session_id.clone(),
        host: host.to_string(),
        port,
        username: username.to_string(),
        connected: true,
    };

    SESSIONS.lock().unwrap().insert(session_id.clone(), session);
    SESSION_INFO.lock().unwrap().insert(session_id.clone(), info);

    Ok(session_id)
}

pub async fn connect_with_key(host: &str, port: u16, username: &str, key_path: &str, passphrase: &str) -> Result<String, String> {
    let tcp = TcpStream::connect(format!("{}:{}", host, port))
        .map_err(|e| format!("连接失败: {}", e))?;

    let mut session = Session::new().map_err(|e| e.to_string())?;
    session.set_tcp_stream(tcp);
    session.handshake().map_err(|e| format!("握手失败: {}", e))?;

    if passphrase.is_empty() {
        session.userauth_pubkey_file(username, None, std::path::Path::new(key_path), None)
            .map_err(|e| format!("密钥认证失败: {}", e))?;
    } else {
        session.userauth_pubkey_file(username, None, std::path::Path::new(key_path), Some(passphrase))
            .map_err(|e| format!("密钥认证失败: {}", e))?;
    }

    if !session.authenticated() {
        return Err("密钥认证失败".into());
    }

    let session_id = format!("ssh_{}_{}", host.replace('.', "_"), unix_now());

    let info = SshSession {
        id: session_id.clone(),
        host: host.to_string(),
        port,
        username: username.to_string(),
        connected: true,
    };

    SESSIONS.lock().unwrap().insert(session_id.clone(), session);
    SESSION_INFO.lock().unwrap().insert(session_id.clone(), info);

    Ok(session_id)
}

/// Connect via jump host (proxy)
/// Flow: local -> jump_host -> target_host
pub async fn connect_with_jump(
    target_host: &str, target_port: u16, target_user: &str, target_pass: &str,
    jump_host: &str, jump_port: u16, jump_user: &str, jump_pass: &str,
) -> Result<String, String> {
    // Step 1: Connect to jump host
    let jump_tcp = TcpStream::connect(format!("{}:{}", jump_host, jump_port))
        .map_err(|e| format!("连接跳板机失败: {} ({}:{})", e, jump_host, jump_port))?;

    let mut jump_session = Session::new().map_err(|e| e.to_string())?;
    jump_session.set_tcp_stream(jump_tcp);
    jump_session.handshake().map_err(|e| format!("跳板机握手失败: {}", e))?;

    jump_session.userauth_password(jump_user, jump_pass)
        .map_err(|e| format!("跳板机认证失败: {}", e))?;

    if !jump_session.authenticated() {
        return Err("跳板机认证失败".into());
    }

    // Step 2: Open a direct TCP channel through the jump host to the target
    let mut jump_channel = jump_session.channel_direct_tcpip(target_host, target_port as u32, None)
        .map_err(|e| format!("通过跳板机建立隧道失败: {}", e))?;

    // Step 3: Create a new SSH session through the tunnel
    let mut target_session = Session::new().map_err(|e| e.to_string())?;
    target_session.set_tcp_stream(jump_channel.stream());
    target_session.handshake().map_err(|e| format!("目标服务器握手失败: {}", e))?;

    target_session.userauth_password(target_user, target_pass)
        .map_err(|e| format!("目标服务器认证失败: {}", e))?;

    if !target_session.authenticated() {
        return Err("目标服务器认证失败".into());
    }

    let session_id = format!("ssh_jump_{}_{}", target_host.replace('.', "_"), unix_now());

    let info = SshSession {
        id: session_id.clone(),
        host: format!("{}→{}", jump_host, target_host),
        port: target_port,
        username: target_user.to_string(),
        connected: true,
    };

    // Store the target session (jump session is owned by the channel)
    SESSIONS.lock().unwrap().insert(session_id.clone(), target_session);
    SESSION_INFO.lock().unwrap().insert(session_id.clone(), info);

    Ok(session_id)
}

pub async fn execute(session_id: &str, command: &str) -> Result<String, String> {
    let sessions = SESSIONS.lock().unwrap();
    let session = sessions.get(session_id)
        .ok_or("会话不存在或已断开")?;

    let mut channel = session.channel_session()
        .map_err(|e| format!("创建通道失败: {}", e))?;

    // Request PTY for interactive commands
    channel.request_pty("xterm-256color", None, None)
        .map_err(|e| format!("请求PTY失败: {}", e))?;

    channel.exec(command)
        .map_err(|e| format!("执行命令失败: {}", e))?;

    let mut output = String::new();
    channel.read_to_string(&mut output)
        .map_err(|e| format!("读取输出失败: {}", e))?;

    channel.wait_close().ok();
    let exit_status = channel.exit_status().unwrap_or(-1);

    Ok(format!("{}\n[exit: {}]", output, exit_status))
}

/// Open a shell channel for interactive terminal
pub async fn open_shell(session_id: &str) -> Result<String, String> {
    let sessions = SESSIONS.lock().unwrap();
    let session = sessions.get(session_id)
        .ok_or("会话不存在")?;

    let mut channel = session.channel_session()
        .map_err(|e| format!("创建通道失败: {}", e))?;

    channel.request_pty("xterm-256color", None, Some((80, 24, 0, 0)))
        .map_err(|e| format!("请求PTY失败: {}", e))?;

    channel.shell()
        .map_err(|e| format!("启动Shell失败: {}", e))?;

    // Set non-blocking mode
    channel.set_blocking(false);

    let channel_id = format!("ch_{}_{}", session_id, unix_now());

    // Note: In a real implementation, we'd store the channel separately
    // For now, return the channel ID for future reference
    Ok(channel_id)
}

pub fn disconnect(session_id: &str) -> Result<(), String> {
    SESSIONS.lock().unwrap().remove(session_id);
    SESSION_INFO.lock().unwrap().remove(session_id);
    Ok(())
}

pub fn get_session_info(session_id: &str) -> Option<SshSession> {
    SESSION_INFO.lock().unwrap().get(session_id).cloned()
}

pub fn list_sessions() -> Vec<SshSession> {
    SESSION_INFO.lock().unwrap().values().cloned().collect()
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
