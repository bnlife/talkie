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

    // 2.1 Determine the system prompt: conversation's own → default template → none.
    let system_prompt = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let conv = store::get_conversation(&db, &conversation_id)
            .map_err(|e| e.to_string())?;
        let from_conv = conv.as_ref().and_then(|c| {
            if c.system_prompt.is_empty() { None } else { Some(c.system_prompt.clone()) }
        });
        if from_conv.is_some() {
            from_conv
        } else {
            store::get_default_prompt(&db)
                .map_err(|e| e.to_string())?
                .map(|p| p.content)
        }
    };

    // Build the messages array with system prompt prepended.
    let mut messages: Vec<models::Message> = Vec::new();
    if let Some(ref sys) = system_prompt {
        log::debug!(
            "Rust::commands::chat::send_message | 注入 system prompt | len={}",
            sys.len()
        );
        messages.push(models::Message {
            id: "system".to_string(),
            conversation_id: conversation_id.clone(),
            role: "system".to_string(),
            content: sys.clone(),
            created_at: 0,
            token_count: None,
        });
    }
    messages.extend(history.iter().cloned());

    // 3. Read the current configuration from the conversation's provider.
    let (base_url, api_key, model, headers, temperature, top_p) = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let conv = store::get_conversation(&db, &conversation_id)
            .map_err(|e| e.to_string())?;

        let (provider, conv_model) = match conv {
            Some(ref c) if !c.provider_id.is_empty() => {
                let p = config.providers.iter().find(|p| p.id == c.provider_id);
                (p, Some(c.model.clone()))
            }
            _ => {
                let p = config.providers.iter().find(|p| p.id == config.active_provider_id);
                (p, None)
            }
        };

        match provider {
            Some(p) => {
                let m = conv_model.unwrap_or_else(|| p.models.first().cloned().unwrap_or_default());
                log::debug!(
                    "Rust::commands::chat::send_message | 使用 provider | name={} model={}",
                    p.name, m
                );
                (p.base_url.clone(), p.api_key.clone(), m, p.headers.clone(), config.temperature, config.top_p)
            }
            None => {
                log::warn!("Rust::commands::chat::send_message | 未找到 provider，使用默认");
                ("https://api.openai.com/v1".to_string(), String::new(), "gpt-3.5-turbo".to_string(), std::collections::HashMap::new(), config.temperature, config.top_p)
            }
        }
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
    let (full_text, usage_tokens) = match llm::stream_chat(
        &base_url,
        &api_key,
        &model,
        &headers,
        temperature,
        top_p,
        &messages,
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
        Ok((text, tokens)) => (text, tokens),
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
        token_count: usage_tokens,
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
            "token_count": usage_tokens,
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

/// Regenerate the last assistant response without creating a new user message.
///
/// Fetches the conversation history and streams a new LLM response.
/// Emits `chat:stream-chunk` and `chat:stream-done` events like `send_message`.
#[tauri::command]
pub async fn regenerate_message(
    conversation_id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!(
        "Rust::commands::chat::regenerate_message | 重新生成 | conv={}",
        conversation_id
    );

    // 1. Retrieve the full conversation history.
    let history = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        store::list_messages_by_conversation(&db, &conversation_id)
            .map_err(|e| e.to_string())?
    };

    // 1.1 Determine the system prompt: conversation's own → default template → none.
    let system_prompt = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let conv = store::get_conversation(&db, &conversation_id)
            .map_err(|e| e.to_string())?;
        let from_conv = conv.as_ref().and_then(|c| {
            if c.system_prompt.is_empty() { None } else { Some(c.system_prompt.clone()) }
        });
        if from_conv.is_some() {
            from_conv
        } else {
            store::get_default_prompt(&db)
                .map_err(|e| e.to_string())?
                .map(|p| p.content)
        }
    };

    // Build the messages array with system prompt prepended.
    let mut messages: Vec<models::Message> = Vec::new();
    if let Some(ref sys) = system_prompt {
        log::debug!(
            "Rust::commands::chat::regenerate_message | 注入 system prompt | len={}",
            sys.len()
        );
        messages.push(models::Message {
            id: "system".to_string(),
            conversation_id: conversation_id.clone(),
            role: "system".to_string(),
            content: sys.clone(),
            created_at: 0,
            token_count: None,
        });
    }
    messages.extend(history.iter().cloned());

    // 2. Read the current configuration from the conversation's provider.
    let (base_url, api_key, model, headers, temperature, top_p) = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let conv = store::get_conversation(&db, &conversation_id)
            .map_err(|e| e.to_string())?;

        let (provider, conv_model) = match conv {
            Some(ref c) if !c.provider_id.is_empty() => {
                let p = config.providers.iter().find(|p| p.id == c.provider_id);
                (p, Some(c.model.clone()))
            }
            _ => {
                let p = config.providers.iter().find(|p| p.id == config.active_provider_id);
                (p, None)
            }
        };

        match provider {
            Some(p) => {
                let m = conv_model.unwrap_or_else(|| p.models.first().cloned().unwrap_or_default());
                log::debug!(
                    "Rust::commands::chat::regenerate_message | 使用 provider | name={} model={}",
                    p.name, m
                );
                (p.base_url.clone(), p.api_key.clone(), m, p.headers.clone(), config.temperature, config.top_p)
            }
            None => {
                log::warn!("Rust::commands::chat::regenerate_message | 未找到 provider，使用默认");
                ("https://api.openai.com/v1".to_string(), String::new(), "gpt-3.5-turbo".to_string(), std::collections::HashMap::new(), config.temperature, config.top_p)
            }
        }
    };

    // 3. Create a CancellationToken and store it in AppState.
    let cancel = CancellationToken::new();
    {
        let mut c = state.cancel.lock().map_err(|e| e.to_string())?;
        if let Some(previous) = c.replace(cancel.clone()) {
            previous.cancel();
        }
    }

    // 4. Generate an ID for the assistant message.
    let message_id = uuid::Uuid::new_v4().to_string();

    // 5. Invoke the streaming LLM call.
    let app_handle = app.clone();
    let mid = message_id.clone();
    let (full_text, usage_tokens) = match llm::stream_chat(
        &base_url,
        &api_key,
        &model,
        &headers,
        temperature,
        top_p,
        &messages,
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
        Ok((text, tokens)) => (text, tokens),
        Err(e) => {
            if e.contains("请求已取消") {
                log::warn!(
                    "Rust::commands::chat::regenerate_message | 助手消息被取消 | conv={}",
                    conversation_id
                );
            } else {
                log::error!(
                    "Rust::commands::chat::regenerate_message | 流式请求失败 | err={}",
                    e
                );
            }
            let _ = app.emit(
                "chat:error",
                serde_json::json!({
                    "message": e,
                }),
            );
            return Ok(());
        }
    };

    // 6. Persist the assistant's full response.
    let assistant_msg = models::Message {
        id: message_id.clone(),
        conversation_id,
        role: "assistant".to_string(),
        content: full_text,
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
        token_count: usage_tokens,
    };
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        store::create_message(&db, &assistant_msg).map_err(|e| e.to_string())?;
    }
    log::info!(
        "Rust::commands::chat::regenerate_message | 助手消息已保存 | msg_id={} chars={}",
        assistant_msg.id,
        assistant_msg.content.len()
    );

    // 7. Clean up the cancellation token.
    {
        let mut c = state.cancel.lock().map_err(|e| e.to_string())?;
        *c = None;
    }

    // 8. Signal completion to the frontend.
    let _ = app.emit(
        "chat:stream-done",
        serde_json::json!({
            "message_id": message_id,
            "token_count": usage_tokens,
        }),
    );

    Ok(())
}
