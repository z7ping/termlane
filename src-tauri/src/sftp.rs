// sftp.rs - Real SFTP file operations via ssh2

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

/// Map of session_id → underlying ssh2::Session
/// We need access to the session to open SFTP subsystem
/// Reuse the same sessions from ssh.rs
use crate::ssh;

// ─── Local file operations ───

pub fn list_local(path: &str) -> Result<Vec<FileEntry>, String> {
    use std::fs;
    use std::path::Path;
    use std::time::SystemTime;

    let dir = Path::new(path);
    if !dir.is_dir() {
        return Err(format!("不是有效目录: {}", path));
    }

    let mut entries = Vec::new();

    // Parent directory entry
    if let Some(parent) = dir.parent() {
        entries.push(FileEntry {
            name: "..".into(),
            path: parent.to_string_lossy().into(),
            size: 0,
            is_dir: true,
            modified: None,
            permissions: Some("drwxr-xr-x".into()),
        });
    }

    for entry in fs::read_dir(dir).map_err(|e| e.to_string())?.flatten() {
        let name = entry.file_name().to_string_lossy().into();
        let path = entry.path().to_string_lossy().into();
        let meta = entry.metadata().ok();
        let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
        let is_dir = meta.as_ref().map(|m| m.is_dir()).unwrap_or(false);
        let modified = meta
            .as_ref()
            .and_then(|m| m.modified().ok())
            .map(|t| {
                let dur = t
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default();
                let secs = dur.as_secs();
                format_timestamp(secs)
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

    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(entries)
}

fn format_timestamp(secs: u64) -> String {
    // Simple timestamp formatting (not timezone-aware)
    let mins = secs / 60;
    let hours = mins / 60;
    let days = hours / 24;
    let years = days / 365;
    let remaining_days = days % 365;
    let months = remaining_days / 30;
    let day = (remaining_days % 30) + 1;
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}",
        1970 + years,
        months + 1,
        day,
        hours % 24,
        mins % 60
    )
}

// ─── Remote SFTP operations (via ssh2) ───

/// List remote directory using SFTP subsystem
/// Note: We can't directly access the Session from ssh.rs due to ownership.
/// For now, we use SSH exec to list directory (works without SFTP subsystem).
pub fn list_remote(session_id: &str, path: &str) -> Result<Vec<FileEntry>, String> {
    // Use ssh exec to list directory with detailed format
    // This approach works when SFTP subsystem might not be available
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    let output = rt.block_on(ssh::execute(
        session_id,
        &format!(
            "ls -la --time-style='+%Y-%m-%d %H:%M' {} 2>/dev/null || ls -la {}",
            escape_path(path),
            escape_path(path)
        ),
    ))?;

    parse_ls_output(&output, path)
}

fn escape_path(path: &str) -> String {
    // Shell-escape the path
    format!("'{}'", path.replace('\'', "'\\''"))
}

fn parse_ls_output(output: &str, base_path: &str) -> Result<Vec<FileEntry>, String> {
    let mut entries = Vec::new();

    // Add parent directory
    if base_path != "/" {
        let parent = std::path::Path::new(base_path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "/".to_string());
        entries.push(FileEntry {
            name: "..".into(),
            path: parent,
            size: 0,
            is_dir: true,
            modified: None,
            permissions: Some("drwxr-xr-x".into()),
        });
    }

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("total ") {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 7 {
            continue;
        }

        let permissions = parts[0].to_string();
        let is_dir = permissions.starts_with('d');
        let size: u64 = parts[4].parse().unwrap_or(0);

        // Date and time (parts[5] = date, parts[6] = time)
        let modified = if parts.len() >= 7 {
            Some(format!("{} {}", parts[5], parts[6]))
        } else {
            None
        };

        // Filename is the rest after date+time+size
        let name_start = if parts.len() >= 8 { 7 } else { 6 };
        let name = parts[name_start..].join(" ");

        // Handle symlinks: "name -> target"
        let clean_name = if let Some(arrow_pos) = name.find(" -> ") {
            name[..arrow_pos].to_string()
        } else {
            name
        };

        if clean_name == "." || clean_name == ".." {
            continue;
        }

        let path = if base_path == "/" {
            format!("/{}", clean_name)
        } else {
            format!("{}/{}", base_path, clean_name)
        };

        entries.push(FileEntry {
            name: clean_name,
            path,
            size,
            is_dir,
            modified,
            permissions: Some(permissions),
        });
    }

    // Sort: directories first, then alphabetical
    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    if entries.is_empty() {
        // Parsing might have failed, return at least parent
        return Err("无法解析目录列表".to_string());
    }

    Ok(entries)
}

pub fn upload(session_id: &str, local: &str, remote: &str) -> Result<String, String> {
    // Use scp via exec
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    let local_content = std::fs::read(local).map_err(|e| format!("读取本地文件失败: {}", e))?;

    // Write content via SSH: cat > remote_path
    let cmd = format!(
        "cat > {} << 'XTERMINAL_EOF'\n{}\nXTERMINAL_EOF",
        escape_path(remote),
        String::from_utf8_lossy(&local_content)
    );
    rt.block_on(ssh::execute(session_id, &cmd))?;

    Ok(format!("已上传: {} → {}", local, remote))
}

pub fn download(session_id: &str, remote: &str, local: &str) -> Result<String, String> {
    // Read remote file via SSH cat
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    let output = rt.block_on(ssh::execute(session_id, &format!("cat {}", escape_path(remote))))?;

    // Remove the "[exit: X]" suffix that execute() adds
    let content = if let Some(pos) = output.rfind("\n[exit:") {
        &output[..pos]
    } else {
        &output
    };

    std::fs::write(local, content).map_err(|e| format!("写入本地文件失败: {}", e))?;

    Ok(format!("已下载: {} → {}", remote, local))
}

pub fn rename_file(session_id: &str, old_path: &str, new_path: &str) -> Result<String, String> {
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    rt.block_on(ssh::execute(
        session_id,
        &format!("mv {} {}", escape_path(old_path), escape_path(new_path)),
    ))?;
    Ok(format!("已重命名: {} → {}", old_path, new_path))
}

pub fn delete_file(session_id: &str, path: &str, is_dir: bool) -> Result<String, String> {
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    let cmd = if is_dir {
        format!("rm -rf {}", escape_path(path))
    } else {
        format!("rm {}", escape_path(path))
    };
    rt.block_on(ssh::execute(session_id, &cmd))?;
    Ok(format!("已删除: {}", path))
}

pub fn create_dir(session_id: &str, path: &str) -> Result<String, String> {
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    rt.block_on(ssh::execute(
        session_id,
        &format!("mkdir -p {}", escape_path(path)),
    ))?;
    Ok(format!("已创建目录: {}", path))
}

pub fn chmod(session_id: &str, path: &str, mode: &str) -> Result<String, String> {
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    rt.block_on(ssh::execute(
        session_id,
        &format!("chmod {} {}", mode, escape_path(path)),
    ))?;
    Ok(format!("已修改权限: {} → {}", path, mode))
}

pub fn read_file(session_id: &str, path: &str) -> Result<String, String> {
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    let output =
        rt.block_on(ssh::execute(session_id, &format!("cat {}", escape_path(path))))?;

    // Remove the "[exit: X]" suffix
    let content = if let Some(pos) = output.rfind("\n[exit:") {
        &output[..pos]
    } else {
        &output
    };

    Ok(content.to_string())
}

pub fn write_file(session_id: &str, path: &str, content: &str) -> Result<String, String> {
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;

    // Use heredoc to write content
    let cmd = format!(
        "cat > {} << 'XTERMINAL_EOF'\n{}\nXTERMINAL_EOF",
        escape_path(path),
        content
    );
    rt.block_on(ssh::execute(session_id, &cmd))?;
    Ok(format!("已保存: {}", path))
}
