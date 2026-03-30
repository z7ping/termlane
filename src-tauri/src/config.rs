// config.rs - Connection configuration management

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
    
    // Update or insert
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
    let content = serde_json::to_string_pretty(&connections).map_err(|e| e.to_string())?;
    fs::write(connections_file(), content).map_err(|e| e.to_string())
}
