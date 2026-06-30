pub mod commands;
pub mod config;
pub mod error;
pub mod llm;
pub mod models;
pub mod store;

use std::path::PathBuf;

use tauri::Manager;
use tauri_plugin_log::{RotationStrategy, Target, TargetKind};
use tokio_util::sync::CancellationToken;

/// Global application state managed by Tauri.
pub struct AppState {
    pub db: std::sync::Mutex<rusqlite::Connection>,
    pub config: std::sync::Mutex<models::Settings>,
    pub config_path: PathBuf,
    /// Token used to cancel an in-flight streaming LLM response.
    /// `send_message` creates it; `stop_stream` takes it and calls `.cancel()`.
    pub cancel: std::sync::Mutex<Option<CancellationToken>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Debug)
                        .clear_targets()
                        .target(Target::new(TargetKind::Stdout))
                        .target(Target::new(TargetKind::LogDir {
                            file_name: Some("app".into()),
                        }))
                        .target(
                            Target::new(TargetKind::LogDir {
                                file_name: Some("app.error".into()),
                            })
                            .filter(|metadata| metadata.level() == log::Level::Error),
                        )
                        .max_file_size(2 * 1024 * 1024)
                        .rotation_strategy(RotationStrategy::KeepSome(10))
                        .build(),
                )?;
            }

            // Resolve standard data directory and ensure it exists.
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;

            // Initialise the SQLite database.
            let db_path = app_data_dir.join("chatbot.db");
            let db = store::init(&db_path)?;

            // Load persisted settings (falls back to defaults if absent).
            let config_path = app_data_dir.join("config.json");
            let settings = config::load(config_path.clone())?;

            // Inject state so Tauri commands can access it via State<>.
            app.manage(AppState {
                db: std::sync::Mutex::new(db),
                config: std::sync::Mutex::new(settings),
                config_path,
                cancel: std::sync::Mutex::new(None),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::chat::send_message,
            commands::chat::stop_stream,
            commands::chat::get_messages,
            commands::chat::delete_message,
            commands::chat::regenerate_message,
            commands::conversation::list_conversations,
            commands::conversation::create_conversation,
            commands::conversation::update_conversation,
            commands::conversation::delete_conversation,
            commands::conversation::pin_conversation,
            commands::conversation::unpin_conversation,
            commands::prompt::list_prompts,
            commands::prompt::create_prompt,
            commands::prompt::update_prompt,
            commands::prompt::delete_prompt,
            commands::prompt::set_default_prompt,
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::settings::test_connection,
            commands::settings::log_message,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
