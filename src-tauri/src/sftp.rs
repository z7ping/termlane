// sftp.rs - SFTP file operations

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
    pub modified: Option<String>,
    pub permissions: Option<String>,
}

// SFTP operations will be implemented when the SSH session provides SFTP
// For now, this is a placeholder module

pub fn list_files_placeholder() -> Vec<FileEntry> {
    vec![
        FileEntry {
            name: "..".into(),
            path: "/".into(),
            size: 0,
            is_dir: true,
            modified: None,
            permissions: Some("drwxr-xr-x".into()),
        },
    ]
}
