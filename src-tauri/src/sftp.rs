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

/// List local directory contents
pub fn list_local(path: &str) -> Result<Vec<FileEntry>, String> {
    let dir = Path::new(path);
    if !dir.is_dir() {
        return Err(format!("不是有效目录: {}", path));
    }

    let mut entries = Vec::new();

    // Add parent directory
    if let Some(parent) = dir.parent() {
        entries.push(FileEntry {
            name: "..".to_string(),
            path: parent.to_string_lossy().to_string(),
            size: 0,
            is_dir: true,
            modified: None,
            permissions: Some("drwxr-xr-x".to_string()),
        });
    }

    let read_dir = std::fs::read_dir(dir).map_err(|e| e.to_string())?;

    for entry in read_dir.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path().to_string_lossy().to_string();
        let metadata = entry.metadata().ok();

        let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
        let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
        let modified = metadata
            .as_ref()
            .and_then(|m| m.modified().ok())
            .map(|t| {
                let dur = t.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
                format_timestamp(dur.as_secs())
            });

        entries.push(FileEntry {
            name,
            path,
            size,
            is_dir,
            modified,
            permissions: None,
        });
    }

    // Sort: directories first, then by name
    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(entries)
}

/// Format Unix timestamp to readable string
fn format_timestamp(secs: u64) -> String {
    // Simple date formatting without chrono
    let total_days = secs / 86400;
    let _year = 1970 + total_days / 365;
    let day_of_year = total_days % 365;
    let month = day_of_year / 30 + 1;
    let day = day_of_year % 30 + 1;
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;
    format!("{:04}-{:02}-{:02} {:02}:{:02}", _year, month, day, hours, minutes)
}
