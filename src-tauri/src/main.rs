mod config;
mod ssh;
mod sftp;

#[tauri::command]
async fn ssh_connect(host: String, port: u16, username: String, password: String) -> Result<String, String> {
    ssh::connect(&host, port, &username, &password).await
}

#[tauri::command]
async fn ssh_connect_key(host: String, port: u16, username: String, key_path: String, passphrase: String) -> Result<String, String> {
    ssh::connect_with_key(&host, port, &username, &key_path, &passphrase).await
}

#[tauri::command]
async fn ssh_execute(session_id: String, command: String) -> Result<String, String> {
    ssh::execute(&session_id, &command).await
}

#[tauri::command]
fn ssh_disconnect(session_id: String) -> Result<(), String> { ssh::disconnect(&session_id) }

#[tauri::command]
fn ssh_list_sessions() -> Vec<ssh::SshSession> { ssh::list_sessions() }

#[tauri::command]
fn load_connections() -> Result<Vec<config::ConnectionConfig>, String> { config::load_connections() }

#[tauri::command]
fn save_connection(conn: config::ConnectionConfig) -> Result<(), String> { config::save_connection(conn) }

#[tauri::command]
fn delete_connection(id: String) -> Result<(), String> { config::delete_connection(&id) }

#[tauri::command]
fn get_app_version() -> String { env!("CARGO_PKG_VERSION").to_string() }

#[tauri::command]
fn sftp_list_local(path: String) -> Result<Vec<sftp::FileEntry>, String> { sftp::list_local(&path) }

#[tauri::command]
fn sftp_list_remote(_sid: String, path: String) -> Result<Vec<sftp::FileEntry>, String> { sftp::list_remote(&_sid, &path) }

#[tauri::command]
fn sftp_upload(_sid: String, local: String, remote: String) -> Result<String, String> { sftp::upload(&_sid, &local, &remote) }

#[tauri::command]
fn sftp_download(_sid: String, remote: String, local: String) -> Result<String, String> { sftp::download(&_sid, &remote, &local) }

#[tauri::command]
fn open_file_dialog() -> Result<String, String> { Err("请在完整Tauri环境中使用".into()) }

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            ssh_connect, ssh_connect_key, ssh_execute,
            ssh_disconnect, ssh_list_sessions,
            load_connections, save_connection, delete_connection,
            get_app_version,
            sftp_list_local, sftp_list_remote, sftp_upload, sftp_download,
            open_file_dialog,
        ])
        .run(tauri::generate_context!())
        .expect("error running app");
}
