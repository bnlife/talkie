use tauri::State;

use crate::store;
use crate::AppState;
use crate::models;

/// Send a message in a conversation.
///
/// Currently a placeholder that returns Ok(()). Will be extended to
/// stream the response from the LLM backend.
#[tauri::command]
pub fn send_message(
    conversation_id: String,
    content: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Rust::commands::chat::send_message | 收到发送消息请求 | conv={} len={}", conversation_id, content.len());
    // TODO: implement LLM streaming
    let _ = (conversation_id, content, state);
    Ok(())
}

/// Stop the currently streaming LLM response.
///
/// Currently a placeholder that returns Ok(()).
#[tauri::command]
pub fn stop_stream(
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Rust::commands::chat::stop_stream | 停止流式响应");
    // TODO: implement stream cancellation
    let _ = state;
    Ok(())
}

/// Retrieve all messages belonging to a conversation.
#[tauri::command]
pub fn get_messages(
    conversation_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<models::Message>, String> {
    log::debug!("Rust::commands::chat::get_messages | 查询消息列表 | conv={}", conversation_id);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    store::list_messages_by_conversation(&db, &conversation_id).map_err(|e| e.to_string())
}
