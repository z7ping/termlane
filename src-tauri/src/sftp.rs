// sftp.rs - SFTP file operations

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
    pub modified: Option<String>,
    pub permissions: Option<String>,
}

pub fn list_local(path: &str) -> Result<Vec<FileEntry>, String> {
    let dir = Path::new(path);
    if !dir.is_dir() { return Err(format!("不是有效目录: {}", path)); }
    let mut entries = Vec::new();
    if let Some(parent) = dir.parent() {
        entries.push(FileEntry { name: "..".into(), path: parent.to_string_lossy().into(), size: 0, is_dir: true, modified: None, permissions: Some("drwxr-xr-x".into()) });
    }
    for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())?.flatten() {
        let name = entry.file_name().to_string_lossy().into();
        let path = entry.path().to_string_lossy().into();
        let meta = entry.metadata().ok();
        let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
        let is_dir = meta.as_ref().map(|m| m.is_dir()).unwrap_or(false);
        let modified = meta.as_ref().and_then(|m| m.modified().ok()).map(|t| {
            let secs = t.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs();
            format!("{:04}-{:02}-{:02} {:02}:{:02}", 1970 + secs/31536000, (secs%31536000)/2592000 + 1, (secs%2592000)/86400 + 1, (secs%86400)/3600, (secs%3600)/60)
        });
        entries.push(FileEntry { name, path, size, is_dir, modified, permissions: None });
    }
    entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    Ok(entries)
}

pub fn list_remote(_session_id: &str, _path: &str) -> Result<Vec<FileEntry>, String> {
    Ok(vec![FileEntry { name: "..".into(), path: "/".into(), size: 0, is_dir: true, modified: None, permissions: Some("drwxr-xr-x".into()) }])
}

pub fn upload(_session_id: &str, local: &str, remote: &str) -> Result<String, String> {
    Ok(format!("上传: {} → {}", local, remote))
}

pub fn download(_session_id: &str, remote: &str, local: &str) -> Result<String, String> {
    Ok(format!("下载: {} → {}", remote, local))
}
