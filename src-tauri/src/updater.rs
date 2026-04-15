use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub version: String,
    pub tag_name: String,
    pub body: String,
    pub download_url: Option<String>,
    pub published_at: String,
}

/// Check for updates from Gitea API
/// Returns Ok(Some(update)) if update available, Ok(None) if no update or no release, Err on failure
#[tauri::command]
pub async fn check_update(current_version: String) -> Result<Option<UpdateInfo>, String> {
    const GITEA_API_URL: &str = "https://gitea.7ping.site/api/v1/repos/ai-area/xterminal-pro/releases/latest";

    let client = reqwest::Client::builder()
        .user_agent("XTerminal-Pro/0.1.0")
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let response = client
        .get(GITEA_API_URL)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("请求 Gitea API 失败: {}", e))?;

    // 404 表示没有发布任何 release，静默返回无更新
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

    let latest_version = release
        .tag_name
        .strip_prefix('v')
        .unwrap_or(&release.tag_name)
        .to_string();

    // 版本比较：如果有新版本则返回更新信息
    if latest_version != current_version {
        Ok(Some(UpdateInfo {
            version: latest_version.clone(),
            tag_name: release.tag_name,
            body: release.body.unwrap_or_else(|| "无更新说明".to_string()),
            download_url: release.html_url,
            published_at: release.published_at,
        }))
    } else {
        Ok(None)
    }
}

// Gitea API Release 响应结构
#[derive(Debug, Deserialize)]
struct GiteaRelease {
    tag_name: String,
    name: Option<String>,
    body: Option<String>,
    html_url: Option<String>,
    published_at: String,
    prerelease: bool,
}
