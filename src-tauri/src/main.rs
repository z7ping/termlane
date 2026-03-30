// XTerminal Pro - Main Entry
mod config;
mod ssh;
mod sftp;

use tauri::Manager;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! XTerminal Pro is running.", name)
}

#[tauri::command]
async fn ssh_connect(host: String, port: u16, username: String, password: String) -> Result<String, String> {
    ssh::connect(&host, port, &username, &password).await
}

#[tauri::command]
async fn ssh_execute(session_id: String, command: String) -> Result<String, String> {
    ssh::execute(&session_id, &command).await
}

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            greet,
            ssh_connect,
            ssh_execute,
            load_connections,
            save_connection,
            delete_connection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
