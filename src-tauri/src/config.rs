use std::path::PathBuf;

use crate::error::AppError;
use crate::models::Settings;

/// Load settings from a JSON configuration file.
///
/// If the file does not exist, returns default settings without error.
pub fn load(path: PathBuf) -> Result<Settings, AppError> {
    log::debug!("Rust::config::load | 加载配置 | path={:?}", path);
    if !path.exists() {
        return Ok(Settings::default());
    }
    let content = std::fs::read_to_string(&path).map_err(|e| {
        log::error!("Rust::config::load | 读取配置文件失败 | path={:?} error={}", path, e);
        AppError::ConfigError(e.to_string())
    })?;
    let settings: Settings = serde_json::from_str(&content).map_err(|e| {
        log::error!("Rust::config::load | 解析配置文件失败 | path={:?} error={}", path, e);
        AppError::ConfigError(e.to_string())
    })?;
    log::info!("Rust::config::load | 配置加载完成");
    Ok(settings)
}

/// Save settings to a JSON configuration file.
///
/// Creates parent directories if they do not exist.
pub fn save(path: PathBuf, settings: &Settings) -> Result<(), AppError> {
    log::info!("Rust::config::save | 保存配置 | path={:?}", path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            log::error!("Rust::config::save | 创建配置目录失败 | path={:?} error={}", path, e);
            AppError::ConfigError(e.to_string())
        })?;
    }
    let content = serde_json::to_string_pretty(settings).map_err(|e| {
        log::error!("Rust::config::save | 序列化配置失败 | error={}", e);
        AppError::ConfigError(e.to_string())
    })?;
    std::fs::write(&path, content).map_err(|e| {
        log::error!("Rust::config::save | 写入配置文件失败 | path={:?} error={}", path, e);
        AppError::ConfigError(e.to_string())
    })?;
    Ok(())
}
