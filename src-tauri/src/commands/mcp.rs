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
pub async fn remove_mcp_instance(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("RS::CMD::mcp | remove | id={}", id);
    // Stop if running
    let _ = state.mcp_pool.stop(&id).await;
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

        // Spawn in background — does NOT block the UI
        let pool = Arc::clone(&state.mcp_pool);
        let instance_id = id.clone();
        tokio::spawn(async move {
            log::info!("RS::CMD::mcp | task start | id={}", instance_id);
            match pool.start(&instance).await {
                Ok(()) => {
                    log::info!("RS::CMD::mcp | started | id={}", instance_id);
                    let _ = app.emit("mcp:started", serde_json::json!({ "id": instance_id }));
                }
                Err(e) => {
                    log::error!("RS::CMD::mcp | start fail | id={} err={}", instance_id, e);
                    let _ = app.emit("mcp:error", serde_json::json!({ "id": instance_id, "error": e }));
                }
            }
        });
    } else {
        // Stop is fast, can be synchronous
        state.mcp_pool.stop(&id).await?;
    }

    Ok(())
}

/// Start an MCP instance (spawn process).
#[tauri::command]
pub async fn start_mcp_instance(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("RS::CMD::mcp | start | id={}", id);
    let instance = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        crate::store::get_mcp_instance(&db, &id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "MCP 实例不存在".to_string())?
    };
    state.mcp_pool.start(&instance).await
}

/// Stop an MCP instance (kill process).
#[tauri::command]
pub async fn stop_mcp_instance(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("RS::CMD::mcp | stop | id={}", id);
    state.mcp_pool.stop(&id).await
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
    state.mcp_pool.call_tool(&instance_id, &tool_name, args).await
}

/// Test MCP instance connectivity by listing its tools.
/// If the instance is not running, attempts to start it first.
#[tauri::command]
pub async fn test_mcp_connection(
    id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    log::info!("RS::CMD::mcp::test | start | id={}", id);

    // If not running, try to start
    let was_running = state.mcp_pool.is_running(&id).await;
    log::debug!("RS::CMD::mcp::test | was_running={} | id={}", was_running, id);

    if !was_running {
        let instance = {
            let db = state.db.lock().map_err(|e| e.to_string())?;
            crate::store::get_mcp_instance(&db, &id)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| "MCP 实例不存在".to_string())?
        };
        log::info!("RS::CMD::mcp::test | starting | id={} name={}", id, instance.name);
        state.mcp_pool.start(&instance).await.map_err(|e| {
            log::error!("RS::CMD::mcp::test | start fail | id={} err={}", id, e);
            format!("启动失败: {}", e)
        })?;
        log::info!("RS::CMD::mcp::test | start ok | id={}", id);
    }

    // Verify it's running
    let running = state.mcp_pool.is_running(&id).await;
    log::debug!("RS::CMD::mcp::test | is_running={} after start | id={}", running, id);

    if !running {
        return Err("MCP 实例启动后未在运行池中".to_string());
    }

    // Call list_tools to verify connectivity
    log::debug!("RS::CMD::mcp::test | calling list_tools | id={}", id);
    let tools = state.mcp_pool.list_tools(&id).await.map_err(|e| {
        log::error!("RS::CMD::mcp::test | list_tools fail | id={} err={}", id, e);
        format!("连接失败: {}", e)
    })?;

    let msg = format!("连接成功，发现 {} 个工具", tools.len());
    log::info!("RS::CMD::mcp::test | ok | id={} tools={}", id, tools.len());
    Ok(msg)
}
