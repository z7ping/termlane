use crate::ssh;
use crate::utils::*;
use serde::{Deserialize, Serialize};
use ssh2::{ErrorCode, FileStat, FileType, Sftp};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

// sftp.rs - Remote file management backed by the SSH SFTP subsystem.
// No remote shell commands, base64 staging files, or GNU userland assumptions.

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
    pub modified: Option<String>,
    pub permissions: Option<String>,
}

fn is_eagain(error: &ssh2::Error) -> bool {
    matches!(error.code(), ErrorCode::Session(-37))
}

fn retry_sftp<T>(
    operation_name: &str,
    mut operation: impl FnMut() -> Result<T, ssh2::Error>,
) -> Result<T, String> {
    let started = std::time::Instant::now();
    loop {
        match operation() {
            Ok(value) => return Ok(value),
            Err(error)
                if is_eagain(&error)
                    && started.elapsed() < std::time::Duration::from_secs(READ_TIMEOUT_SECS) =>
            {
                std::thread::sleep(std::time::Duration::from_millis(PTY_POLL_FAST_MS));
            }
            Err(error) if is_eagain(&error) => {
                return Err(format!("{}超时", operation_name));
            }
            Err(error) => return Err(format!("{}失败: {}", operation_name, error)),
        }
    }
}

fn retry_io<T>(
    operation_name: &str,
    mut operation: impl FnMut() -> std::io::Result<T>,
) -> Result<T, String> {
    let started = std::time::Instant::now();
    loop {
        match operation() {
            Ok(value) => return Ok(value),
            Err(error)
                if (error.kind() == std::io::ErrorKind::WouldBlock
                    || error.kind() == std::io::ErrorKind::TimedOut)
                    && started.elapsed() < std::time::Duration::from_secs(READ_TIMEOUT_SECS) =>
            {
                std::thread::sleep(std::time::Duration::from_millis(PTY_POLL_FAST_MS));
            }
            Err(error)
                if error.kind() == std::io::ErrorKind::WouldBlock
                    || error.kind() == std::io::ErrorKind::TimedOut =>
            {
                return Err(format!("{}超时", operation_name));
            }
            Err(error) => return Err(format!("{}失败: {}", operation_name, error)),
        }
    }
}

fn open_sftp(session_id: &str) -> Result<Sftp, String> {
    ssh::open_sftp(session_id)
}

// ─── Local file operations ───

fn validate_local_path(path: &str) -> Result<(), String> {
    if path.trim().is_empty() {
        return Err("路径不能为空".to_string());
    }
    std::fs::canonicalize(path)
        .map(|_| ())
        .map_err(|error| format!("无法访问本地路径: {}", error))
}

pub fn list_local(path: &str) -> Result<Vec<FileEntry>, String> {
    validate_local_path(path)?;
    let dir = Path::new(path);
    if !dir.is_dir() {
        return Err(format!("不是有效目录: {}", path));
    }

    let mut entries = Vec::new();
    if let Some(parent) = dir.parent() {
        entries.push(FileEntry {
            name: "..".into(),
            path: parent.to_string_lossy().into_owned(),
            size: 0,
            is_dir: true,
            modified: None,
            permissions: None,
        });
    }

    for entry in std::fs::read_dir(dir).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let metadata = entry.metadata().map_err(|error| error.to_string())?;
        let modified = metadata
            .modified()
            .ok()
            .and_then(|value| value.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|value| format_timestamp(value.as_secs()));

        entries.push(FileEntry {
            name: entry.file_name().to_string_lossy().into_owned(),
            path: entry.path().to_string_lossy().into_owned(),
            size: metadata.len(),
            is_dir: metadata.is_dir(),
            modified,
            permissions: None,
        });
    }

    sort_entries(&mut entries);
    Ok(entries)
}

// ─── SFTP remote file operations ───

pub fn list_remote(session_id: &str, path: &str) -> Result<Vec<FileEntry>, String> {
    let sftp = open_sftp(session_id)?;
    let base = Path::new(path);
    let remote_entries = retry_sftp("读取远程目录", || sftp.readdir(base))?;
    let mut entries = Vec::with_capacity(remote_entries.len() + 1);

    if path != "/" {
        entries.push(FileEntry {
            name: "..".into(),
            path: remote_parent(path),
            size: 0,
            is_dir: true,
            modified: None,
            permissions: None,
        });
    }

    for (remote_path, stat) in remote_entries {
        let Some(name) = remote_path.file_name().map(|value| value.to_string_lossy().into_owned()) else {
            continue;
        };
        if name == "." || name == ".." {
            continue;
        }

        entries.push(FileEntry {
            name,
            path: remote_path.to_string_lossy().into_owned(),
            size: stat.size.unwrap_or(0),
            is_dir: stat.is_dir(),
            modified: stat.mtime.map(format_timestamp),
            permissions: format_permissions(&stat),
        });
    }

    sort_entries(&mut entries);
    Ok(entries)
}

pub fn upload(session_id: &str, local: &str, remote: &str) -> Result<String, String> {
    validate_local_path(local)?;
    let metadata = std::fs::metadata(local).map_err(|error| format!("读取本地文件失败: {}", error))?;
    if !metadata.is_file() {
        return Err("当前上传仅支持文件".to_string());
    }

    let sftp = open_sftp(session_id)?;
    let existing_perm = remote_permissions(&sftp, remote);
    let mut source = std::fs::File::open(local).map_err(|error| format!("打开本地文件失败: {}", error))?;
    let temp_path = remote_temp_path(remote);
    let result = (|| {
        let mut target = retry_sftp("创建远程临时文件", || sftp.create(Path::new(&temp_path)))?;
        copy_local_to_remote(&mut source, &mut target)?;
        drop(target);
        apply_remote_permissions(&sftp, &temp_path, existing_perm)?;
        replace_remote_file(&sftp, &temp_path, remote)
    })();

    if result.is_err() {
        let _ = retry_sftp("清理远程临时文件", || sftp.unlink(Path::new(&temp_path)));
    }
    result?;

    Ok(format!("已上传: {} → {} ({} bytes)", local, remote, metadata.len()))
}

pub fn download(session_id: &str, remote: &str, local: &str) -> Result<String, String> {
    let sftp = open_sftp(session_id)?;
    let mut source = retry_sftp("打开远程文件", || sftp.open(Path::new(remote)))?;
    let original_permissions = std::fs::metadata(local).ok().map(|metadata| metadata.permissions());
    let temp_path = local_temp_path(local);
    let result = (|| {
        let mut target = std::fs::File::create(&temp_path)
            .map_err(|error| format!("创建本地临时文件失败: {}", error))?;
        let copied = copy_remote_to_local(&mut source, &mut target)?;
        drop(target);
        if let Some(permissions) = original_permissions {
            std::fs::set_permissions(&temp_path, permissions)
                .map_err(|error| format!("保留本地文件权限失败: {}", error))?;
        }
        replace_local_file(&temp_path, Path::new(local))?;
        Ok(copied)
    })();

    if result.is_err() {
        let _ = std::fs::remove_file(&temp_path);
    }
    let copied = result?;
    Ok(format!("已下载: {} → {} ({} bytes)", remote, local, copied))
}

pub fn rename_file(session_id: &str, old_path: &str, new_path: &str) -> Result<String, String> {
    let sftp = open_sftp(session_id)?;
    retry_sftp("重命名远程文件", || {
        sftp.rename(Path::new(old_path), Path::new(new_path), None)
    })?;
    Ok(format!("已重命名: {} → {}", old_path, new_path))
}

pub fn delete_file(session_id: &str, path: &str, is_dir: bool) -> Result<String, String> {
    let sftp = open_sftp(session_id)?;
    remove_remote_path(&sftp, Path::new(path), is_dir)?;
    Ok(format!("已删除: {}", path))
}

pub fn create_dir(session_id: &str, path: &str) -> Result<String, String> {
    let sftp = open_sftp(session_id)?;
    retry_sftp("创建远程目录", || sftp.mkdir(Path::new(path), 0o755))?;
    Ok(format!("已创建目录: {}", path))
}

pub fn chmod(session_id: &str, path: &str, mode: &str) -> Result<String, String> {
    let mode = parse_octal_mode(mode)?;
    let sftp = open_sftp(session_id)?;
    apply_remote_permissions(&sftp, path, Some(mode))?;
    Ok(format!("已修改权限: {} → {:o}", path, mode))
}

pub fn read_file(session_id: &str, path: &str) -> Result<String, String> {
    let sftp = open_sftp(session_id)?;
    let stat = retry_sftp("读取远程文件信息", || sftp.stat(Path::new(path)))?;
    if let Some(size) = stat.size {
        if size > MAX_INLINE_EDIT_BYTES as u64 {
            return Err(format!(
                "文件过大：{} bytes，在线编辑最大允许 {} bytes",
                size, MAX_INLINE_EDIT_BYTES
            ));
        }
    }

    let mut file = retry_sftp("打开远程文件", || sftp.open(Path::new(path)))?;
    let mut content = Vec::with_capacity(stat.size.unwrap_or(0).min(MAX_INLINE_EDIT_BYTES as u64) as usize);
    read_remote_limited(&mut file, &mut content, MAX_INLINE_EDIT_BYTES)?;
    String::from_utf8(content).map_err(|error| format!("文件不是有效的 UTF-8 文本: {}", error))
}

pub fn write_file(session_id: &str, path: &str, content: &str) -> Result<String, String> {
    if content.len() > MAX_INLINE_EDIT_BYTES {
        return Err(format!(
            "文件内容过大：{} bytes，在线编辑最大允许 {} bytes",
            content.len(), MAX_INLINE_EDIT_BYTES
        ));
    }

    let sftp = open_sftp(session_id)?;
    let existing_perm = remote_permissions(&sftp, path);
    let temp_path = remote_temp_path(path);
    let result = (|| {
        let mut file = retry_sftp("创建远程临时文件", || sftp.create(Path::new(&temp_path)))?;
        write_remote_all(&mut file, content.as_bytes())?;
        retry_io("刷新远程文件", || file.flush())?;
        drop(file);
        apply_remote_permissions(&sftp, &temp_path, existing_perm)?;
        replace_remote_file(&sftp, &temp_path, path)
    })();

    if result.is_err() {
        let _ = retry_sftp("清理远程临时文件", || sftp.unlink(Path::new(&temp_path)));
    }
    result?;
    Ok(format!("已保存: {} ({} bytes)", path, content.len()))
}

fn remote_permissions(sftp: &Sftp, path: &str) -> Option<u32> {
    retry_sftp("读取远程文件权限", || sftp.stat(Path::new(path)))
        .ok()
        .and_then(|stat| stat.perm)
}

fn apply_remote_permissions(sftp: &Sftp, path: &str, permissions: Option<u32>) -> Result<(), String> {
    let Some(permissions) = permissions else {
        return Ok(());
    };
    let stat = FileStat {
        size: None,
        uid: None,
        gid: None,
        perm: Some(permissions),
        atime: None,
        mtime: None,
    };
    retry_sftp("设置远程文件权限", || sftp.setstat(Path::new(path), stat.clone()))
}

fn replace_remote_file(sftp: &Sftp, temp: &str, target: &str) -> Result<(), String> {
    retry_sftp("提交远程文件", || {
        sftp.rename(Path::new(temp), Path::new(target), None)
    })
}

fn replace_local_file(temp: &Path, target: &Path) -> Result<(), String> {
    // std::fs::rename uses replacement semantics for an existing file target.
    // Do not remove the destination first: if the rename fails, the original
    // file must remain intact.
    std::fs::rename(temp, target).map_err(|error| format!("提交本地文件失败: {}", error))
}

fn remote_temp_path(target: &str) -> String {
    format!("{}.termlane-{}.tmp", target, unique_suffix())
}

fn local_temp_path(target: &str) -> PathBuf {
    let path = Path::new(target);
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "download".to_string());
    let temp_name = format!(".{}.termlane-{}.tmp", file_name, unique_suffix());
    path.parent().unwrap_or_else(|| Path::new(".")).join(temp_name)
}

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}

fn copy_local_to_remote(source: &mut std::fs::File, target: &mut ssh2::File) -> Result<u64, String> {
    let mut buffer = [0u8; PTY_BUF_SIZE];
    let mut total = 0u64;
    loop {
        let read = source
            .read(&mut buffer)
            .map_err(|error| format!("读取本地文件失败: {}", error))?;
        if read == 0 {
            break;
        }
        write_remote_all(target, &buffer[..read])?;
        total += read as u64;
    }
    retry_io("刷新远程文件", || target.flush())?;
    Ok(total)
}

fn copy_remote_to_local(source: &mut ssh2::File, target: &mut std::fs::File) -> Result<u64, String> {
    let mut buffer = [0u8; PTY_BUF_SIZE];
    let mut total = 0u64;
    loop {
        let read = retry_io("读取远程文件", || source.read(&mut buffer))?;
        if read == 0 {
            break;
        }
        target
            .write_all(&buffer[..read])
            .map_err(|error| format!("写入本地文件失败: {}", error))?;
        total += read as u64;
    }
    target.flush().map_err(|error| format!("刷新本地文件失败: {}", error))?;
    Ok(total)
}

fn read_remote_limited(source: &mut ssh2::File, output: &mut Vec<u8>, limit: usize) -> Result<(), String> {
    let mut buffer = [0u8; PTY_BUF_SIZE];
    loop {
        let read = retry_io("读取远程文件", || source.read(&mut buffer))?;
        if read == 0 {
            return Ok(());
        }
        if output.len() + read > limit {
            return Err(format!("文件超过在线编辑上限 {} bytes", limit));
        }
        output.extend_from_slice(&buffer[..read]);
    }
}

fn write_remote_all(target: &mut ssh2::File, data: &[u8]) -> Result<(), String> {
    let mut written = 0;
    while written < data.len() {
        let count = retry_io("写入远程文件", || target.write(&data[written..]))?;
        if count == 0 {
            return Err("写入远程文件失败：未写入任何数据".to_string());
        }
        written += count;
    }
    Ok(())
}

fn remove_remote_path(sftp: &Sftp, path: &Path, is_dir: bool) -> Result<(), String> {
    if !is_dir {
        return retry_sftp("删除远程文件", || sftp.unlink(path));
    }

    let children = retry_sftp("读取待删除目录", || sftp.readdir(path))?;
    for (child_path, stat) in children {
        match stat.file_type() {
            FileType::Directory => remove_remote_path(sftp, &child_path, true)?,
            _ => retry_sftp("删除远程文件", || sftp.unlink(&child_path))?,
        }
    }
    retry_sftp("删除远程目录", || sftp.rmdir(path))
}

fn parse_octal_mode(mode: &str) -> Result<u32, String> {
    let trimmed = mode.trim().trim_start_matches('0');
    let normalized = if trimmed.is_empty() { "0" } else { trimmed };
    let value = u32::from_str_radix(normalized, 8)
        .map_err(|_| format!("无效的权限模式: {}，请输入如 755 或 0644", mode))?;
    if value > 0o7777 {
        return Err(format!("无效的权限模式: {}", mode));
    }
    Ok(value)
}

fn remote_parent(path: &str) -> String {
    let trimmed = path.trim_end_matches('/');
    if trimmed.is_empty() || trimmed == "/" {
        return "/".to_string();
    }
    match trimmed.rsplit_once('/') {
        Some(("", _)) | None => "/".to_string(),
        Some((parent, _)) => parent.to_string(),
    }
}

fn sort_entries(entries: &mut [FileEntry]) {
    entries.sort_by(|left, right| {
        if left.name == ".." {
            return std::cmp::Ordering::Less;
        }
        if right.name == ".." {
            return std::cmp::Ordering::Greater;
        }
        right
            .is_dir
            .cmp(&left.is_dir)
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
    });
}

fn format_permissions(stat: &FileStat) -> Option<String> {
    let mode = stat.perm?;
    let file_type = match stat.file_type() {
        FileType::Directory => 'd',
        FileType::Symlink => 'l',
        FileType::RegularFile => '-',
        FileType::NamedPipe => 'p',
        FileType::Socket => 's',
        FileType::BlockDevice => 'b',
        FileType::CharDevice => 'c',
        FileType::Other(_) => '?',
    };
    let masks = [0o400, 0o200, 0o100, 0o040, 0o020, 0o010, 0o004, 0o002, 0o001];
    let symbols = ['r', 'w', 'x', 'r', 'w', 'x', 'r', 'w', 'x'];
    let mut result = String::with_capacity(10);
    result.push(file_type);
    for (mask, symbol) in masks.into_iter().zip(symbols) {
        result.push(if mode & mask != 0 { symbol } else { '-' });
    }
    Some(result)
}

fn format_timestamp(secs: u64) -> String {
    let total_days = secs / SECS_PER_DAY;
    let remaining_secs = secs % SECS_PER_DAY;
    let hours = remaining_secs / SECS_PER_HOUR;
    let minutes = (remaining_secs % SECS_PER_HOUR) / 60;

    let mut year = 1970u64;
    let mut days_left = total_days;
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if days_left < days_in_year {
            break;
        }
        days_left -= days_in_year;
        year += 1;
    }

    let month_days = [
        31u64,
        if is_leap_year(year) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 1u64;
    let mut day_left = days_left;
    for days_in_month in month_days {
        if day_left < days_in_month {
            break;
        }
        day_left -= days_in_month;
        month += 1;
    }

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}",
        year,
        month,
        day_left + 1,
        hours,
        minutes
    )
}

fn is_leap_year(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parent_path_is_remote_posix_style() {
        assert_eq!(remote_parent("/var/www/html"), "/var/www");
        assert_eq!(remote_parent("/home"), "/");
        assert_eq!(remote_parent("/"), "/");
    }

    #[test]
    fn octal_mode_validation() {
        assert_eq!(parse_octal_mode("755").unwrap(), 0o755);
        assert_eq!(parse_octal_mode("0644").unwrap(), 0o644);
        assert!(parse_octal_mode("u+x").is_err());
        assert!(parse_octal_mode("888").is_err());
    }

    #[test]
    fn permission_format_is_unix_like() {
        let stat = FileStat {
            size: None,
            uid: None,
            gid: None,
            perm: Some(0o100644),
            atime: None,
            mtime: None,
        };
        assert_eq!(format_permissions(&stat).as_deref(), Some("-rw-r--r--"));
    }

    #[test]
    fn temp_paths_stay_near_targets() {
        let remote = remote_temp_path("/var/www/app.txt");
        assert!(remote.starts_with("/var/www/app.txt.termlane-"));
        assert!(remote.ends_with(".tmp"));

        let local = local_temp_path("/tmp/app.txt");
        assert_eq!(local.parent(), Some(Path::new("/tmp")));
        assert!(local.file_name().unwrap().to_string_lossy().starts_with(".app.txt.termlane-"));
    }

    #[test]
    fn local_replace_overwrites_without_predeleting_target() {
        let base = std::env::temp_dir().join(format!("termlane-replace-{}", unique_suffix()));
        std::fs::create_dir_all(&base).unwrap();
        let target = base.join("target.txt");
        let temp = base.join("temp.txt");
        std::fs::write(&target, b"old").unwrap();
        std::fs::write(&temp, b"new").unwrap();

        replace_local_file(&temp, &target).unwrap();

        assert_eq!(std::fs::read(&target).unwrap(), b"new");
        assert!(!temp.exists());
        std::fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn timestamp_format_keeps_existing_contract() {
        assert_eq!(format_timestamp(1_609_459_200), "2021-01-01 00:00");
    }
}