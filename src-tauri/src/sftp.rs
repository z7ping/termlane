use crate::utils::*;
// sftp.rs - Remote file operations via SSH exec with base64 encoding
// All remote file operations use SSH exec commands with base64 encoding
// to safely handle binary files across different shell environments.

use serde::{Deserialize, Serialize};

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

use crate::ssh;
use crate::utils;

// ─── Shared tokio runtime (avoid creating new runtime per operation) ───
static RUNTIME: std::sync::LazyLock<tokio::runtime::Runtime> =
    std::sync::LazyLock::new(|| tokio::runtime::Runtime::new().expect("Failed to create tokio runtime"));

fn split_exec_output(output: &str) -> Result<(String, i32), String> {
    let marker = output
        .rfind("\n[exit:")
        .ok_or_else(|| "远程命令返回格式异常：缺少退出状态".to_string())?;
    let status_text = output[marker + 1..]
        .strip_prefix("[exit: ")
        .and_then(|value| value.strip_suffix(']'))
        .ok_or_else(|| "远程命令返回格式异常：退出状态无效".to_string())?;
    let status = status_text
        .parse::<i32>()
        .map_err(|_| "远程命令返回格式异常：退出状态不是数字".to_string())?;
    Ok((output[..marker].trim_end_matches('\n').to_string(), status))
}

fn execute_remote_checked(session_id: &str, command: &str) -> Result<String, String> {
    let output = RUNTIME.block_on(ssh::execute(session_id, command))?;
    let (body, exit_status) = split_exec_output(&output)?;
    if exit_status != 0 {
        let detail = body.trim();
        return Err(if detail.is_empty() {
            format!("远程命令执行失败 (exit {})", exit_status)
        } else {
            format!("远程命令执行失败 (exit {}): {}", exit_status, detail)
        });
    }
    Ok(body)
}

// ─── Local file operations ───

/// Validate a local filesystem path, rejecting sensitive directories and traversal.
fn validate_local_path(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("路径不能为空".to_string());
    }

    // Canonicalize to resolve symlinks and `..` components
    let canonical = std::fs::canonicalize(path)
        .map_err(|e| format!("无法解析路径: {}", e))?;
    let canonical_str = canonical.to_string_lossy();

    // Reject access to sensitive system directories
    let forbidden_prefixes = ["/etc", "/proc", "/sys", "/dev", "/boot", "/root"];
    for prefix in &forbidden_prefixes {
        if canonical_str == *prefix || canonical_str.starts_with(&format!("{}/", prefix)) {
            return Err(format!("禁止访问系统目录: {}", prefix));
        }
    }

    Ok(())
}

pub fn list_local(path: &str) -> Result<Vec<FileEntry>, String> {
    validate_local_path(path)?;
    use std::fs;
    use std::path::Path;
    use std::time::SystemTime;

    let dir = Path::new(path);
    if !dir.is_dir() {
        return Err(format!("不是有效目录: {}", path));
    }

    let mut entries = Vec::new();

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
                let dur = t.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
                let secs = dur.as_secs();
                format_timestamp(secs)
            });

        entries.push(FileEntry {
            name, path, size, is_dir, modified,
            permissions: None,
        });
    }

    entries.sort_by(|a, b| {
        b.is_dir.cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(entries)
}

fn format_timestamp(secs: u64) -> String {
    // Convert Unix timestamp to date-time string (UTC, approximate)
    // Uses proper leap year calculation
    let total_days = secs / SECS_PER_DAY;
    let remaining_secs = secs % SECS_PER_DAY;
    let hours = remaining_secs / SECS_PER_HOUR;
    let minutes = (remaining_secs % SECS_PER_HOUR) / 60;

    // Calculate year with leap years
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

    // Calculate month
    let month_lengths = [
        if is_leap_year(year) { 29u64 } else { 28u64 },
        31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
    ];
    // Reorder: Jan=31, Feb=28/29, Mar=30, ...
    let month_days = [31u64, month_lengths[0], 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    let mut month = 1u64;
    let mut day_left = days_left;
    for &days_in_month in &month_days {
        if day_left < days_in_month {
            break;
        }
        day_left -= days_in_month;
        month += 1;
    }
    let day = day_left + 1;

    format!("{:04}-{:02}-{:02} {:02}:{:02}", year, month, day, hours, minutes)
}

fn is_leap_year(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

// ─── Path escaping ───

fn escape_path(path: &str) -> String {
    format!("'{}'", path.replace('\'', "'\\''"))
}

// ─── Remote file operations (via SSH exec + base64) ───

pub fn list_remote(session_id: &str, path: &str) -> Result<Vec<FileEntry>, String> {
    let output = execute_remote_checked(
        session_id,
        &format!(
            "ls -la --time-style='+%Y-%m-%d %H:%M' {} 2>/dev/null || ls -la {}",
            escape_path(path),
            escape_path(path)
        ),
    )?;
    parse_ls_output(&output, path)
}

fn parse_ls_output(output: &str, base_path: &str) -> Result<Vec<FileEntry>, String> {
    let mut entries = Vec::new();

    if base_path != "/" {
        let parent = std::path::Path::new(base_path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "/".to_string());
        entries.push(FileEntry {
            name: "..".into(), path: parent, size: 0, is_dir: true,
            modified: None, permissions: Some("drwxr-xr-x".into()),
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

        let modified = if parts.len() >= 7 {
            Some(format!("{} {}", parts[5], parts[6]))
        } else {
            None
        };

        let name_start = if parts.len() >= 8 { 7 } else { 6 };
        let name = parts[name_start..].join(" ");

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
            name: clean_name, path, size, is_dir, modified,
            permissions: Some(permissions),
        });
    }

    entries.sort_by(|a, b| {
        b.is_dir.cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(entries)
}

/// Upload a local file to remote via base64 encoding (safe for binary files).
/// Uses temp file + chunked printf to avoid shell ARG_MAX limits.
pub fn upload(session_id: &str, local: &str, remote: &str) -> Result<String, String> {
    validate_local_path(local)?;

    let local_content = std::fs::read(local).map_err(|e| format!("读取本地文件失败: {}", e))?;
    let encoded = base64_encode(&local_content);
    let remote_tmp = format!("/tmp/xterminal_upload_{}.b64", utils::unix_now());

    // Write base64 in chunks via printf to avoid argument length limits
    let chunk_size = SFTP_CHUNK_SIZE;
    let chunks: Vec<&str> = encoded.as_bytes()
        .chunks(chunk_size)
        .filter_map(|c| std::str::from_utf8(c).ok())
        .collect();

    if let Some(first) = chunks.first() {
        let cmd = format!("printf '%s' {} > {}", escape_path(first), escape_path(&remote_tmp));
        execute_remote_checked(session_id, &cmd)?;
    }
    for chunk in chunks.iter().skip(1) {
        let cmd = format!("printf '%s' {} >> {}", escape_path(chunk), escape_path(&remote_tmp));
        execute_remote_checked(session_id, &cmd)?;
    }

    // Decode base64 temp file to final destination and clean up
    let cmd = format!(
        "base64 -d {} > {} && rm -f {}",
        escape_path(&remote_tmp), escape_path(remote), escape_path(&remote_tmp)
    );
    execute_remote_checked(session_id, &cmd)?;

    Ok(format!("已上传: {} → {} ({} bytes)", local, remote, local_content.len()))
}

/// Download a remote file via base64 encoding (safe for binary files).
pub fn download(session_id: &str, remote: &str, local: &str) -> Result<String, String> {
    let cmd = format!("base64 -w 0 {}", escape_path(remote));
    let encoded = execute_remote_checked(session_id, &cmd)?;
    let encoded: String = encoded.chars().filter(|c| !c.is_whitespace()).collect();

    let decoded = base64_decode(&encoded)?;
    std::fs::write(local, &decoded).map_err(|e| format!("写入本地文件失败: {}", e))?;

    Ok(format!("已下载: {} → {} ({} bytes)", remote, local, decoded.len()))
}

pub fn rename_file(session_id: &str, old_path: &str, new_path: &str) -> Result<String, String> {
    execute_remote_checked(
        session_id,
        &format!("mv {} {}", escape_path(old_path), escape_path(new_path)),
    )?;
    Ok(format!("已重命名: {} → {}", old_path, new_path))
}

pub fn delete_file(session_id: &str, path: &str, is_dir: bool) -> Result<String, String> {
    let cmd = if is_dir {
        format!("rm -rf {}", escape_path(path))
    } else {
        format!("rm {}", escape_path(path))
    };
    execute_remote_checked(session_id, &cmd)?;
    Ok(format!("已删除: {}", path))
}

pub fn create_dir(session_id: &str, path: &str) -> Result<String, String> {
    execute_remote_checked(
        session_id,
        &format!("mkdir -p {}", escape_path(path)),
    )?;
    Ok(format!("已创建目录: {}", path))
}

pub fn chmod(session_id: &str, path: &str, mode: &str) -> Result<String, String> {
    // Validate mode to prevent command injection
    static OCTAL_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    static SYMBOLIC_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let is_valid_octal = OCTAL_RE.get_or_init(|| regex::Regex::new(r"^0?[0-7]{3,4}$").unwrap()).is_match(mode);
    let is_valid_symbolic = SYMBOLIC_RE.get_or_init(|| regex::Regex::new(r"^[ugoa]*[+-=][rwxXst]*([,][ugoa]*[+-=][rwxXst]*)*$").unwrap()).is_match(mode);
    if !is_valid_octal && !is_valid_symbolic {
        return Err(format!("无效的权限模式: {}", mode));
    }

    execute_remote_checked(
        session_id,
        &format!("chmod {} {}", mode, escape_path(path)),
    )?;
    Ok(format!("已修改权限: {} → {}", path, mode))
}

/// Read a remote file as UTF-8 text via base64 (safe for any encoding).
pub fn read_file(session_id: &str, path: &str) -> Result<String, String> {
    let cmd = format!("base64 -w 0 {}", escape_path(path));
    let encoded = execute_remote_checked(session_id, &cmd)?;
    let encoded: String = encoded.chars().filter(|c| !c.is_whitespace()).collect();

    let decoded = base64_decode(&encoded)?;
    String::from_utf8(decoded).map_err(|e| format!("文件不是有效的 UTF-8 文本: {}", e))
}

/// Write UTF-8 text to a remote file via base64 (safe for any content).
pub fn write_file(session_id: &str, path: &str, content: &str) -> Result<String, String> {
    let encoded = base64_encode(content.as_bytes());
    let remote_tmp = format!("/tmp/xterminal_edit_{}.b64", utils::unix_now());

    let chunk_size = SFTP_CHUNK_SIZE;
    let chunks: Vec<&str> = encoded.as_bytes()
        .chunks(chunk_size)
        .filter_map(|c| std::str::from_utf8(c).ok())
        .collect();

    if let Some(first) = chunks.first() {
        let cmd = format!("printf '%s' {} > {}", escape_path(first), escape_path(&remote_tmp));
        execute_remote_checked(session_id, &cmd)?;
    }
    for chunk in chunks.iter().skip(1) {
        let cmd = format!("printf '%s' {} >> {}", escape_path(chunk), escape_path(&remote_tmp));
        execute_remote_checked(session_id, &cmd)?;
    }

    let cmd = format!(
        "base64 -d {} > {} && rm -f {}",
        escape_path(&remote_tmp), escape_path(path), escape_path(&remote_tmp)
    );
    execute_remote_checked(session_id, &cmd)?;
    Ok(format!("已保存: {} ({} bytes)", path, content.len()))
}

// ─── Base64 helpers (no external crate needed) ───

const B64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(input: &[u8]) -> String {
    let mut result = String::with_capacity((input.len() + 2) / 3 * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(B64_CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(B64_CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(B64_CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(B64_CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    let table: [u8; 256] = {
        let mut t = [255u8; 256];
        let mut i = 0u8;
        for &c in B64_CHARS {
            t[c as usize] = i;
            i += 1;
        }
        t[b'=' as usize] = 0;
        t
    };

    let clean: Vec<u8> = input.bytes().filter(|&b| b != b'\n' && b != b'\r' && b != b' ').collect();
    if clean.len() % 4 != 0 {
        return Err("无效的 base64 编码".to_string());
    }

    let mut result = Vec::with_capacity(clean.len() / 4 * 3);
    for chunk in clean.chunks(4) {
        let a = table[chunk[0] as usize];
        let b = table[chunk[1] as usize];
        let c = table[chunk[2] as usize];
        let d = table[chunk[3] as usize];
        if a == 255 || b == 255 || c == 255 || d == 255 {
            return Err("无效的 base64 字符".to_string());
        }
        result.push(((a << 2) | (b >> 4)) as u8);
        if chunk[2] != b'=' {
            result.push((((b & 0x0F) << 4) | (c >> 2)) as u8);
        }
        if chunk[3] != b'=' {
            result.push((((c & 0x03) << 6) | d) as u8);
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_exec_output_success() {
        let (body, exit) = split_exec_output("hello\n[exit: 0]").expect("valid exec output");
        assert_eq!(body, "hello");
        assert_eq!(exit, 0);
    }

    #[test]
    fn test_split_exec_output_failure_status() {
        let (_, exit) = split_exec_output("\n[exit: 23]").expect("valid exec output");
        assert_eq!(exit, 23);
    }

    #[test]
    fn test_escape_path_simple() {
        assert_eq!(escape_path("/home/user"), "'/home/user'");
    }

    #[test]
    fn test_escape_path_with_single_quote() {
        let escaped = escape_path("/path/to/file'name");
        assert_eq!(escaped, "'/path/to/file'\\''name'");
    }

    #[test]
    fn test_escape_path_with_spaces() {
        assert_eq!(escape_path("/path/to/my file"), "'/path/to/my file'");
    }

    #[test]
    fn test_parse_ls_output() {
        let output = "total 24\n\
drwxr-xr-x 2 user group  4096 2024-01-15 10:30 src\n\
-rw-r--r-- 1 user group 12345 2024-01-15 09:00 readme.txt\n\
lrwxrwxrwx 1 user group     5 2024-01-15 08:00 link -> target\n";
        let entries = parse_ls_output(output, "/home/user").expect("parse_ls_output");
        assert!(entries.len() >= 3, "Expected at least 3 entries, got {}", entries.len());
        assert_eq!(entries[0].name, "..");
        let file = entries.iter().find(|e| e.name == "readme.txt").expect("readme.txt");
        assert!(!file.is_dir);
        assert_eq!(file.size, 12345);
        let dir = entries.iter().find(|e| e.name == "src").expect("src");
        assert!(dir.is_dir);
        let link = entries.iter().find(|e| e.name == "link").expect("link");
        assert_eq!(link.path, "/home/user/link");
    }

    #[test]
    fn test_parse_ls_output_root_dir() {
        let output = "total 8\ndrwxr-xr-x 2 root root 4096 2024-01-01 00:00 etc\n";
        let entries = parse_ls_output(output, "/").expect("parse root dir");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "etc");
    }

    #[test]
    fn test_parse_ls_output_empty_root_dir() {
        let entries = parse_ls_output("total 0\n", "/").expect("empty root should be valid");
        assert!(entries.is_empty());
    }

    #[test]
    fn test_parse_ls_output_skips_dot_entries() {
        let output = "total 4\ndrwxr-xr-x 2 root root 4096 2024-01-01 00:00 .\ndrwxr-xr-x 2 root root 4096 2024-01-01 00:00 ..\n-rw-r--r-- 1 root root 0 2024-01-01 00:00 file.txt\n";
        let entries = parse_ls_output(output, "/home").expect("parse ls");
        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        assert!(!names.iter().any(|n| *n == "."), "Should skip .");
        let parent = entries.iter().find(|e| e.name == "..").unwrap();
        assert_eq!(parent.path, "/");
    }

    #[test]
    fn test_format_timestamp() {
        // 1609459200 = 2021-01-01 00:00:00 UTC
        let ts = format_timestamp(1609459200);
        assert_eq!(ts, "2021-01-01 00:00", "Got: {}", ts);

        // 1640995200 = 2022-01-01 00:00:00 UTC
        let ts2 = format_timestamp(1640995200);
        assert_eq!(ts2, "2022-01-01 00:00", "Got: {}", ts2);

        // 946684800 = 2000-01-01 00:00:00 UTC (leap year boundary)
        let ts3 = format_timestamp(946684800);
        assert_eq!(ts3, "2000-01-01 00:00", "Got: {}", ts3);
    }

    #[test]
    fn test_base64_encode_simple() {
        assert_eq!(base64_encode(b"Hello"), "SGVsbG8=");
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"a"), "YQ==");
        assert_eq!(base64_encode(b"ab"), "YWI=");
        assert_eq!(base64_encode(b"abc"), "YWJj");
    }

    #[test]
    fn test_base64_decode_simple() {
        assert_eq!(base64_decode("SGVsbG8=").unwrap(), b"Hello");
        assert_eq!(base64_decode("").unwrap(), b"");
        assert_eq!(base64_decode("YQ==").unwrap(), b"a");
        assert_eq!(base64_decode("YWI=").unwrap(), b"ab");
        assert_eq!(base64_decode("YWJj").unwrap(), b"abc");
    }

    #[test]
    fn test_base64_roundtrip_binary() {
        let data: Vec<u8> = (0..=255).collect();
        let encoded = base64_encode(&data);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn test_base64_roundtrip_text() {
        let text = "Hello World";
        let encoded = base64_encode(text.as_bytes());
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(text.as_bytes(), decoded.as_slice());
    }
}
