use std::path::PathBuf;

use crate::error::AppError;
use crate::models::{ModelProvider, Settings};

/// Load settings from a JSON configuration file.
///
/// If the file does not exist, returns default settings without error.
/// If the file uses the old format (no `providers` field), automatically
/// migrates it to the new multi-provider format.
pub fn load(path: PathBuf) -> Result<Settings, AppError> {
    log::debug!("RS::config::load | path={:?}", path);
    if !path.exists() {
        return Ok(Settings::default());
    }
    let content = std::fs::read_to_string(&path).map_err(|e| {
        log::error!("RS::config::load | read fail | path={:?} err={}", path, e);
        AppError::ConfigError(e.to_string())
    })?;
    let raw: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
        log::error!("RS::config::load | parse fail | path={:?} err={}", path, e);
        AppError::ConfigError(e.to_string())
    })?;

    // Detect old format: has base_url/api_key but no providers
    if raw.get("providers").is_none() && raw.get("base_url").is_some() {
        log::info!("RS::config::load | migrating old format");
        let settings = migrate_old_format(&raw, &path)?;
        return Ok(settings);
    }

    let settings: Settings = serde_json::from_value(raw).map_err(|e| {
        log::error!("RS::config::load | parse fail | path={:?} err={}", path, e);
        AppError::ConfigError(e.to_string())
    })?;
    log::info!("RS::config::load | ok | providers={}", settings.providers.len());
    Ok(settings)
}

/// Migrate old flat config format to new multi-provider format.
fn migrate_old_format(raw: &serde_json::Value, path: &PathBuf) -> Result<Settings, AppError> {
    let base_url = raw.get("base_url").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let api_key = raw.get("api_key").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let model = raw.get("model").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let temperature = raw.get("temperature").and_then(|v| v.as_f64()).unwrap_or(0.7) as f32;
    let top_p = raw.get("top_p").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
    let last_active = raw.get("last_active_conversation_id").and_then(|v| v.as_str()).map(|s| s.to_string());

    let provider_id = uuid::Uuid::new_v4().to_string();
    let provider = ModelProvider {
        id: provider_id.clone(),
        name: "默认 Provider".to_string(),
        icon: Some("Bot".to_string()),
        base_url,
        api_key,
        headers: std::collections::HashMap::new(),
        models: if model.is_empty() { vec![] } else { vec![model] },
        enabled: true,
    };

    let settings = Settings {
        providers: vec![provider],
        active_provider_id: provider_id,
        temperature,
        top_p,
        last_active_conversation_id: last_active,
        dark_mode: false,
    };

    // Save migrated config
    if let Err(e) = save(path.clone(), &settings) {
        log::warn!("RS::config::migrate | save fail | err={}", e);
    } else {
        log::info!("RS::config::migrate | ok");
    }

    Ok(settings)
}

/// Save settings to a JSON configuration file.
///
/// Creates parent directories if they do not exist.
pub fn save(path: PathBuf, settings: &Settings) -> Result<(), AppError> {
    log::info!("RS::config::save | path={:?}", path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            log::error!("RS::config::save | mkdir fail | path={:?} err={}", path, e);
            AppError::ConfigError(e.to_string())
        })?;
    }
    let content = serde_json::to_string_pretty(settings).map_err(|e| {
        log::error!("RS::config::save | serialize fail | err={}", e);
        AppError::ConfigError(e.to_string())
    })?;
    std::fs::write(&path, content).map_err(|e| {
        log::error!("RS::config::save | write fail | path={:?} err={}", path, e);
        AppError::ConfigError(e.to_string())
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_path(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("talkie_config_test");
        std::fs::create_dir_all(&dir).unwrap();
        dir.join(name)
    }

    #[test]
    fn migrate_old_config_to_providers() {
        let path = temp_path("migrate_old.json");
        let old = serde_json::json!({
            "base_url": "https://api.deepseek.com/v1",
            "api_key": "sk-test",
            "model": "deepseek-chat",
            "temperature": 0.8
        });
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "{}", old).unwrap();

        let settings = load(path.clone()).unwrap();

        assert_eq!(settings.providers.len(), 1);
        assert_eq!(settings.providers[0].name, "默认 Provider");
        assert_eq!(settings.providers[0].base_url, "https://api.deepseek.com/v1");
        assert_eq!(settings.providers[0].api_key, "sk-test");
        assert_eq!(settings.providers[0].models, vec!["deepseek-chat"]);
        assert_eq!(settings.active_provider_id, settings.providers[0].id);
        assert!((settings.temperature - 0.8).abs() < 0.01);

        // cleanup
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn load_new_config_preserves_providers() {
        let path = temp_path("new_format.json");
        let new = serde_json::json!({
            "providers": [
                {
                    "id": "p1",
                    "name": "OpenAI",
                    "base_url": "https://api.openai.com/v1",
                    "api_key": "sk-abc",
                    "headers": {},
                    "models": ["gpt-4o"],
                    "enabled": true
                }
            ],
            "active_provider_id": "p1",
            "temperature": 0.5
        });
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "{}", new).unwrap();

        let settings = load(path.clone()).unwrap();

        assert_eq!(settings.providers.len(), 1);
        assert_eq!(settings.providers[0].id, "p1");
        assert_eq!(settings.providers[0].name, "OpenAI");
        assert_eq!(settings.active_provider_id, "p1");

        // cleanup
        let _ = std::fs::remove_file(&path);
    }
}
