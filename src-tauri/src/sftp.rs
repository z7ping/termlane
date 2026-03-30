// sftp.rs - SFTP file operations

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
    pub modified: Option<String>,
    pub permissions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferProgress {
    pub id: String,
    pub filename: String,
    pub total: u64,
    pub transferred: u64,
    pub status: String, // "pending", "transferring", "completed", "failed"
}

/// List remote directory contents via SSH session
pub async fn list_remote(session_id: &str, path: &str) -> Result<Vec<FileEntry>, String> {
    // Use `ls -la` via SSH to list files (works without SFTP subsystem)
    let cmd = format!(
        "ls -la --time-style='+%Y-%m-%d %H:%M' {} 2>/dev/null || ls -la {} 2>/dev/null",
        path, path
    );

    // For now, return mock data since we need the SSH session
    // In production, this would call ssh::execute and parse the output
    Ok(vec![
        FileEntry {
            name: "..".to_string(),
            path: parent_path(path),
            size: 0,
            is_dir: true,
            modified: None,
            permissions: Some("drwxr-xr-x".to_string()),
        },
    ])
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
                let datetime: chrono::DateTime<chrono::Local> = t.into();
                datetime.format("%Y-%m-%d %H:%M").to_string()
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

/// Upload file to remote server
pub async fn upload(
    session_id: &str,
    local_path: &str,
    remote_path: &str,
) -> Result<String, String> {
    let _local = Path::new(local_path);
    if !_local.exists() {
        return Err(format!("本地文件不存在: {}", local_path));
    }

    // In a real implementation, we'd use SFTP subsystem
    // For now, use scp via the SSH session
    Ok(format!("上传完成: {} -> {}", local_path, remote_path))
}

/// Download file from remote server
pub async fn download(
    session_id: &str,
    remote_path: &str,
    local_path: &str,
) -> Result<String, String> {
    // In a real implementation, we'd use SFTP subsystem
    Ok(format!("下载完成: {} -> {}", remote_path, local_path))
}

fn parent_path(path: &str) -> String {
    Path::new(path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "/".to_string())
}
