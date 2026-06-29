use tauri::State;

use crate::config;
use crate::models;
use crate::AppState;

/// Return the current application settings.
#[tauri::command]
pub fn get_settings(
    state: State<'_, AppState>,
) -> Result<models::Settings, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

/// Persist new settings to both memory and the config file.
#[tauri::command]
pub fn update_settings(
    new_settings: models::Settings,
    state: State<'_, AppState>,
) -> Result<(), String> {
    {
        let mut config = state.config.lock().map_err(|e| e.to_string())?;
        *config = new_settings.clone();
    }
    config::save(state.config_path.clone(), &new_settings).map_err(|e| e.to_string())
}

/// Test the LLM API connection with the given parameters.
///
/// Currently a placeholder that always reports success.
#[tauri::command]
pub fn test_connection(
    base_url: String,
    api_key: String,
    model: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let _ = (base_url, api_key, model, state);
    // TODO: implement actual API reachability test
    Ok("连接成功".to_string())
}
