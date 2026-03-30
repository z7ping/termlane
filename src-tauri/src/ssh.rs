// ssh.rs - SSH connection management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::Mutex;
use ssh2::Session;
use std::io::Read;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshSession {
    pub id: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub connected: bool,
}

static SESSIONS: std::sync::LazyLock<Mutex<HashMap<String, Session>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

static SESSION_INFO: std::sync::LazyLock<Mutex<HashMap<String, SshSession>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

fn create_session(tcp: TcpStream) -> Result<Session, String> {
    let mut session = Session::new().map_err(|e| e.to_string())?;
    session.set_tcp_stream(tcp);
    session.handshake().map_err(|e| format!("握手失败: {}", e))?;
    Ok(session)
}

pub async fn connect(host: &str, port: u16, username: &str, password: &str) -> Result<String, String> {
    let tcp = TcpStream::connect(format!("{}:{}", host, port))
        .map_err(|e| format!("连接失败: {}", e))?;
    tcp.set_read_timeout(Some(std::time::Duration::from_secs(30))).ok();

    let mut session = create_session(tcp)?;
    session.userauth_password(username, password)
        .map_err(|e| format!("认证失败: {}", e))?;
    if !session.authenticated() { return Err("用户名或密码错误".into()); }

    let id = format!("ssh_{}_{}", host.replace('.', "_"), unix_now());
    SESSION_INFO.lock().unwrap().insert(id.clone(), SshSession {
        id: id.clone(), host: host.into(), port, username: username.into(), connected: true,
    });
    SESSIONS.lock().unwrap().insert(id.clone(), session);
    Ok(id)
}

pub async fn connect_with_key(host: &str, port: u16, username: &str, key_path: &str, passphrase: &str) -> Result<String, String> {
    let tcp = TcpStream::connect(format!("{}:{}", host, port)).map_err(|e| format!("连接失败: {}", e))?;
    let mut session = create_session(tcp)?;
    if passphrase.is_empty() {
        session.userauth_pubkey_file(username, None, std::path::Path::new(key_path), None).map_err(|e| format!("密钥认证失败: {}", e))?;
    } else {
        session.userauth_pubkey_file(username, None, std::path::Path::new(key_path), Some(passphrase)).map_err(|e| format!("密钥认证失败: {}", e))?;
    }
    if !session.authenticated() { return Err("密钥认证失败".into()); }
    let id = format!("ssh_{}_{}", host.replace('.', "_"), unix_now());
    SESSION_INFO.lock().unwrap().insert(id.clone(), SshSession {
        id: id.clone(), host: host.into(), port, username: username.into(), connected: true,
    });
    SESSIONS.lock().unwrap().insert(id.clone(), session);
    Ok(id)
}

pub async fn execute(session_id: &str, command: &str) -> Result<String, String> {
    let sessions = SESSIONS.lock().unwrap();
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
    SESSIONS.lock().unwrap().remove(session_id);
    SESSION_INFO.lock().unwrap().remove(session_id);
    Ok(())
}

pub fn list_sessions() -> Vec<SshSession> {
    SESSION_INFO.lock().unwrap().values().cloned().collect()
}

fn unix_now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
}
