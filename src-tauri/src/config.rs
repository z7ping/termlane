// config.rs - Connection configuration management with keyring support

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_type: String, // "password" or "key"
    pub group: String,
    pub icon: Option<String>,
    pub key_path: Option<String>,
    pub last_connected: Option<String>,
    // New fields
    pub color: Option<String>,     // Tag color: "red", "yellow", "green", "blue", "purple"
    pub tags: Option<Vec<String>>, // Tags: ["production", "web", "db"]
    pub proxy_type: Option<String>, // "none", "http", "socks5"
    pub proxy_host: Option<String>,
    pub proxy_port: Option<u16>,
    pub favorite: Option<bool>,
}

fn config_dir() -> PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("xterminal-pro");
    fs::create_dir_all(&dir).ok();
    dir
}

fn connections_file() -> PathBuf {
    config_dir().join("connections.json")
}

pub fn load_connections() -> Result<Vec<ConnectionConfig>, String> {
    let path = connections_file();
    if !path.exists() {
        return Ok(vec![]);
    }
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

pub fn save_connection(conn: ConnectionConfig) -> Result<(), String> {
    let mut connections = load_connections().unwrap_or_default();

    // Store password in keyring if auth_type is password
    if conn.auth_type == "password" {
        // Password is not stored in the JSON, only in keyring
        // The frontend should call save_password separately
    }

    if let Some(idx) = connections.iter().position(|c| c.id == conn.id) {
        connections[idx] = conn;
    } else {
        connections.push(conn);
    }

    let content = serde_json::to_string_pretty(&connections).map_err(|e| e.to_string())?;
    fs::write(connections_file(), content).map_err(|e| e.to_string())
}

pub fn delete_connection(id: &str) -> Result<(), String> {
    let mut connections = load_connections().unwrap_or_default();
    connections.retain(|c| c.id != id);

    // Also delete password from keyring
    delete_password(id).ok();

    let content = serde_json::to_string_pretty(&connections).map_err(|e| e.to_string())?;
    fs::write(connections_file(), content).map_err(|e| e.to_string())
}

// ─── Keyring password storage ───

pub fn save_password(conn_id: &str, password: &str) -> Result<(), String> {
    let entry = keyring::Entry::new("xterminal-pro", conn_id)
        .map_err(|e| format!("Keyring 初始化失败: {}", e))?;
    entry
        .set_password(password)
        .map_err(|e| format!("保存密码失败: {}", e))
}

pub fn load_password(conn_id: &str) -> Result<String, String> {
    let entry = keyring::Entry::new("xterminal-pro", conn_id)
        .map_err(|e| format!("Keyring 初始化失败: {}", e))?;
    entry
        .get_password()
        .map_err(|e| format!("读取密码失败: {}", e))
}

pub fn delete_password(conn_id: &str) -> Result<(), String> {
    let entry = keyring::Entry::new("xterminal-pro", conn_id)
        .map_err(|e| format!("Keyring 初始化失败: {}", e))?;
    entry
        .delete_credential()
        .map_err(|e| format!("删除密码失败: {}", e))
}

// ─── Window state persistence ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub maximized: Option<bool>,
}

pub fn save_window_state(state: WindowState) -> Result<(), String> {
    let path = config_dir().join("window_state.json");
    let content = serde_json::to_string_pretty(&state).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())
}

pub fn load_window_state() -> Result<WindowState, String> {
    let path = config_dir().join("window_state.json");
    if !path.exists() {
        return Ok(WindowState {
            x: None,
            y: None,
            width: Some(1200),
            height: Some(800),
            maximized: Some(false),
        });
    }
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

// ─── Session recordings storage ───

fn recordings_dir() -> PathBuf {
    let dir = config_dir().join("recordings");
    fs::create_dir_all(&dir).ok();
    dir
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingMeta {
    pub id: String,
    pub name: String,
    pub connection_name: String,
    pub started_at: String,
    pub duration_secs: u64,
    pub file_path: String,
    pub tags: Vec<String>,
}

pub fn list_recordings() -> Result<Vec<RecordingMeta>, String> {
    let path = recordings_dir().join("index.json");
    if !path.exists() {
        return Ok(vec![]);
    }
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

pub fn save_recording_meta(meta: RecordingMeta) -> Result<(), String> {
    let mut recordings = list_recordings().unwrap_or_default();
    if let Some(idx) = recordings.iter().position(|r| r.id == meta.id) {
        recordings[idx] = meta;
    } else {
        recordings.push(meta);
    }
    let path = recordings_dir().join("index.json");
    let content = serde_json::to_string_pretty(&recordings).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())
}

pub fn delete_recording(id: &str) -> Result<(), String> {
    let mut recordings = list_recordings().unwrap_or_default();
    if let Some(rec) = recordings.iter().find(|r| r.id == id) {
        // Delete the recording file
        let file_path = recordings_dir().join(&rec.file_path);
        fs::remove_file(&file_path).ok();
    }
    recordings.retain(|r| r.id != id);
    let path = recordings_dir().join("index.json");
    let content = serde_json::to_string_pretty(&recordings).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())
}

pub fn get_recording_dir() -> PathBuf {
    recordings_dir()
}
