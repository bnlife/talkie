pub mod chat;
pub mod commands;
pub mod config;
pub mod crypto;
pub mod error;
pub mod llm;
pub mod mcp;
pub mod models;
pub mod store;

use std::path::PathBuf;
use std::sync::Arc;

use crate::error::AppError;
use tauri::Manager;
use tauri_plugin_log::{RotationStrategy, Target, TargetKind};
use tokio_util::sync::CancellationToken;

/// Global application state managed by Tauri.
pub struct AppState {
    /// Shared HTTP client for all outbound requests (LLM, provider, MCP HTTP).
    /// Reusing a single client enables TCP connection pooling and reduces latency.
    pub http_client: reqwest::Client,
    pub db: std::sync::Mutex<rusqlite::Connection>,
    pub config: std::sync::Mutex<models::Settings>,
    pub config_path: PathBuf,
    /// Token used to cancel an in-flight streaming LLM response.
    /// `send_message` creates it; `stop_stream` takes it and calls `.cancel()`.
    pub cancel: std::sync::Mutex<Option<CancellationToken>>,
    /// Pool of running MCP server processes.
    pub mcp_pool: Arc<mcp::pool::McpPool>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_wdio_webdriver::init())
        .setup(|app| {
            // Boot log: session anchor
            let session_id = uuid::Uuid::new_v4().to_string();
            let ver = env!("CARGO_PKG_VERSION");
            let plat = std::env::consts::OS;
            log::info!("RS::boot | session={} ver={} plat={}", session_id, ver, plat);

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
                        .format(|out, message, record| {
                            out.finish(format_args!(
                                "[{}][{}] {}",
                                chrono::Local::now().format("%m-%d %H:%M:%S"),
                                record.level(),
                                message
                            ))
                        })
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

            // Copy bundled MCP server scripts to app data dir (always overwrite)
            let mcp_servers_dir = app_data_dir.join("mcp-servers");
            std::fs::create_dir_all(&mcp_servers_dir)?;
            let bocha_dir = mcp_servers_dir.join("bocha-search");
            std::fs::create_dir_all(&bocha_dir)?;
            let bocha_script = bocha_dir.join("index.js");
            let script_content = include_str!("../mcp-servers/bocha-search/index.js");
            std::fs::write(&bocha_script, script_content)?;
            log::info!("RS::lib | bocha script deployed | path={}", bocha_script.display());

            // Create a shared HTTP client for all outbound requests.
            // This enables TCP connection pooling across LLM, provider, and MCP HTTP calls.
            let http_client = reqwest::Client::builder()
                .connect_timeout(std::time::Duration::from_secs(30))
                .build()
                .map_err(|e| AppError::ConfigError(format!("failed to create HTTP client: {}", e)))?;

            // Inject state so Tauri commands can access it via State<>.
            app.manage(AppState {
                http_client,
                db: std::sync::Mutex::new(db),
                config: std::sync::Mutex::new(settings),
                config_path,
                cancel: std::sync::Mutex::new(None),
                mcp_pool: Arc::new(mcp::pool::McpPool::new(app_data_dir)),
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
            commands::settings::test_provider_connection,
            commands::settings::verify_model,
            commands::settings::fetch_provider_models,
            commands::settings::log_message,
            commands::settings::open_url,
            commands::mcp::list_mcp_categories,
            commands::mcp::list_mcp_servers,
            commands::mcp::list_mcp_instances,
            commands::mcp::add_mcp_instance,
            commands::mcp::remove_mcp_instance,
            commands::mcp::toggle_mcp_instance,
            commands::mcp::start_mcp_instance,
            commands::mcp::stop_mcp_instance,
            commands::mcp::call_mcp_tool,
            commands::mcp::test_mcp_connection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
