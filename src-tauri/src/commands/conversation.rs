use tauri::State;

use crate::store;
use crate::AppState;
use crate::models;

/// List all conversations, ordered by most recently updated first.
#[tauri::command]
pub fn list_conversations(
    state: State<'_, AppState>,
) -> Result<Vec<models::Conversation>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    store::list_conversations(&db).map_err(|e| e.to_string())
}

/// Create a new conversation with an optional title.
///
/// The model is inherited from the current application settings.
#[tauri::command]
pub fn create_conversation(
    title: Option<String>,
    state: State<'_, AppState>,
) -> Result<models::Conversation, String> {
    let model = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        config.model.clone()
    };

    let db = state.db.lock().map_err(|e| e.to_string())?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs() as i64;

    let conversation = models::Conversation {
        id: uuid::Uuid::new_v4().to_string(),
        title: title.unwrap_or_else(|| "新对话".to_string()),
        model,
        system_prompt: String::new(),
        created_at: now,
        updated_at: now,
    };

    store::create_conversation(&db, &conversation).map_err(|e| e.to_string())?;
    Ok(conversation)
}

/// Update the title of an existing conversation.
#[tauri::command]
pub fn update_conversation(
    id: String,
    title: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let mut conversation = store::get_conversation(&db, &id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "对话不存在".to_string())?;

    conversation.title = title;
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
    let db = state.db.lock().map_err(|e| e.to_string())?;
    store::delete_conversation(&db, &id).map_err(|e| e.to_string())
}
