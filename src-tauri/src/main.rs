mod config;
mod sftp;
mod ssh;

// ─── Exec-based SSH (legacy) ───

#[tauri::command]
async fn ssh_connect(
    host: String,
    port: u16,
    username: String,
    password: String,
) -> Result<String, String> {
    ssh::connect(&host, port, &username, &password).await
}

#[tauri::command]
async fn ssh_connect_key(
    host: String,
    port: u16,
    username: String,
    key_path: String,
    passphrase: String,
) -> Result<String, String> {
    ssh::connect_with_key(&host, port, &username, &key_path, &passphrase).await
}

#[tauri::command]
async fn ssh_execute(session_id: String, command: String) -> Result<String, String> {
    ssh::execute(&session_id, &command).await
}

#[tauri::command]
fn ssh_disconnect(session_id: String) -> Result<(), String> {
    // Close exec session or PTY shell
    let _ = ssh::close_shell(&session_id);
    ssh::disconnect(&session_id)
}

#[tauri::command]
fn ssh_list_sessions() -> Vec<ssh::SshSession> {
    ssh::list_sessions()
}

// ─── Jump Host ───

#[tauri::command]
async fn ssh_connect_jump(
    jump_host: String,
    jump_port: u16,
    jump_user: String,
    jump_pass: String,
    target_host: String,
    target_port: u16,
    target_user: String,
    target_pass: String,
) -> Result<String, String> {
    ssh::connect_jump(
        &jump_host,
        jump_port,
        &jump_user,
        &jump_pass,
        &target_host,
        target_port,
        &target_user,
        &target_pass,
    )
    .await
}

// ─── PTY Shell ───

#[tauri::command]
fn ssh_start_shell(
    app: tauri::AppHandle,
    host: String,
    port: u16,
    username: String,
    password: String,
    key_path: Option<String>,
    passphrase: Option<String>,
    cols: Option<u16>,
    rows: Option<u16>,
) -> Result<String, String> {
    ssh::start_shell(
        app,
        &host,
        port,
        &username,
        &password,
        key_path.as_deref(),
        passphrase.as_deref(),
        cols.unwrap_or(80),
        rows.unwrap_or(24),
    )
}

#[tauri::command]
fn ssh_shell_input(session_id: String, data: String) -> Result<(), String> {
    ssh::shell_input(&session_id, &data)
}

#[tauri::command]
fn ssh_shell_resize(session_id: String, cols: u16, rows: u16) -> Result<(), String> {
    ssh::shell_resize(&session_id, cols, rows)
}

#[tauri::command]
fn ssh_close_shell(session_id: String) -> Result<(), String> {
    ssh::close_shell(&session_id)
}

#[tauri::command]
fn ssh_list_shells() -> Vec<String> {
    ssh::list_shells()
}

// ─── Monitoring ───

#[tauri::command]
async fn ssh_monitor(session_id: String) -> Result<ssh::MonitorData, String> {
    ssh::get_monitor_data(&session_id).await
}

// ─── Config ───

#[tauri::command]
fn load_connections() -> Result<Vec<config::ConnectionConfig>, String> {
    config::load_connections()
}

#[tauri::command]
fn save_connection(conn: config::ConnectionConfig) -> Result<(), String> {
    config::save_connection(conn)
}

#[tauri::command]
fn delete_connection(id: String) -> Result<(), String> {
    config::delete_connection(&id)
}

#[tauri::command]
fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// ─── SFTP ───

#[tauri::command]
fn sftp_list_local(path: String) -> Result<Vec<sftp::FileEntry>, String> {
    sftp::list_local(&path)
}

#[tauri::command]
fn sftp_list_remote(session_id: String, path: String) -> Result<Vec<sftp::FileEntry>, String> {
    sftp::list_remote(&session_id, &path)
}

#[tauri::command]
fn sftp_upload(session_id: String, local: String, remote: String) -> Result<String, String> {
    sftp::upload(&session_id, &local, &remote)
}

#[tauri::command]
fn sftp_download(session_id: String, remote: String, local: String) -> Result<String, String> {
    sftp::download(&session_id, &remote, &local)
}

#[tauri::command]
fn sftp_rename(
    session_id: String,
    old_path: String,
    new_path: String,
) -> Result<String, String> {
    sftp::rename_file(&session_id, &old_path, &new_path)
}

#[tauri::command]
fn sftp_delete(session_id: String, path: String, is_dir: bool) -> Result<String, String> {
    sftp::delete_file(&session_id, &path, is_dir)
}

#[tauri::command]
fn sftp_mkdir(session_id: String, path: String) -> Result<String, String> {
    sftp::create_dir(&session_id, &path)
}

#[tauri::command]
fn sftp_chmod(session_id: String, path: String, mode: String) -> Result<String, String> {
    sftp::chmod(&session_id, &path, &mode)
}

#[tauri::command]
fn sftp_read_file(session_id: String, path: String) -> Result<String, String> {
    sftp::read_file(&session_id, &path)
}

#[tauri::command]
fn sftp_write_file(session_id: String, path: String, content: String) -> Result<String, String> {
    sftp::write_file(&session_id, &path, &content)
}

#[tauri::command]
fn open_file_dialog() -> Result<String, String> {
    Err("请在完整Tauri环境中使用".into())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // Exec-based SSH
            ssh_connect,
            ssh_connect_key,
            ssh_execute,
            ssh_disconnect,
            ssh_list_sessions,
            // Jump host
            ssh_connect_jump,
            // PTY Shell
            ssh_start_shell,
            ssh_shell_input,
            ssh_shell_resize,
            ssh_close_shell,
            ssh_list_shells,
            // Monitoring
            ssh_monitor,
            // Config
            load_connections,
            save_connection,
            delete_connection,
            get_app_version,
            // SFTP
            sftp_list_local,
            sftp_list_remote,
            sftp_upload,
            sftp_download,
            sftp_rename,
            sftp_delete,
            sftp_mkdir,
            sftp_chmod,
            sftp_read_file,
            sftp_write_file,
            // Misc
            open_file_dialog,
        ])
        .run(tauri::generate_context!())
        .expect("error running app");
}
