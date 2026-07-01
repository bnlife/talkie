use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

use crate::models;
use crate::AppState;

/// List all market categories.
#[tauri::command]
pub fn list_mcp_categories(
    state: State<'_, AppState>,
) -> Result<Vec<models::McpCategory>, String> {
    log::debug!("RS::CMD::mcp | categories");
    let db = state.db.lock().map_err(|e| e.to_string())?;
    crate::store::list_mcp_categories(&db).map_err(|e| e.to_string())
}

/// List servers in a category (or all if categoryId is None).
#[tauri::command]
pub fn list_mcp_servers(
    category_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<models::McpServer>, String> {
    log::debug!("RS::CMD::mcp | servers | cat={:?}", category_id);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    crate::store::list_mcp_servers(&db, category_id.as_deref()).map_err(|e| e.to_string())
}

/// List all installed MCP instances.
#[tauri::command]
pub fn list_mcp_instances(
    state: State<'_, AppState>,
) -> Result<Vec<models::McpInstance>, String> {
    log::debug!("RS::CMD::mcp | instances");
    let db = state.db.lock().map_err(|e| e.to_string())?;
    crate::store::list_mcp_instances(&db).map_err(|e| e.to_string())
}

/// Install a new MCP instance.
#[tauri::command]
pub fn add_mcp_instance(
    instance: models::McpInstance,
    state: State<'_, AppState>,
) -> Result<models::McpInstance, String> {
    log::info!("RS::CMD::mcp | add | name={} server_id={}", instance.name, instance.server_id);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    crate::store::create_mcp_instance(&db, &instance).map_err(|e| e.to_string())?;
    Ok(instance)
}

/// Remove an installed MCP instance. Stops it first if running.
#[tauri::command]
pub fn remove_mcp_instance(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("RS::CMD::mcp | remove | id={}", id);
    // Stop if running
    let _ = state.mcp_pool.stop(&id);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    crate::store::delete_mcp_instance(&db, &id).map_err(|e| e.to_string())
}

/// Toggle an MCP instance enabled/disabled.
/// Updates DB immediately, then spawns/stops process in background.
/// Emits `mcp:started` or `mcp:error` events.
#[tauri::command]
pub async fn toggle_mcp_instance(
    id: String,
    enabled: bool,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("RS::CMD::mcp | toggle | id={} enabled={}", id, enabled);

    // Update DB immediately
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        crate::store::toggle_mcp_instance(&db, &id, enabled).map_err(|e| e.to_string())?;
    }

    if enabled {
        // Get instance config for background spawn
        let instance = {
            let db = state.db.lock().map_err(|e| e.to_string())?;
            crate::store::get_mcp_instance(&db, &id)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| "MCP 实例不存在".to_string())?
        };

        // Spawn in background thread — does NOT block the UI
        let pool = Arc::clone(&state.mcp_pool);
        let instance_id = id.clone();
        std::thread::spawn(move || {
            log::info!("RS::CMD::mcp | thread start | id={}", instance_id);
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                pool.start(&instance)
            }));
            match result {
                Ok(Ok(())) => {
                    log::info!("RS::CMD::mcp | started | id={}", instance_id);
                    let _ = app.emit("mcp:started", serde_json::json!({ "id": instance_id }));
                }
                Ok(Err(e)) => {
                    log::error!("RS::CMD::mcp | start fail | id={} err={}", instance_id, e);
                    let _ = app.emit("mcp:error", serde_json::json!({ "id": instance_id, "error": e }));
                }
                Err(panic) => {
                    let msg = if let Some(s) = panic.downcast_ref::<String>() {
                        s.clone()
                    } else if let Some(s) = panic.downcast_ref::<&str>() {
                        s.to_string()
                    } else {
                        "unknown panic".to_string()
                    };
                    log::error!("RS::CMD::mcp | panic | id={} err={}", instance_id, msg);
                    let _ = app.emit("mcp:error", serde_json::json!({ "id": instance_id, "error": format!("panic: {}", msg) }));
                }
            }
        });
    } else {
        // Stop is fast, can be synchronous
        state.mcp_pool.stop(&id)?;
    }

    Ok(())
}

/// Start an MCP instance (spawn process).
#[tauri::command]
pub fn start_mcp_instance(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("RS::CMD::mcp | start | id={}", id);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let instance = crate::store::get_mcp_instance(&db, &id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "MCP 实例不存在".to_string())?;
    drop(db);
    state.mcp_pool.start(&instance)
}

/// Stop an MCP instance (kill process).
#[tauri::command]
pub fn stop_mcp_instance(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("RS::CMD::mcp | stop | id={}", id);
    state.mcp_pool.stop(&id)
}

/// Call a tool on a running MCP instance.
#[tauri::command]
pub async fn call_mcp_tool(
    instance_id: String,
    tool_name: String,
    args: serde_json::Value,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    log::info!("RS::CMD::mcp | call | instance={} tool={}", instance_id, tool_name);
    state.mcp_pool.call_tool(&instance_id, &tool_name, args)
}
