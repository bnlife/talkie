use std::path::PathBuf;

use crate::error::AppError;
use crate::models::Settings;

/// Load settings from a JSON configuration file.
///
/// If the file does not exist, returns default settings without error.
pub fn load(path: PathBuf) -> Result<Settings, AppError> {
    if !path.exists() {
        return Ok(Settings::default());
    }
    let content = std::fs::read_to_string(&path)?;
    let settings: Settings = serde_json::from_str(&content)?;
    Ok(settings)
}

/// Save settings to a JSON configuration file.
///
/// Creates parent directories if they do not exist.
pub fn save(path: PathBuf, settings: &Settings) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(settings)?;
    std::fs::write(&path, content)?;
    Ok(())
}
