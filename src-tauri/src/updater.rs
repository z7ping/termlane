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

/// Check for updates from Gitea API.
/// Returns Ok(Some(update)) only when the latest release is newer than the
/// current app version. Returns Ok(None) when there is no release or the app is
/// already on an equal/newer version.
#[tauri::command]
pub async fn check_update(current_version: String) -> Result<Option<UpdateInfo>, String> {
    const GITEA_API_URL: &str = "https://gitea.7ping.site/api/v1/repos/ai-area/xterminal-pro/releases/latest";

    let current = Version::parse(current_version.trim_start_matches('v'))
        .map_err(|e| format!("当前版本号无效: {}", e))?;

    let client = reqwest::Client::builder()
        .user_agent(format!("XTerminal-Pro/{}", env!("CARGO_PKG_VERSION")))
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

    let latest_version_text = release.tag_name.trim_start_matches('v');
    let latest = Version::parse(latest_version_text)
        .map_err(|e| format!("Release 版本号无效: {}", e))?;

    if latest <= current {
        return Ok(None);
    }

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
