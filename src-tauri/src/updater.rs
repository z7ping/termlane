use semver::Version;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub version: String,
    pub tag_name: String,
    pub body: String,
    pub download_url: Option<String>,
    pub published_at: String,
}

fn parse_version(value: &str) -> Result<Version, String> {
    Version::parse(value.trim().trim_start_matches('v'))
        .map_err(|e| format!("版本号无效 `{}`: {}", value, e))
}

fn is_newer_version(current: &str, latest: &str) -> Result<bool, String> {
    Ok(parse_version(latest)? > parse_version(current)?)
}

/// Check the latest public GitHub Release.
///
/// This command only performs version discovery. Download, signature
/// verification and installation belong to the signed Tauri updater chain and
/// are intentionally not implemented here.
#[tauri::command]
pub async fn check_update(current_version: String) -> Result<Option<UpdateInfo>, String> {
    const RELEASE_API_URL: &str = "https://api.github.com/repos/z7ping/termlane/releases/latest";

    let client = reqwest::Client::builder()
        .user_agent(format!("Termlane/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let response = client
        .get(RELEASE_API_URL)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await
        .map_err(|e| format!("请求 GitHub Releases 失败: {}", e))?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }

    if !response.status().is_success() {
        return Err(format!(
            "GitHub Releases 返回错误状态码: {}",
            response.status()
        ));
    }

    let release: GitHubRelease = response
        .json()
        .await
        .map_err(|e| format!("解析 GitHub Release 响应失败: {}", e))?;

    if !is_newer_version(&current_version, &release.tag_name)? {
        return Ok(None);
    }

    let latest = parse_version(&release.tag_name)?;
    Ok(Some(UpdateInfo {
        version: latest.to_string(),
        tag_name: release.tag_name,
        body: release.body.unwrap_or_else(|| "无更新说明".to_string()),
        download_url: release.html_url,
        published_at: release.published_at.unwrap_or_default(),
    }))
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    body: Option<String>,
    html_url: Option<String>,
    published_at: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_newer_release() {
        assert!(is_newer_version("0.1.0", "0.1.1").unwrap());
    }

    #[test]
    fn ignores_same_release() {
        assert!(!is_newer_version("0.1.0", "0.1.0").unwrap());
    }

    #[test]
    fn ignores_older_release() {
        assert!(!is_newer_version("0.2.0", "0.1.9").unwrap());
    }

    #[test]
    fn accepts_v_prefix() {
        assert!(is_newer_version("v0.1.0", "v0.2.0").unwrap());
    }

    #[test]
    fn follows_semver_prerelease_ordering() {
        assert!(is_newer_version("1.0.0-alpha.1", "1.0.0").unwrap());
        assert!(!is_newer_version("1.0.0", "1.0.0-rc.1").unwrap());
    }
}
