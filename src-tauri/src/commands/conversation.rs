use tauri::State;

use crate::store;
use crate::AppState;
use crate::models;

/// List all conversations, ordered by most recently updated first.
#[tauri::command]
pub fn list_conversations(
    state: State<'_, AppState>,
) -> Result<Vec<models::Conversation>, String> {
    log::debug!("Rust::commands::conversation::list_conversations | 查询对话列表");
    let db = state.db.lock().map_err(|e| e.to_string())?;
    store::list_conversations(&db).map_err(|e| e.to_string())
}

/// Create a new conversation with an optional title.
///
/// The model is inherited from the current application settings.
#[tauri::command]
pub fn create_conversation(
    provider_id: String,
    title: Option<String>,
    state: State<'_, AppState>,
) -> Result<models::Conversation, String> {
    log::info!("Rust::commands::conversation::create_conversation | 创建新对话 | title={:?} provider_id={}", title, provider_id);
    let model = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        // Find the provider and use its first model as default
        config.providers.iter()
            .find(|p| p.id == provider_id)
            .and_then(|p| p.models.first().cloned())
            .unwrap_or_else(|| "unknown".to_string())
    };

    let db = state.db.lock().map_err(|e| e.to_string())?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs() as i64;

    let conversation = models::Conversation {
        id: uuid::Uuid::new_v4().to_string(),
        title: title.unwrap_or_else(|| "新对话".to_string()),
        provider_id,
        model,
        system_prompt: String::new(),
        created_at: now,
        updated_at: now,
        pinned: false,
        search_enabled: false,
    };

    store::create_conversation(&db, &conversation).map_err(|e| e.to_string())?;
    Ok(conversation)
}

/// Update a conversation's title, provider_id, model, and/or search_enabled.
#[tauri::command]
pub fn update_conversation(
    id: String,
    title: Option<String>,
    provider_id: Option<String>,
    model: Option<String>,
    search_enabled: Option<bool>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Rust::commands::conversation::update_conversation | 更新对话 | id={} title={:?} provider_id={:?} model={:?} search_enabled={:?}", id, title, provider_id, model, search_enabled);
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let mut conversation = store::get_conversation(&db, &id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "对话不存在".to_string())?;

    if let Some(t) = title {
        conversation.title = t;
    }
    if let Some(p) = provider_id {
        conversation.provider_id = p;
    }
    if let Some(m) = model {
        conversation.model = m;
    }
    if let Some(s) = search_enabled {
        conversation.search_enabled = s;
    }
    conversation.updated_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs() as i64;

    store::update_conversation(&db, &conversation).map_err(|e| e.to_string())
}

/// Delete a conversation and all its messages.
#[tauri::command]
pub fn delete_conversation(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Rust::commands::conversation::delete_conversation | 删除对话 | id={}", id);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    store::delete_conversation(&db, &id).map_err(|e| e.to_string())
}

/// Pin a conversation so it appears at the top of the list.
#[tauri::command]
pub fn pin_conversation(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Rust::commands::conversation::pin_conversation | 置顶对话 | id={}", id);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    store::pin_conversation(&db, &id).map_err(|e| e.to_string())
}

/// Unpin a conversation so it returns to normal ordering.
#[tauri::command]
pub fn unpin_conversation(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Rust::commands::conversation::unpin_conversation | 取消置顶 | id={}", id);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    store::unpin_conversation(&db, &id).map_err(|e| e.to_string())
}
