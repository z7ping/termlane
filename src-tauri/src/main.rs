mod config;
mod local_pty;
mod sftp;
mod ssh;
mod updater;

use std::io::{Read, Write};
use tauri::Manager;

// ─── Window state auto-save helpers ───

fn save_window_state_now(window: &tauri::WebviewWindow) {
    if let (Ok(pos), Ok(size), Ok(maximized)) = (window.outer_position(), window.outer_size(), window.is_maximized()) {
        let monitor_name = window.current_monitor().ok().flatten().map(|m| m.name().cloned().unwrap_or_default());

        let _ = config::save_window_state(config::WindowState {
            x: Some(pos.x),
            y: Some(pos.y),
            width: Some(size.width),
            height: Some(size.height),
            maximized: Some(maximized),
            display_id: monitor_name,
        });
    }
}

/// Check if a point is within any of the available monitors
fn is_position_on_monitor(window: &tauri::WebviewWindow, x: i32, y: i32) -> bool {
    if let Ok(monitors) = window.available_monitors() {
        for monitor in &monitors {
            let pos = monitor.position();
            let size = monitor.size();
            if x >= pos.x && x < pos.x + size.width as i32
                && y >= pos.y && y < pos.y + size.height as i32
            {
                return true;
            }
        }
    }
    false
}

// ─── Exec-based SSH (legacy) ───

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
fn ssh_disconnect(session_id: String) -> Result<(), String> {
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
    jump_host: String, jump_port: u16, jump_user: String, jump_pass: String,
    target_host: String, target_port: u16, target_user: String, target_pass: String,
) -> Result<String, String> {
    ssh::connect_jump(&jump_host, jump_port, &jump_user, &jump_pass, &target_host, target_port, &target_user, &target_pass).await
}

// ─── PTY Shell ───

#[tauri::command]
fn ssh_start_shell(
    app: tauri::AppHandle,
    host: String, port: u16, username: String, password: String,
    key_path: Option<String>, passphrase: Option<String>,
    cols: Option<u16>, rows: Option<u16>,
) -> Result<String, String> {
    ssh::start_shell(app, &host, port, &username, &password, key_path.as_deref(), passphrase.as_deref(), cols.unwrap_or(80), rows.unwrap_or(24))
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

// ─── TCP Ping ───

#[tauri::command]
async fn tcp_ping(host: String, port: u16) -> Result<u64, String> {
    use std::time::Instant;
    let addr = format!("{}:{}", host, port);
    let start = Instant::now();
    let _ = std::net::TcpStream::connect_timeout(
        &addr.parse().map_err(|e| format!("地址无效: {}", e))?,
        std::time::Duration::from_secs(5),
    ).map_err(|e| format!("连接失败: {}", e))?;
    Ok(start.elapsed().as_millis() as u64)
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

// ─── Keyring Password ───

#[tauri::command]
fn keyring_save_password(conn_id: String, password: String) -> Result<(), String> {
    config::save_password(&conn_id, &password)
}

#[tauri::command]
fn keyring_load_password(conn_id: String) -> Result<String, String> {
    config::load_password(&conn_id)
}

#[tauri::command]
fn keyring_delete_password(conn_id: String) -> Result<(), String> {
    config::delete_password(&conn_id)
}

// ─── Window State ───

#[tauri::command]
fn save_window_state(x: Option<i32>, y: Option<i32>, width: Option<u32>, height: Option<u32>, maximized: Option<bool>) -> Result<(), String> {
    config::save_window_state(config::WindowState {
        x, y, width, height, maximized,
        display_id: None,
    })
}

#[tauri::command]
fn load_window_state() -> Result<config::WindowState, String> {
    config::load_window_state()
}

// ─── Recordings ───

#[tauri::command]
fn list_recordings() -> Result<Vec<config::RecordingMeta>, String> {
    config::list_recordings()
}

#[tauri::command]
fn save_recording_meta(meta: config::RecordingMeta) -> Result<(), String> {
    config::save_recording_meta(meta)
}

#[tauri::command]
fn delete_recording(id: String) -> Result<(), String> {
    config::delete_recording(&id)
}

#[tauri::command]
fn get_recording_dir() -> Result<String, String> {
    Ok(config::get_recording_dir().to_string_lossy().to_string())
}

#[tauri::command]
fn save_recording_file(filename: String, content: String) -> Result<(), String> {
    let path = config::get_recording_dir().join(&filename);
    std::fs::write(&path, &content).map_err(|e| format!("写入录制文件失败: {}", e))
}

#[tauri::command]
fn read_recording_file(filename: String) -> Result<String, String> {
    let path = config::get_recording_dir().join(&filename);
    std::fs::read_to_string(&path).map_err(|e| format!("读取录制文件失败: {}", e))
}

// ─── Local PTY Shell ───

#[tauri::command]
fn local_start_shell(app: tauri::AppHandle, cols: Option<u16>, rows: Option<u16>, shell: Option<String>, cwd: Option<String>) -> Result<String, String> {
    local_pty::start_local_shell(app, cols.unwrap_or(80), rows.unwrap_or(24), shell.as_deref(), cwd.as_deref())
}

#[tauri::command]
fn local_shell_input(session_id: String, data: String) -> Result<(), String> {
    local_pty::local_input(&session_id, &data)
}

#[tauri::command]
fn local_shell_resize(session_id: String, cols: u16, rows: u16) -> Result<(), String> {
    local_pty::local_resize(&session_id, cols, rows)
}

#[tauri::command]
fn local_close_shell(session_id: String) -> Result<(), String> {
    local_pty::close_local_shell(&session_id)
}

#[tauri::command]
fn local_list_shells() -> Vec<String> {
    local_pty::list_local_shells()
}

// ─── SFTP ───

#[tauri::command]
fn sftp_list_local(path: String) -> Result<Vec<sftp::FileEntry>, String> { sftp::list_local(&path) }

#[tauri::command]
fn sftp_list_remote(session_id: String, path: String) -> Result<Vec<sftp::FileEntry>, String> { sftp::list_remote(&session_id, &path) }

#[tauri::command]
fn sftp_upload(session_id: String, local: String, remote: String) -> Result<String, String> { sftp::upload(&session_id, &local, &remote) }

#[tauri::command]
fn sftp_download(session_id: String, remote: String, local: String) -> Result<String, String> { sftp::download(&session_id, &remote, &local) }

#[tauri::command]
fn sftp_rename(session_id: String, old_path: String, new_path: String) -> Result<String, String> { sftp::rename_file(&session_id, &old_path, &new_path) }

#[tauri::command]
fn sftp_delete(session_id: String, path: String, is_dir: bool) -> Result<String, String> { sftp::delete_file(&session_id, &path, is_dir) }

#[tauri::command]
fn sftp_mkdir(session_id: String, path: String) -> Result<String, String> { sftp::create_dir(&session_id, &path) }

#[tauri::command]
fn sftp_chmod(session_id: String, path: String, mode: String) -> Result<String, String> { sftp::chmod(&session_id, &path, &mode) }

#[tauri::command]
fn sftp_read_file(session_id: String, path: String) -> Result<String, String> { sftp::read_file(&session_id, &path) }

#[tauri::command]
fn sftp_write_file(session_id: String, path: String, content: String) -> Result<String, String> { sftp::write_file(&session_id, &path, &content) }

#[tauri::command]
fn open_file_dialog() -> Result<String, String> { Err("请在完整Tauri环境中使用".into()) }

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_stronghold::Builder::new().build())
        .setup(|app| {
            let window = app.get_webview_window("main")
                .expect("failed to get main window");

            // ─── Restore window state on startup ───
            if let Ok(state) = config::load_window_state() {
                // Set size first (before position)
                if let (Some(w), Some(h)) = (state.width, state.height) {
                    let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
                        width: w,
                        height: h,
                    }));
                }

                // Restore position if it's on a valid monitor
                if let (Some(x), Some(y)) = (state.x, state.y) {
                    if is_position_on_monitor(&window, x, y) {
                        let _ = window.set_position(tauri::Position::Physical(
                            tauri::PhysicalPosition { x, y },
                        ));
                    } else {
                        // Saved position is outside current monitors, center on primary
                        let _ = window.center();
                    }
                }

                // Restore maximized state last
                if state.maximized == Some(true) {
                    let _ = window.maximize();
                }
            }

            // ─── Auto-save window state on events ───
            let win_for_move = window.clone();
            let win_for_resize = window.clone();
            let win_for_close = window.clone();

            window.on_window_event(move |event| {
                match event {
                    tauri::WindowEvent::Moved(_) => {
                        save_window_state_now(&win_for_move);
                    }
                    tauri::WindowEvent::Resized(_) => {
                        save_window_state_now(&win_for_resize);
                    }
                    tauri::WindowEvent::Destroyed => {
                        save_window_state_now(&win_for_close);
                    }
                    _ => {}
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // SSH
            ssh_connect, ssh_connect_key, ssh_execute, ssh_disconnect, ssh_list_sessions,
            ssh_connect_jump,
            // PTY Shell
            ssh_start_shell, ssh_shell_input, ssh_shell_resize, ssh_close_shell, ssh_list_shells,
            // Local PTY Shell
            local_start_shell, local_shell_input, local_shell_resize, local_close_shell, local_list_shells,
            // Monitoring
            ssh_monitor,
            // TCP Ping
            tcp_ping,
            // Config
            load_connections, save_connection, delete_connection, get_app_version,
            // Keyring
            keyring_save_password, keyring_load_password, keyring_delete_password,
            // Window State
            save_window_state, load_window_state,
            // Recordings
            list_recordings, save_recording_meta, delete_recording, get_recording_dir,
            save_recording_file, read_recording_file,
            // SFTP
            sftp_list_local, sftp_list_remote, sftp_upload, sftp_download,
            sftp_rename, sftp_delete, sftp_mkdir, sftp_chmod, sftp_read_file, sftp_write_file,
            // Updater
            updater::check_update,
            // Misc
            open_file_dialog,
        ])
        .run(tauri::generate_context!())
        .expect("error running app");
}
