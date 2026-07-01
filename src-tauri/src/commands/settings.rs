use tauri::State;

use crate::config;
use crate::models;
use crate::AppState;

/// Return the current application settings.
#[tauri::command]
pub fn get_settings(
    state: State<'_, AppState>,
) -> Result<models::Settings, String> {
    log::debug!("Rust::commands::settings::get_settings | 读取配置");
    let config = state.config.lock().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

/// Persist new settings to both memory and the config file.
#[tauri::command]
pub fn update_settings(
    settings: models::Settings,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!(
        "Rust::commands::settings::update_settings | 更新配置 | providers={} active={}",
        settings.providers.len(),
        settings.active_provider_id,
    );
    {
        let mut config = state.config.lock().map_err(|e| e.to_string())?;
        *config = settings.clone();
    }
    config::save(state.config_path.clone(), &settings).map_err(|e| e.to_string())
}

/// Verify LLM API connectivity by sending a POST request to the chat completions endpoint.
///
/// This is a pure function (no Tauri dependency) so it can be tested directly.
pub async fn verify_connection(base_url: &str, api_key: &str, model: &str, headers: &std::collections::HashMap<String, String>) -> Result<String, String> {
    log::info!("Rust::commands::settings::verify_connection | 验证连接 | base_url={} model={}", base_url, model);
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| {
            log::error!("Rust::commands::settings::verify_connection | 创建 HTTP 客户端失败: {}", e);
            format!("创建 HTTP 客户端失败: {}", e)
        })?;

    let body = serde_json::json!({
        "model": model,
        "messages": [{"role": "user", "content": "test"}],
        "max_tokens": 1
    });

    let body_str =
        serde_json::to_string(&body).map_err(|e| {
            log::error!("Rust::commands::settings::verify_connection | 序列化请求体失败: {}", e);
            format!("序列化请求体失败: {}", e)
        })?;

    let mut request = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json");

    for (key, value) in headers {
        request = request.header(key, value);
    }

    let response = request
        .body(body_str)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                log::error!("Rust::commands::settings::verify_connection | 连接超时，请检查网络或 API 地址");
                "连接超时，请检查网络或 API 地址".to_string()
            } else if e.is_connect() {
                log::error!("Rust::commands::settings::verify_connection | 无法连接到服务器，请检查 API 地址: {}", e);
                format!("无法连接到服务器，请检查 API 地址: {}", e)
            } else {
                log::error!("Rust::commands::settings::verify_connection | 网络请求失败: {}", e);
                format!("网络请求失败: {}", e)
            }
        })?;

    let status = response.status();
    let body_text = response.text().await.unwrap_or_default();
    log::debug!("Rust::commands::settings::verify_connection | API 响应 | status={} body={}", status, body_text);
    match status.as_u16() {
        200 => Ok("连接成功".to_string()),
        401 => {
            log::error!("Rust::commands::settings::verify_connection | API Key 认证失败 | body={}", body_text);
            Err(format!("API Key 认证失败: {}", body_text))
        }
        404 => {
            log::error!("Rust::commands::settings::verify_connection | API 地址错误 | body={}", body_text);
            Err(format!("API 地址错误: {}", body_text))
        }
        _ => {
            log::error!("Rust::commands::settings::verify_connection | 服务器返回异常 | status={} body={}", status, body_text);
            Err(format!("服务器返回异常状态码 {}: {}", status, body_text))
        }
    }
}

/// Test the LLM API connection with a single provider's configuration.
#[tauri::command]
pub async fn test_provider_connection(
    provider: models::ModelProvider,
) -> Result<String, String> {
    log::info!("Rust::commands::settings::test_provider_connection | 测试连接 | provider={} base_url={}", provider.name, provider.base_url);
    verify_connection(&provider.base_url, &provider.api_key, provider.models.first().unwrap_or(&"gpt-3.5-turbo".to_string()), &provider.headers).await
}

/// Fetch available models from a provider's /v1/models endpoint.
#[tauri::command]
pub async fn fetch_provider_models(
    provider: models::ModelProvider,
) -> Result<Vec<String>, String> {
    log::info!("Rust::commands::settings::fetch_provider_models | 拉取模型列表 | provider={} base_url={}", provider.name, provider.base_url);

    let url = format!("{}/models", provider.base_url.trim_end_matches('/'));

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| {
            log::error!("Rust::commands::settings::fetch_provider_models | 创建 HTTP 客户端失败: {}", e);
            format!("创建 HTTP 客户端失败: {}", e)
        })?;

    let mut request = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", provider.api_key));

    for (key, value) in &provider.headers {
        request = request.header(key.as_str(), value.as_str());
    }

    let response = request.send().await.map_err(|e| {
        log::error!("Rust::commands::settings::fetch_provider_models | 请求失败: {}", e);
        format!("请求失败: {}", e)
    })?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        log::error!("Rust::commands::settings::fetch_provider_models | HTTP 错误 | status={} body={}", status, body);
        return Err(format!("HTTP error {}: {}", status, body));
    }

    let body: serde_json::Value = response.json().await.map_err(|e| {
        log::error!("Rust::commands::settings::fetch_provider_models | 解析响应失败: {}", e);
        format!("解析响应失败: {}", e)
    })?;

    let models: Vec<String> = body
        .get("data")
        .and_then(|d| d.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m.get("id").and_then(|id| id.as_str()).map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    log::info!("Rust::commands::settings::fetch_provider_models | 获取到 {} 个模型", models.len());
    Ok(models)
}

/// Forward a log message from the frontend to the backend logging system.
#[tauri::command]
pub fn log_message(level: String, message: String) {
    match level.as_str() {
        "info" => log::info!("{}", message),
        "warn" => log::warn!("{}", message),
        "error" => log::error!("{}", message),
        _ => log::debug!("{}", message),
    }
}

/// Open a URL in the system default browser.
#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    log::info!("Rust::commands::settings::open_url | 打开链接 | url={}", url);
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &url])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&url)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
