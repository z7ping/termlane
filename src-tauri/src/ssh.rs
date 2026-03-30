// ssh.rs - SSH connection management

use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::Mutex;
use ssh2::Session;
use std::io::Read;

// Active sessions store
static SESSIONS: std::sync::LazyLock<Mutex<HashMap<String, Session>>> = 
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

pub async fn connect(host: &str, port: u16, username: &str, password: &str) -> Result<String, String> {
    let tcp = TcpStream::connect(format!("{}:{}", host, port))
        .map_err(|e| format!("连接失败: {}", e))?;
    
    let mut session = Session::new().map_err(|e| e.to_string())?;
    session.set_tcp_stream(tcp);
    session.handshake().map_err(|e| format!("握手失败: {}", e))?;
    
    session.userauth_password(username, password)
        .map_err(|e| format!("认证失败: {}", e))?;
    
    if !session.authenticated() {
        return Err("认证失败: 用户名或密码错误".into());
    }
    
    let session_id = format!("ssh_{}_{}", host, chrono_timestamp());
    
    SESSIONS.lock().unwrap().insert(session_id.clone(), session);
    
    Ok(session_id)
}

pub async fn execute(session_id: &str, command: &str) -> Result<String, String> {
    let sessions = SESSIONS.lock().unwrap();
    let session = sessions.get(session_id)
        .ok_or("会话不存在")?;
    
    let mut channel = session.channel_session()
        .map_err(|e| format!("创建通道失败: {}", e))?;
    
    channel.exec(command)
        .map_err(|e| format!("执行命令失败: {}", e))?;
    
    let mut output = String::new();
    channel.read_to_string(&mut output)
        .map_err(|e| format!("读取输出失败: {}", e))?;
    
    channel.wait_close().ok();
    let exit_status = channel.exit_status().unwrap_or(-1);
    
    Ok(format!("{}\n[exit: {}]", output, exit_status))
}

pub fn disconnect(session_id: &str) {
    SESSIONS.lock().unwrap().remove(session_id);
}

fn chrono_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
