// commands.rs - Tauri command wrappers
// 所有前端可调用的命令集中在此，保持 ssh.rs / sftp.rs 等内部接口不变

use tauri::AppHandle;

// ─── Input Validation Helpers ───

/// Maximum allowed length for any string IPC parameter.
const MAX_STRING_LEN: usize = 10_000;

/// Validate that a host is non-empty and a port is in the valid TCP/UDP range.
fn validate_host_port(host: &str, port: u16) -> Result<(), String> {
    if host.trim().is_empty() {
        return Err("host must not be empty".into());
    }
    if port == 0 {
        return Err("port must be between 1 and 65535".into());
    }
    Ok(())
}

/// Reject any string parameter that exceeds the safety limit.
fn validate_string_len(name: &str, value: &str) -> Result<(), String> {
    if value.len() > MAX_STRING_LEN {
        return Err(format!("{} exceeds maximum length of {} chars", name, MAX_STRING_LEN));
    }
    Ok(())
}

// ─── SSH Exec ───

#[tauri::command(rename_all = "snake_case")]
pub async fn ssh_connect(host: String, port: u16, username: String, password: String) -> Result<String, String> {
    validate_host_port(&host, port)?;
    validate_string_len("host", &host)?;
    validate_string_len("username", &username)?;
    validate_string_len("password", &password)?;
    if username.trim().is_empty() {
        return Err("username must not be empty".into());
    }
    crate::ssh::connect(&host, port, &username, &password).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn ssh_connect_key(host: String, port: u16, username: String, key_path: String, passphrase: String) -> Result<String, String> {
    validate_host_port(&host, port)?;
    validate_string_len("host", &host)?;
    validate_string_len("username", &username)?;
    validate_string_len("key_path", &key_path)?;
    validate_string_len("passphrase", &passphrase)?;
    if username.trim().is_empty() {
        return Err("username must not be empty".into());
    }
    crate::ssh::connect_with_key(&host, port, &username, &key_path, &passphrase).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn ssh_connect_jump(
    jump_host: String, jump_port: u16, jump_user: String, jump_pass: String,
    target_host: String, target_port: u16, target_user: String, target_pass: String,
) -> Result<String, String> {
    validate_host_port(&jump_host, jump_port)?;
    validate_host_port(&target_host, target_port)?;
    validate_string_len("jump_host", &jump_host)?;
    validate_string_len("jump_user", &jump_user)?;
    validate_string_len("jump_pass", &jump_pass)?;
    validate_string_len("target_host", &target_host)?;
    validate_string_len("target_user", &target_user)?;
    validate_string_len("target_pass", &target_pass)?;
    if jump_user.trim().is_empty() {
        return Err("jump_user must not be empty".into());
    }
    if target_user.trim().is_empty() {
        return Err("target_user must not be empty".into());
    }
    crate::ssh::connect_jump(&jump_host, jump_port, &jump_user, &jump_pass, &target_host, target_port, &target_user, &target_pass).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn ssh_execute(session_id: String, command: String) -> Result<String, String> {
    crate::ssh::execute(&session_id, &command).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn ssh_monitor(session_id: String) -> Result<crate::ssh::MonitorData, String> {
    crate::ssh::get_monitor_data(&session_id).await
}

#[tauri::command(rename_all = "snake_case")]
pub fn ssh_list_sessions() -> Vec<crate::ssh::SshSession> {
    crate::ssh::list_sessions()
}

#[tauri::command(rename_all = "snake_case")]
pub fn ssh_disconnect(session_id: String) -> Result<(), String> {
    crate::ssh::disconnect(&session_id)
}

// ─── SSH PTY Shell ───

#[tauri::command(rename_all = "snake_case")]
pub fn ssh_start_shell(
    app: AppHandle,
    host: String, port: u16, username: String, password: String,
    key_path: Option<String>, passphrase: Option<String>,
    cols: u16, rows: u16,
) -> Result<String, String> {
    crate::ssh::start_shell(app, &host, port, &username, &password, key_path.as_deref(), passphrase.as_deref(), cols, rows)
}

#[tauri::command(rename_all = "snake_case")]
pub fn ssh_shell_input(session_id: String, data: String) -> Result<(), String> {
    crate::ssh::shell_input(&session_id, &data)
}

#[tauri::command(rename_all = "snake_case")]
pub fn ssh_shell_resize(session_id: String, cols: u16, rows: u16) -> Result<(), String> {
    crate::ssh::shell_resize(&session_id, cols, rows)
}

#[tauri::command(rename_all = "snake_case")]
pub fn ssh_close_shell(session_id: String) -> Result<(), String> {
    crate::ssh::close_shell(&session_id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn ssh_list_shells() -> Vec<String> {
    crate::ssh::list_shells()
}

// ─── SFTP ───

#[tauri::command(rename_all = "snake_case")]
pub fn sftp_list_local(path: String) -> Result<Vec<crate::sftp::FileEntry>, String> {
    crate::sftp::list_local(&path)
}

#[tauri::command(rename_all = "snake_case")]
pub fn sftp_list_remote(session_id: String, path: String) -> Result<Vec<crate::sftp::FileEntry>, String> {
    crate::sftp::list_remote(&session_id, &path)
}

#[tauri::command(rename_all = "snake_case")]
pub fn sftp_upload(session_id: String, local: String, remote: String) -> Result<String, String> {
    crate::sftp::upload(&session_id, &local, &remote)
}

#[tauri::command(rename_all = "snake_case")]
pub fn sftp_download(session_id: String, remote: String, local: String) -> Result<String, String> {
    crate::sftp::download(&session_id, &remote, &local)
}

#[tauri::command(rename_all = "snake_case")]
pub fn sftp_rename(session_id: String, old_path: String, new_path: String) -> Result<String, String> {
    crate::sftp::rename_file(&session_id, &old_path, &new_path)
}

#[tauri::command(rename_all = "snake_case")]
pub fn sftp_delete(session_id: String, path: String, is_dir: bool) -> Result<String, String> {
    validate_string_len("path", &path)?;
    if path.trim().is_empty() {
        return Err("path must not be empty".into());
    }
    // Prevent deletion of root-like paths (e.g. "/." or "/" followed by only dots)
    let trimmed = path.trim_end_matches('/');
    if trimmed.starts_with('/') && trimmed[1..].chars().all(|c| c == '.') {
        return Err("refusing to delete root-like path".into());
    }
    crate::sftp::delete_file(&session_id, &path, is_dir)
}

#[tauri::command(rename_all = "snake_case")]
pub fn sftp_mkdir(session_id: String, path: String) -> Result<String, String> {
    crate::sftp::create_dir(&session_id, &path)
}

#[tauri::command(rename_all = "snake_case")]
pub fn sftp_chmod(session_id: String, path: String, mode: String) -> Result<String, String> {
    crate::sftp::chmod(&session_id, &path, &mode)
}

#[tauri::command(rename_all = "snake_case")]
pub fn sftp_read_file(session_id: String, path: String) -> Result<String, String> {
    crate::sftp::read_file(&session_id, &path)
}

#[tauri::command(rename_all = "snake_case")]
pub fn sftp_write_file(session_id: String, path: String, content: String) -> Result<String, String> {
    crate::sftp::write_file(&session_id, &path, &content)
}

// ─── Config / Storage ───

#[tauri::command(rename_all = "snake_case")]
pub fn load_connections() -> Result<Vec<crate::config::ConnectionConfig>, String> {
    crate::config::load_connections()
}

#[tauri::command(rename_all = "snake_case")]
pub fn save_connection(conn: crate::config::ConnectionConfig) -> Result<(), String> {
    crate::config::save_connection(conn)
}

#[tauri::command(rename_all = "snake_case")]
pub fn delete_connection(id: String) -> Result<(), String> {
    crate::config::delete_connection(&id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn keyring_save_password(conn_id: String, password: String) -> Result<(), String> {
    crate::config::save_password(&conn_id, &password)
}

#[tauri::command(rename_all = "snake_case")]
pub fn keyring_load_password(conn_id: String) -> Result<String, String> {
    crate::config::load_password(&conn_id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn keyring_delete_password(conn_id: String) -> Result<(), String> {
    crate::config::delete_password(&conn_id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn save_window_state(state: crate::config::WindowState) -> Result<(), String> {
    crate::config::save_window_state(state)
}

#[tauri::command(rename_all = "snake_case")]
pub fn load_window_state() -> Result<crate::config::WindowState, String> {
    crate::config::load_window_state()
}

#[tauri::command(rename_all = "snake_case")]
pub fn list_recordings() -> Result<Vec<crate::config::RecordingMeta>, String> {
    crate::config::list_recordings()
}

#[tauri::command(rename_all = "snake_case")]
pub fn save_recording_meta(meta: crate::config::RecordingMeta) -> Result<(), String> {
    crate::config::save_recording_meta(meta)
}

#[tauri::command(rename_all = "snake_case")]
pub fn delete_recording(id: String) -> Result<(), String> {
    crate::config::delete_recording(&id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_recording_dir() -> Result<std::path::PathBuf, String> {
    Ok(crate::config::get_recording_dir())
}

// ─── Local PTY ───

#[tauri::command(rename_all = "snake_case")]
pub fn local_start_shell(app: AppHandle, cols: u16, rows: u16) -> Result<String, String> {
    crate::local_pty::start_local_shell(app, cols, rows, None, None)
}

#[tauri::command(rename_all = "snake_case")]
pub fn local_input(session_id: String, data: String) -> Result<(), String> {
    crate::local_pty::local_input(&session_id, &data)
}

#[tauri::command(rename_all = "snake_case")]
pub fn local_resize(session_id: String, cols: u16, rows: u16) -> Result<(), String> {
    crate::local_pty::local_resize(&session_id, cols, rows)
}

#[tauri::command(rename_all = "snake_case")]
pub fn local_close_shell(session_id: String) -> Result<(), String> {
    crate::local_pty::close_local_shell(&session_id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn local_list_shells() -> Vec<String> {
    crate::local_pty::list_local_shells()
}

// ─── Updater ───



// ─── Network ───

#[tauri::command(rename_all = "snake_case")]
pub fn tcp_ping(host: String, port: u16) -> Result<u64, String> {
    validate_host_port(&host, port)?;
    validate_string_len("host", &host)?;
    use std::net::TcpStream;
    use std::time::{Duration, Instant};
    let addr = format!("{}:{}", host, port);
    let start = Instant::now();
    TcpStream::connect_timeout(
        &addr.parse().map_err(|e: std::net::AddrParseError| e.to_string())?,
        Duration::from_secs(5),
    ).map_err(|e| format!("连接失败: {}", e))?;
    Ok(start.elapsed().as_millis() as u64)
}

// ─── App Info ───

#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// ─── Greet (保留) ───
