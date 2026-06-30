use tauri::{AppHandle, Emitter, State};

use crate::{llm, models, store, AppState};
use tokio_util::sync::CancellationToken;

/// Send a message in a conversation.
///
/// Creates a user message, persists it, streams the LLM response via SSE,
/// and saves the assistant's reply.  Emits `chat:stream-chunk` events for
/// each content delta and a final `chat:stream-done` event when complete.
#[tauri::command]
pub async fn send_message(
    conversation_id: String,
    content: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!(
        "Rust::commands::chat::send_message | 发送消息 | conv={} len={}",
        conversation_id,
        content.len()
    );

    // 1. Create the user message and persist it.
    let msg = models::Message {
        id: uuid::Uuid::new_v4().to_string(),
        conversation_id: conversation_id.clone(),
        role: "user".to_string(),
        content: content.clone(),
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
        token_count: None,
    };
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        store::create_message(&db, &msg).map_err(|e| e.to_string())?;
    }
    log::debug!(
        "Rust::commands::chat::send_message | 用户消息已保存 | msg_id={}",
        msg.id
    );

    // 2. Retrieve the full conversation history.
    let history = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        store::list_messages_by_conversation(&db, &conversation_id)
            .map_err(|e| e.to_string())?
    };

    // 3. Read the current configuration.
    let (base_url, api_key, model) = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        (
            config.base_url.clone(),
            config.api_key.clone(),
            config.model.clone(),
        )
    };

    // 4. Create a CancellationToken and store it in AppState so that
    //    `stop_stream` can cancel this stream.
    let cancel = CancellationToken::new();
    {
        // If there is an existing token from a previous stream, cancel it.
        let mut c = state.cancel.lock().map_err(|e| e.to_string())?;
        if let Some(previous) = c.replace(cancel.clone()) {
            previous.cancel();
        }
    }

    // 5. Generate an ID for the assistant message before we start streaming.
    let message_id = uuid::Uuid::new_v4().to_string();

    // 6. Invoke the streaming LLM call.  Each content delta is forwarded to
    //    the frontend via a `chat:stream-chunk` event.
    let app_handle = app.clone();
    let mid = message_id.clone();
    let full_text = match llm::stream_chat(
        &base_url,
        &api_key,
        &model,
        &history,
        cancel.clone(),
        move |delta| {
            let _ = app_handle.emit(
                "chat:stream-chunk",
                serde_json::json!({
                    "message_id": mid,
                    "delta": delta,
                }),
            );
        },
    )
    .await
    {
        Ok(text) => text,
        Err(e) => {
            if e.contains("请求已取消") {
                log::warn!(
                    "Rust::commands::chat::send_message | 助手消息被取消 | conv={}",
                    conversation_id
                );
            } else {
                log::error!(
                    "Rust::commands::chat::send_message | 流式请求失败 | err={}",
                    e
                );
            }
            // 通过 event 推送错误给前端
            let _ = app.emit(
                "chat:error",
                serde_json::json!({
                    "message": e,
                }),
            );
            return Ok(());
        }
    };

    // 7. Persist the assistant's full response.
    let assistant_msg = models::Message {
        id: message_id.clone(),
        conversation_id,
        role: "assistant".to_string(),
        content: full_text,
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
        token_count: None,
    };
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        store::create_message(&db, &assistant_msg).map_err(|e| e.to_string())?;
    }
    log::info!(
        "Rust::commands::chat::send_message | 助手消息已保存 | msg_id={} chars={}",
        assistant_msg.id,
        assistant_msg.content.len()
    );

    // 8. Clean up the cancellation token from AppState (our stream is done).
    {
        let mut c = state.cancel.lock().map_err(|e| e.to_string())?;
        *c = None;
    }

    // 9. Signal completion to the frontend.
    let _ = app.emit(
        "chat:stream-done",
        serde_json::json!({
            "message_id": message_id,
        }),
    );

    Ok(())
}

/// Stop the currently streaming LLM response.
///
/// Retrieves the stored `CancellationToken` from AppState and calls
/// `.cancel()` on it, which causes `stream_chat` to abort on its next
/// cancellation check.
#[tauri::command]
pub async fn stop_stream(state: State<'_, AppState>) -> Result<(), String> {
    log::info!("Rust::commands::chat::stop_stream | 用户停止生成");
    let mut c = state.cancel.lock().map_err(|e| e.to_string())?;
    if let Some(token) = c.take() {
        token.cancel();
    } else {
        log::debug!("Rust::commands::chat::stop_stream | 无活跃流式可取消");
    }
    Ok(())
}

/// Retrieve all messages belonging to a conversation.
#[tauri::command]
pub fn get_messages(
    conversation_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<models::Message>, String> {
    log::debug!(
        "Rust::commands::chat::get_messages | 查询消息列表 | conv={}",
        conversation_id
    );
    let db = state.db.lock().map_err(|e| e.to_string())?;
    store::list_messages_by_conversation(&db, &conversation_id).map_err(|e| e.to_string())
}

/// Delete a single message by its ID.
#[tauri::command]
pub fn delete_message(
    message_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!(
        "Rust::commands::chat::delete_message | 删除消息 | id={}",
        message_id
    );
    let db = state.db.lock().map_err(|e| e.to_string())?;
    store::delete_message(&db, &message_id).map_err(|e| e.to_string())
}
