use tauri::State;

use crate::config;
use crate::models;
use crate::AppState;

/// Return the current application settings.
#[tauri::command]
pub fn get_settings(
    state: State<'_, AppState>,
) -> Result<models::Settings, String> {
    log::debug!("RS::CMD::settings | read");
    let config = state.config.lock().map_err(|e| {
        log::error!("RS::ERR::E1001 | db_lock_fail");
        e.to_string()
    })?;
    Ok(config.clone())
}

/// Persist new settings to both memory and the config file.
#[tauri::command]
pub fn update_settings(
    settings: models::Settings,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!(
        "RS::CMD::settings | update | providers={} active={}",
        settings.providers.len(),
        settings.active_provider_id,
    );
    // Decrypt API keys before storing in memory (so LLM calls use plaintext)
    let mut decrypted = settings.clone();
    config::decrypt_provider_keys(&mut decrypted);
    {
        let mut config = state.config.lock().map_err(|e| {
            log::error!("RS::ERR::E1001 | db_lock_fail");
            e.to_string()
        })?;
        *config = decrypted;
    }
    // Save to disk with encrypted keys
    config::save(state.config_path.clone(), &settings).map_err(|e| {
        log::error!("RS::ERR::E6003 | config_write_fail | err={}", e);
        e.to_string()
    })
}

/// Verify LLM API connectivity by sending a POST request to the chat completions endpoint.
///
/// This is a pure function (no Tauri dependency) so it can be tested directly.
pub async fn verify_connection(client: &reqwest::Client, base_url: &str, api_key: &str, model: &str, headers: &std::collections::HashMap<String, String>) -> Result<String, String> {
    log::info!("RS::CMD::verify | base_url={} model={}", base_url, model);
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    let body = serde_json::json!({
        "model": model,
        "messages": [{"role": "user", "content": "test"}],
        "max_tokens": 1
    });

    let body_str =
        serde_json::to_string(&body).map_err(|e| {
            log::error!("RS::ERR::E3004 | body_serialize_fail | err={}", e);
            format!("serialize_body_fail: {}", e)
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
                log::error!("RS::ERR::E2001 | net_timeout | url={}", url);
                "net_timeout: check network or api address".to_string()
            } else if e.is_connect() {
                log::error!("RS::ERR::E2002 | net_connect_fail | url={}", url);
                format!("net_connect_fail: {}", e)
            } else {
                log::error!("RS::ERR::E2003 | net_request_fail | url={}", url);
                format!("net_request_fail: {}", e)
            }
        })?;

    let status = response.status();
    let body_text = response.text().await.unwrap_or_default();
    log::debug!("RS::CMD::verify | resp | status={} body_len={}", status, body_text.len());
    match status.as_u16() {
        200 => Ok("connect_ok".to_string()),
        401 => {
            log::error!("RS::ERR::E5001 | auth_fail | body_len={}", body_text.len());
            Err(format!("auth_fail: {}", body_text))
        }
        404 => {
            log::error!("RS::ERR::E5002 | model_not_found | body_len={}", body_text.len());
            Err(format!("addr_err: {}", body_text))
        }
        _ => {
            log::error!("RS::ERR::E2004 | http_error | status={} body_len={}", status, body_text.len());
            Err(format!("http_error {}: {}", status, body_text))
        }
    }
}

/// Test the LLM API connection with a single provider's configuration.
#[tauri::command]
pub async fn test_provider_connection(
    state: State<'_, AppState>,
    provider: models::ModelProvider,
) -> Result<String, String> {
    log::info!("RS::CMD::test | provider={} base_url={}", provider.name, provider.base_url);
    verify_connection(&state.http_client, &provider.base_url, &provider.api_key, provider.models.first().unwrap_or(&"gpt-3.5-turbo".to_string()), &provider.headers).await
}

/// Verify that a specific model is available on a provider by sending a test request.
#[tauri::command]
pub async fn verify_model(
    state: State<'_, AppState>,
    provider: models::ModelProvider,
    model: String,
) -> Result<String, String> {
    log::info!("RS::CMD::verify_model | provider={} model={}", provider.name, model);
    verify_connection(&state.http_client, &provider.base_url, &provider.api_key, &model, &provider.headers).await
}

/// Fetch available models from a provider's /v1/models endpoint.
#[tauri::command]
pub async fn fetch_provider_models(
    state: State<'_, AppState>,
    provider: models::ModelProvider,
) -> Result<Vec<String>, String> {
    log::info!("RS::CMD::models | provider={} base_url={}", provider.name, provider.base_url);

    let url = format!("{}/models", provider.base_url.trim_end_matches('/'));

    let mut request = state.http_client
        .get(&url)
        .header("Authorization", format!("Bearer {}", provider.api_key));

    for (key, value) in &provider.headers {
        request = request.header(key.as_str(), value.as_str());
    }

    let response = request.send().await.map_err(|e| {
        log::error!("RS::ERR::E2003 | net_request_fail | provider={}", provider.name);
        format!("net_request_fail: {}", e)
    })?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        log::error!("RS::ERR::E2004 | http_error | provider={} status={}", provider.name, status);
        return Err(format!("http_error {}: {}", status, body));
    }

    let body: serde_json::Value = response.json().await.map_err(|e| {
        log::error!("RS::ERR::E6002 | parse_fail | provider={}", provider.name);
        format!("parse_fail: {}", e)
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

    log::info!("RS::CMD::models | got {} models", models.len());
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
    log::info!("RS::CMD::open | url={}", url);
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
