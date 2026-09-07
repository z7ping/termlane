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

/// Check for updates from Gitea API.
/// Returns Ok(Some(update)) only when the latest release is newer than the
/// current app version. Returns Ok(None) when there is no release or the app is
/// already on an equal/newer version.
#[tauri::command]
pub async fn check_update(current_version: String) -> Result<Option<UpdateInfo>, String> {
    // The Gitea repository still uses its historical slug. Change this URL only
    // after that remote repository is renamed as well.
    const GITEA_API_URL: &str = "https://gitea.7ping.site/api/v1/repos/ai-area/xterminal-pro/releases/latest";

    let client = reqwest::Client::builder()
        .user_agent(format!("Termlane/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let response = client
        .get(GITEA_API_URL)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("请求 Gitea API 失败: {}", e))?;

    if response.status() == 404 {
        return Ok(None);
    }

    if !response.status().is_success() {
        return Err(format!("Gitea API 返回错误状态码: {}", response.status()));
    }

    let release: GiteaRelease = response
        .json()
        .await
        .map_err(|e| format!("解析 Gitea API 响应失败: {}", e))?;

    if !is_newer_version(&current_version, &release.tag_name)? {
        return Ok(None);
    }

    let latest = parse_version(&release.tag_name)?;
    Ok(Some(UpdateInfo {
        version: latest.to_string(),
        tag_name: release.tag_name,
        body: release.body.unwrap_or_else(|| "无更新说明".to_string()),
        download_url: release.html_url,
        published_at: release.published_at,
    }))
}

#[derive(Debug, Deserialize)]
struct GiteaRelease {
    tag_name: String,
    body: Option<String>,
    html_url: Option<String>,
    published_at: String,
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
