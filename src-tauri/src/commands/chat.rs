use tauri::{AppHandle, Emitter, State};

use crate::{llm, models, store, AppState};
use tokio_util::sync::CancellationToken;

// ---------------------------------------------------------------------------
// Internal helpers — shared between send_message and regenerate_message
// ---------------------------------------------------------------------------

/// Conversation context: history messages + optional system prompt.
struct ConversationContext {
    messages: Vec<models::Message>,
    system_prompt: Option<String>,
}

/// Gather conversation history and system prompt, building the full message
/// array (with system prompt prepended if present).
fn gather_context(
    state: &AppState,
    conversation_id: &str,
) -> Result<ConversationContext, String> {
    let history = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        store::list_messages_by_conversation(&db, conversation_id)
            .map_err(|e| e.to_string())?
    };

    let system_prompt = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let conv = store::get_conversation(&db, conversation_id)
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

    let mut messages: Vec<models::Message> = Vec::new();
    if let Some(ref sys) = system_prompt {
        messages.push(models::Message {
            id: "system".to_string(),
            conversation_id: conversation_id.to_string(),
            role: "system".to_string(),
            content: sys.clone(),
            created_at: 0,
            token_count: None,
        });
    }
    messages.extend(history.iter().cloned());

    Ok(ConversationContext { messages, system_prompt })
}

/// Resolved LLM configuration from the conversation's provider.
struct LlmConfig {
    base_url: String,
    api_key: String,
    model: String,
    headers: std::collections::HashMap<String, String>,
    temperature: f32,
    top_p: f32,
}

/// Resolve the LLM provider and model for a conversation.
fn resolve_llm_config(
    state: &AppState,
    conversation_id: &str,
) -> Result<LlmConfig, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conv = store::get_conversation(&db, conversation_id)
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
            Ok(LlmConfig {
                base_url: p.base_url.clone(),
                api_key: p.api_key.clone(),
                model: m,
                headers: p.headers.clone(),
                temperature: config.temperature,
                top_p: config.top_p,
            })
        }
        None => Ok(LlmConfig {
            base_url: "https://api.openai.com/v1".to_string(),
            api_key: String::new(),
            model: "gpt-3.5-turbo".to_string(),
            headers: std::collections::HashMap::new(),
            temperature: config.temperature,
            top_p: config.top_p,
        }),
    }
}

/// Create or replace the cancellation token in AppState.
fn setup_cancel_token(state: &AppState) -> Result<CancellationToken, String> {
    let cancel = CancellationToken::new();
    let mut c = state.cancel.lock().map_err(|e| e.to_string())?;
    if let Some(previous) = c.replace(cancel.clone()) {
        previous.cancel();
    }
    Ok(cancel)
}

/// Invoke `stream_chat`, handle errors (emit `chat:error` on failure),
/// and return `(full_text, token_count)` on success.
async fn execute_stream(
    app: &AppHandle,
    conversation_id: &str,
    message_id: &str,
    cfg: &LlmConfig,
    messages: &[models::Message],
    cancel: CancellationToken,
) -> Result<Option<(String, Option<i64>)>, String> {
    let app_handle = app.clone();
    let mid = message_id.to_string();
    let result = llm::stream_chat(
        &cfg.base_url,
        &cfg.api_key,
        &cfg.model,
        &cfg.headers,
        cfg.temperature,
        cfg.top_p,
        messages,
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
    .await;

    match result {
        Ok((text, tokens)) => Ok(Some((text, tokens))),
        Err(e) => {
            if e.contains("请求已取消") {
                log::warn!("Rust::chat | 流式被取消 | conv={}", conversation_id);
            } else {
                log::error!("Rust::chat | 流式请求失败 | err={}", e);
            }
            let _ = app.emit(
                "chat:error",
                serde_json::json!({ "message": e }),
            );
            Ok(None)
        }
    }
}

/// Persist the assistant message and emit `chat:stream-done`.
fn finalize_response(
    app: &AppHandle,
    state: &AppState,
    conversation_id: String,
    message_id: String,
    full_text: String,
    usage_tokens: Option<i64>,
) -> Result<(), String> {
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
        "Rust::chat | 助手消息已保存 | msg_id={} chars={}",
        assistant_msg.id,
        assistant_msg.content.len()
    );

    // Clean up the cancellation token.
    {
        let mut c = state.cancel.lock().map_err(|e| e.to_string())?;
        *c = None;
    }

    let _ = app.emit(
        "chat:stream-done",
        serde_json::json!({
            "message_id": message_id,
            "token_count": usage_tokens,
        }),
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Public commands
// ---------------------------------------------------------------------------

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

    // 1. Create and persist the user message.
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

    do_generate(&app, &state, &conversation_id).await
}

/// Regenerate the last assistant response without creating a new user message.
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

    do_generate(&app, &state, &conversation_id).await
}

// ---------------------------------------------------------------------------
// Shared generation logic
// ---------------------------------------------------------------------------

/// Core generation flow shared by `send_message` and `regenerate_message`.
async fn do_generate(
    app: &AppHandle,
    state: &State<'_, AppState>,
    conversation_id: &str,
) -> Result<(), String> {
    // 1. Gather context (history + system prompt).
    let ctx = gather_context(state, conversation_id)?;
    if let Some(ref sys) = ctx.system_prompt {
        log::debug!("Rust::chat | 注入 system prompt | len={}", sys.len());
    }

    // 2. Resolve LLM provider config.
    let cfg = resolve_llm_config(state, conversation_id)?;
    log::debug!("Rust::chat | 使用 model={}", cfg.model);

    // 3. Set up cancellation token.
    let cancel = setup_cancel_token(state)?;

    // 4. Generate assistant message ID.
    let message_id = uuid::Uuid::new_v4().to_string();

    // 5. Stream the LLM response.
    let (full_text, usage_tokens) = match execute_stream(
        app, conversation_id, &message_id,
        &cfg, &ctx.messages, cancel,
    ).await? {
        Some(result) => result,
        None => return Ok(()), // Error already emitted via chat:error
    };

    // 6. Persist and signal completion.
    finalize_response(
        app, state,
        conversation_id.to_string(),
        message_id,
        full_text,
        usage_tokens,
    )
}

/// Stop the currently streaming LLM response.
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
