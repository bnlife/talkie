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
    log::info!("Rust::commands::settings::update_settings | 更新配置 | model={}", settings.model);
    {
        let mut config = state.config.lock().map_err(|e| e.to_string())?;
        *config = settings.clone();
    }
    config::save(state.config_path.clone(), &settings).map_err(|e| e.to_string())
}

/// Verify LLM API connectivity by sending a POST request to the chat completions endpoint.
///
/// This is a pure function (no Tauri dependency) so it can be tested directly.
pub async fn verify_connection(base_url: &str, api_key: &str, model: &str) -> Result<String, String> {
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

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
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

/// Test the LLM API connection with the given settings.
#[tauri::command]
pub async fn test_connection(
    settings: models::Settings,
    _state: State<'_, AppState>,
) -> Result<String, String> {
    log::info!("Rust::commands::settings::test_connection | 测试连接 | url={}", settings.base_url);
    verify_connection(&settings.base_url, &settings.api_key, &settings.model).await
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
