// config.rs - Connection configuration management with keyring support

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

// ─── Session recordings storage ───

fn recordings_dir() -> PathBuf {
    let dir = config_dir().join("recordings");
    fs::create_dir_all(&dir).ok();
    dir
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_config_serialize_deserialize() {
        let conn = ConnectionConfig {
            id: "test-001".into(),
            name: "My Server".into(),
            host: "192.168.1.100".into(),
            port: 22,
            username: "admin".into(),
            auth_type: "password".into(),
            group: "Production".into(),
            icon: Some("🖥️".into()),
            key_path: None,
            last_connected: Some("2024-01-15 10:30".into()),
            color: Some("red".into()),
            tags: Some(vec!["production".into(), "web".into()]),
            proxy_type: Some("socks5".into()),
            proxy_host: Some("127.0.0.1".into()),
            proxy_port: Some(1080),
            favorite: Some(true),
        };

        let json = serde_json::to_string_pretty(&conn).expect("serialize ConnectionConfig");
        assert!(json.contains("test-001"));
        assert!(json.contains("My Server"));
        assert!(json.contains("production"));

        let back: ConnectionConfig = serde_json::from_str(&json).expect("deserialize ConnectionConfig");
        assert_eq!(back.id, "test-001");
        assert_eq!(back.name, "My Server");
        assert_eq!(back.port, 22);
        assert_eq!(back.auth_type, "password");
        assert_eq!(back.color.as_deref(), Some("red"));
        assert_eq!(back.tags.as_ref().unwrap().len(), 2);
        assert_eq!(back.proxy_port, Some(1080));
        assert_eq!(back.favorite, Some(true));
    }

    #[test]
    fn test_connection_config_minimal() {
        let conn = ConnectionConfig {
            id: "min".into(),
            name: "Minimal".into(),
            host: "10.0.0.1".into(),
            port: 22,
            username: "root".into(),
            auth_type: "key".into(),
            group: "".into(),
            icon: None,
            key_path: Some("/root/.ssh/id_rsa".into()),
            last_connected: None,
            color: None,
            tags: None,
            proxy_type: None,
            proxy_host: None,
            proxy_port: None,
            favorite: None,
        };

        let json = serde_json::to_string(&conn).expect("serialize minimal");
        let back: ConnectionConfig = serde_json::from_str(&json).expect("deserialize minimal");
        assert_eq!(back.id, "min");
        assert_eq!(back.key_path.as_deref(), Some("/root/.ssh/id_rsa"));
        assert!(back.color.is_none());
        assert!(back.tags.is_none());
        assert!(back.favorite.is_none());
    }

    #[test]
    fn test_connection_config_vec_serialize() {
        let conns = vec![
            ConnectionConfig {
                id: "a".into(), name: "A".into(), host: "1.1.1.1".into(),
                port: 22, username: "root".into(), auth_type: "password".into(),
                group: "G1".into(), icon: None, key_path: None, last_connected: None,
                color: None, tags: None, proxy_type: None, proxy_host: None,
                proxy_port: None, favorite: None,
            },
            ConnectionConfig {
                id: "b".into(), name: "B".into(), host: "2.2.2.2".into(),
                port: 2222, username: "admin".into(), auth_type: "key".into(),
                group: "G2".into(), icon: None, key_path: Some("/key".into()),
                last_connected: None, color: None, tags: None, proxy_type: None,
                proxy_host: None, proxy_port: None, favorite: None,
            },
        ];
        let json = serde_json::to_string_pretty(&conns).expect("serialize vec");
        let back: Vec<ConnectionConfig> = serde_json::from_str(&json).expect("deserialize vec");
        assert_eq!(back.len(), 2);
        assert_eq!(back[0].id, "a");
        assert_eq!(back[1].port, 2222);
    }

    #[test]
    fn test_recording_meta_serialize() {
        let meta = RecordingMeta {
            id: "rec-001".into(),
            name: "Session 1".into(),
            connection_name: "My Server".into(),
            started_at: "2024-01-15T10:00:00Z".into(),
            duration_secs: 3600,
            file_path: "rec-001.cast".into(),
            tags: vec!["demo".into()],
        };
        let json = serde_json::to_string(&meta).expect("serialize RecordingMeta");
        let back: RecordingMeta = serde_json::from_str(&json).expect("deserialize RecordingMeta");
        assert_eq!(back.id, "rec-001");
        assert_eq!(back.duration_secs, 3600);
        assert_eq!(back.tags, vec!["demo"]);
    }
}
