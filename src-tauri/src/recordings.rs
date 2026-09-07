use std::fs;
use std::path::{Component, Path};

const MAX_RECORDING_BYTES: usize = 32 * 1024 * 1024;

fn recording_path(filename: &str) -> Result<std::path::PathBuf, String> {
    let trimmed = filename.trim();
    if trimmed.is_empty() {
        return Err("录制文件名不能为空".into());
    }

    let path = Path::new(trimmed);
    if path.components().count() != 1
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err("录制文件名不能包含目录".into());
    }

    if path.extension().and_then(|value| value.to_str()) != Some("cast") {
        return Err("录制文件必须使用 .cast 扩展名".into());
    }

    Ok(crate::config::get_recording_dir().join(path))
}

fn validate_content(content: &str) -> Result<(), String> {
    if content.len() > MAX_RECORDING_BYTES {
        return Err(format!(
            "录制文件过大：{} bytes，最大允许 {} bytes",
            content.len(),
            MAX_RECORDING_BYTES
        ));
    }
    Ok(())
}

#[tauri::command]
pub fn save_recording(
    filename: String,
    content: String,
    meta: crate::config::RecordingMeta,
) -> Result<(), String> {
    validate_content(&content)?;
    if meta.file_path != filename {
        return Err("录制元数据中的文件名与实际文件名不一致".into());
    }

    let path = recording_path(&filename)?;
    fs::write(&path, content).map_err(|error| format!("保存录制文件失败: {}", error))?;

    if let Err(error) = crate::config::save_recording_meta(meta) {
        let _ = fs::remove_file(&path);
        return Err(format!("保存录制元数据失败: {}", error));
    }

    Ok(())
}

#[tauri::command]
pub fn read_recording_file(filename: String) -> Result<String, String> {
    let path = recording_path(&filename)?;
    fs::read_to_string(path).map_err(|error| format!("读取录制文件失败: {}", error))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_recording_path_traversal() {
        assert!(recording_path("../session.cast").is_err());
        assert!(recording_path("nested/session.cast").is_err());
        assert!(recording_path("session.txt").is_err());
    }

    #[test]
    fn accepts_plain_cast_filename() {
        let path = recording_path("session.cast").expect("valid recording filename");
        assert_eq!(
            path.file_name().and_then(|value| value.to_str()),
            Some("session.cast")
        );
    }

    #[test]
    fn rejects_oversized_recording() {
        let content = "x".repeat(MAX_RECORDING_BYTES + 1);
        assert!(validate_content(&content).is_err());
    }
}
