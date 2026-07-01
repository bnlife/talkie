use tauri::State;

use crate::store;
use crate::AppState;
use crate::models;

/// List all conversations, ordered by most recently updated first.
#[tauri::command]
pub fn list_conversations(
    state: State<'_, AppState>,
) -> Result<Vec<models::ConversationView>, String> {
    log::debug!("Rust::commands::conversation::list_conversations | 查询对话列表");
    let db = state.db.lock().map_err(|e| e.to_string())?;
    store::list_conversations(&db).map_err(|e| e.to_string())
}

/// Create a new conversation with an optional title.
#[tauri::command]
pub fn create_conversation(
    provider_id: String,
    title: Option<String>,
    state: State<'_, AppState>,
) -> Result<models::ConversationView, String> {
    log::info!("Rust::commands::conversation::create_conversation | 创建新对话 | title={:?} provider_id={}", title, provider_id);
    let model = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
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

    let id = uuid::Uuid::new_v4().to_string();
    let conversation = models::Conversation {
        id: id.clone(),
        title: title.unwrap_or_else(|| "新对话".to_string()),
        created_at: now,
        updated_at: now,
        pinned: false,
    };

    let config = models::ConversationConfig {
        conversation_id: id.clone(),
        provider_id,
        model,
        prompt_id: None,
        search_enabled: false,
    };

    store::create_conversation(&db, &conversation, &config).map_err(|e| e.to_string())?;

    Ok(models::ConversationView {
        id: conversation.id,
        title: conversation.title,
        created_at: conversation.created_at,
        updated_at: conversation.updated_at,
        pinned: conversation.pinned,
        provider_id: config.provider_id,
        model: config.model,
        prompt_id: config.prompt_id,
        search_enabled: config.search_enabled,
    })
}

/// Update a conversation's config fields.
#[tauri::command]
pub fn update_conversation(
    id: String,
    title: Option<String>,
    provider_id: Option<String>,
    model: Option<String>,
    prompt_id: Option<String>,
    search_enabled: Option<bool>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Rust::commands::conversation::update_conversation | 更新对话 | id={}", id);
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let conv = store::get_conversation(&db, &id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "对话不存在".to_string())?;

    // Update conversation core fields if title changed
    if let Some(t) = title {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs() as i64;
        store::update_conversation(&db, &models::Conversation {
            id: conv.id.clone(),
            title: t,
            created_at: conv.created_at,
            updated_at: now,
            pinned: conv.pinned,
        }).map_err(|e| e.to_string())?;
    }

    // Update config fields if any changed
    if provider_id.is_some() || model.is_some() || prompt_id.is_some() || search_enabled.is_some() {
        store::update_conversation_config(&db, &models::ConversationConfig {
            conversation_id: id,
            provider_id: provider_id.unwrap_or(conv.provider_id),
            model: model.unwrap_or(conv.model),
            prompt_id: prompt_id.or(conv.prompt_id),
            search_enabled: search_enabled.unwrap_or(conv.search_enabled),
        }).map_err(|e| e.to_string())?;
    }

    Ok(())
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
