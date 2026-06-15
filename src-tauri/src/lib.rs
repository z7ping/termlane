#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod config;
mod local_pty;
mod sftp;
mod ssh;
mod updater;
mod utils;

fn main() {
    tauri::Builder::default()

        .invoke_handler(tauri::generate_handler![
            // SSH Exec
            commands::ssh_connect,
            commands::ssh_connect_key,
            commands::ssh_connect_jump,
            commands::ssh_execute,
            commands::ssh_disconnect,
            commands::ssh_list_sessions,
            commands::ssh_monitor,
            // SSH PTY Shell
            commands::ssh_start_shell,
            commands::ssh_shell_input,
            commands::ssh_shell_resize,
            commands::ssh_close_shell,
            commands::ssh_list_shells,
            // SFTP
            commands::sftp_list_local,
            commands::sftp_list_remote,
            commands::sftp_upload,
            commands::sftp_download,
            commands::sftp_rename,
            commands::sftp_delete,
            commands::sftp_mkdir,
            commands::sftp_chmod,
            commands::sftp_read_file,
            commands::sftp_write_file,
            // Config / Storage
            commands::load_connections,
            commands::save_connection,
            commands::delete_connection,
            commands::keyring_save_password,
            commands::keyring_load_password,
            commands::keyring_delete_password,
            commands::save_window_state,
            commands::load_window_state,
            commands::list_recordings,
            commands::save_recording_meta,
            commands::delete_recording,
            commands::get_recording_dir,
            // Local PTY
            commands::local_start_shell,
            commands::local_input,
            commands::local_resize,
            commands::local_close_shell,
            commands::local_list_shells,
            // Updater
            updater::check_update,
            // Network
            commands::tcp_ping,
            // App Info
            commands::get_app_version,
            // Greet (保留)
            commands::greet,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
