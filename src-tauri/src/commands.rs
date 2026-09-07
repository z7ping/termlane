use crate::utils::*;
// commands.rs - Tauri command wrappers
// 所有前端可调用的命令集中在此，保持 ssh.rs / sftp.rs 等内部接口不变

use tauri::AppHandle;

// ─── Input Validation Helpers ───

/// 通用短字符串参数上限：主机、路径、用户名、命令等。
const MAX_STRING_LEN: usize = 10_000;

fn validate_host_port(host: &str, port: u16) -> Result<(), String> {
    if host.trim().is_empty() {
        return Err("host must not be empty".into());
    }
    if port == 0 {
        return Err("port must be between 1 and 65535".into());
    }
    Ok(())
}

fn validate_string_len(name: &str, value: &str) -> Result<(), String> {
    if value.len() > MAX_STRING_LEN {
        return Err(format!("{} exceeds maximum length of {} bytes", name, MAX_STRING_LEN));
    }
    Ok(())
}

fn validate_file_content_len(content: &str) -> Result<(), String> {
    if content.len() > MAX_INLINE_EDIT_BYTES {
        return Err(format!(
            "文件内容过大：{} bytes，在线编辑最大允许 {} bytes",
            content.len(), MAX_INLINE_EDIT_BYTES
        ));
    }
    Ok(())
}

fn validate_remote_delete_path(path: &str) -> Result<(), String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("远程删除路径不能为空".into());
    }

    let absolute = trimmed.starts_with('/');
    let mut depth = 0i32;
    for component in trimmed.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                depth -= 1;
                if depth < 0 {
                    return Err("拒绝删除包含越界父目录的路径".into());
                }
            }
            _ => depth += 1,
        }
    }

    if depth <= 0 || (absolute && trimmed.trim_matches('/').is_empty()) {
        return Err("拒绝删除根目录或等价路径".into());
    }
    Ok(())
}

// ─── SSH Connection Test ───

#[tauri::command]
pub async fn ssh_connect(
    host: String,
    port: u16,
    username: String,
    password: String,
    trust_new_host_key: bool,
) -> Result<String, String> {
    validate_host_port(&host, port)?;
    validate_string_len("host", &host)?;
    validate_string_len("username", &username)?;
    validate_string_len("password", &password)?;
    if username.trim().is_empty() {
        return Err("username must not be empty".into());
    }
    crate::ssh::connect(&host, port, &username, &password, trust_new_host_key).await
}

#[tauri::command]
pub async fn ssh_connect_key(
    host: String,
    port: u16,
    username: String,
    key_path: String,
    passphrase: String,
    trust_new_host_key: bool,
) -> Result<String, String> {
    validate_host_port(&host, port)?;
    validate_string_len("host", &host)?;
    validate_string_len("username", &username)?;
    validate_string_len("key_path", &key_path)?;
    validate_string_len("passphrase", &passphrase)?;
    if username.trim().is_empty() {
        return Err("username must not be empty".into());
    }
    crate::ssh::connect_with_key(
        &host,
        port,
        &username,
        &key_path,
        &passphrase,
        trust_new_host_key,
    )
    .await
}

#[tauri::command]
pub fn ssh_disconnect(session_id: String) -> Result<(), String> {
    crate::ssh::disconnect(&session_id)
}

// ─── SSH PTY Shell ───

#[tauri::command(async)]
pub fn ssh_start_shell(
    app: AppHandle,
    session_id: String,
    host: String,
    port: u16,
    username: String,
    password: String,
    key_path: Option<String>,
    passphrase: Option<String>,
    trust_new_host_key: bool,
    cols: u16,
    rows: u16,
) -> Result<String, String> {
    validate_host_port(&host, port)?;
    validate_string_len("session_id", &session_id)?;
    validate_string_len("host", &host)?;
    validate_string_len("username", &username)?;
    validate_string_len("password", &password)?;
    if let Some(ref key_path) = key_path {
        validate_string_len("key_path", key_path)?;
    }
    if let Some(ref passphrase) = passphrase {
        validate_string_len("passphrase", passphrase)?;
    }
    if username.trim().is_empty() {
        return Err("username must not be empty".into());
    }
    crate::ssh::start_shell(
        app,
        &session_id,
        &host,
        port,
        &username,
        &password,
        key_path.as_deref(),
        passphrase.as_deref(),
        trust_new_host_key,
        cols,
        rows,
    )
}

#[tauri::command]
pub fn ssh_shell_input(session_id: String, data: String) -> Result<(), String> {
    crate::ssh::shell_input(&session_id, &data)
}

#[tauri::command]
pub fn ssh_shell_resize(session_id: String, cols: u16, rows: u16) -> Result<(), String> {
    crate::ssh::shell_resize(&session_id, cols, rows)
}

#[tauri::command]
pub fn ssh_close_shell(session_id: String) -> Result<(), String> {
    crate::ssh::close_shell(&session_id)
}

#[tauri::command]
pub fn ssh_list_shells() -> Vec<String> {
    crate::ssh::list_shells()
}

// ─── SFTP / Local Files ───

#[tauri::command(async)]
pub fn sftp_list_local(path: String) -> Result<Vec<crate::sftp::FileEntry>, String> {
    validate_string_len("path", &path)?;
    crate::sftp::list_local(&path)
}

#[tauri::command(async)]
pub fn sftp_list_remote(session_id: String, path: String) -> Result<Vec<crate::sftp::FileEntry>, String> {
    validate_string_len("path", &path)?;
    crate::sftp::list_remote(&session_id, &path)
}

#[tauri::command(async)]
pub fn sftp_upload(session_id: String, local: String, remote: String) -> Result<String, String> {
    validate_string_len("local", &local)?;
    validate_string_len("remote", &remote)?;
    crate::sftp::upload(&session_id, &local, &remote)
}

#[tauri::command(async)]
pub fn sftp_download(session_id: String, remote: String, local: String) -> Result<String, String> {
    validate_string_len("remote", &remote)?;
    validate_string_len("local", &local)?;
    crate::sftp::download(&session_id, &remote, &local)
}

#[tauri::command(async)]
pub fn sftp_rename(session_id: String, old_path: String, new_path: String) -> Result<String, String> {
    validate_string_len("old_path", &old_path)?;
    validate_string_len("new_path", &new_path)?;
    crate::sftp::rename_file(&session_id, &old_path, &new_path)
}

#[tauri::command(async)]
pub fn sftp_delete(session_id: String, path: String, is_dir: bool) -> Result<String, String> {
    validate_string_len("path", &path)?;
    validate_remote_delete_path(&path)?;
    crate::sftp::delete_file(&session_id, &path, is_dir)
}

#[tauri::command(async)]
pub fn sftp_mkdir(session_id: String, path: String) -> Result<String, String> {
    validate_string_len("path", &path)?;
    crate::sftp::create_dir(&session_id, &path)
}

#[tauri::command(async)]
pub fn sftp_chmod(session_id: String, path: String, mode: String) -> Result<String, String> {
    validate_string_len("path", &path)?;
    validate_string_len("mode", &mode)?;
    crate::sftp::chmod(&session_id, &path, &mode)
}

#[tauri::command(async)]
pub fn sftp_read_file(session_id: String, path: String) -> Result<String, String> {
    validate_string_len("path", &path)?;
    crate::sftp::read_file(&session_id, &path)
}

#[tauri::command(async)]
pub fn sftp_write_file(session_id: String, path: String, content: String) -> Result<String, String> {
    validate_string_len("path", &path)?;
    validate_file_content_len(&content)?;
    crate::sftp::write_file(&session_id, &path, &content)
}

// ─── Config / Storage ───

#[tauri::command]
pub fn load_connections() -> Result<Vec<crate::config::ConnectionConfig>, String> {
    crate::config::load_connections()
}

#[tauri::command]
pub fn save_connection(conn: crate::config::ConnectionConfig) -> Result<(), String> {
    crate::config::save_connection(conn)
}

#[tauri::command]
pub fn delete_connection(id: String) -> Result<(), String> {
    crate::config::delete_connection(&id)
}

#[tauri::command]
pub fn keyring_save_password(conn_id: String, password: String) -> Result<(), String> {
    crate::config::save_password(&conn_id, &password)
}

#[tauri::command]
pub fn keyring_load_password(conn_id: String) -> Result<String, String> {
    crate::config::load_password(&conn_id)
}

#[tauri::command]
pub fn keyring_delete_password(conn_id: String) -> Result<(), String> {
    crate::config::delete_password(&conn_id)
}

#[tauri::command]
pub fn list_recordings() -> Result<Vec<crate::config::RecordingMeta>, String> {
    crate::config::list_recordings()
}

#[tauri::command]
pub fn save_recording_meta(meta: crate::config::RecordingMeta) -> Result<(), String> {
    crate::config::save_recording_meta(meta)
}

#[tauri::command]
pub fn delete_recording(id: String) -> Result<(), String> {
    crate::config::delete_recording(&id)
}

#[tauri::command]
pub fn get_recording_dir() -> Result<std::path::PathBuf, String> {
    Ok(crate::config::get_recording_dir())
}

// ─── Local PTY ───

#[tauri::command]
pub fn local_start_shell(app: AppHandle, cols: u16, rows: u16) -> Result<String, String> {
    crate::local_pty::start_local_shell(app, cols, rows, None, None)
}

#[tauri::command]
pub fn local_input(session_id: String, data: String) -> Result<(), String> {
    crate::local_pty::local_input(&session_id, &data)
}

#[tauri::command]
pub fn local_resize(session_id: String, cols: u16, rows: u16) -> Result<(), String> {
    crate::local_pty::local_resize(&session_id, cols, rows)
}

#[tauri::command]
pub fn local_close_shell(session_id: String) -> Result<(), String> {
    crate::local_pty::close_local_shell(&session_id)
}

#[tauri::command]
pub fn local_list_shells() -> Vec<String> {
    crate::local_pty::list_local_shells()
}

// ─── Network ───

#[tauri::command(async)]
pub fn tcp_ping(host: String, port: u16) -> Result<u64, String> {
    validate_host_port(&host, port)?;
    validate_string_len("host", &host)?;
    let start = std::time::Instant::now();
    connect_tcp(&host, port, TCP_PING_TIMEOUT_SECS)?;
    Ok(start.elapsed().as_millis() as u64)
}

// ─── App Info ───

#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_string_limit_remains_strict() {
        assert!(validate_string_len("path", &"x".repeat(MAX_STRING_LEN)).is_ok());
        assert!(validate_string_len("path", &"x".repeat(MAX_STRING_LEN + 1)).is_err());
    }

    #[test]
    fn file_content_has_independent_limit() {
        assert!(validate_file_content_len(&"x".repeat(MAX_STRING_LEN + 1)).is_ok());
        assert!(validate_file_content_len(&"x".repeat(MAX_INLINE_EDIT_BYTES + 1)).is_err());
    }

    #[test]
    fn remote_delete_rejects_root_equivalents() {
        for path in ["", "/", "///", ".", "..", "/a/..", "a/.."] {
            assert!(validate_remote_delete_path(path).is_err(), "should reject {path:?}");
        }
        assert!(validate_remote_delete_path("/home/user/file").is_ok());
        assert!(validate_remote_delete_path("/home/user/../other").is_ok());
    }
}