use tauri::{AppHandle, State};

use crate::{chat, models, store, AppState};

/// Send a message in a conversation.
///
/// Creates a user message, persists it, streams the LLM response via SSE,
/// and saves the assistant's reply.  Emits `chat:stream-chunk` events for
/// each content delta and a final `chat:stream-done` event when complete.
#[tauri::command]
pub async fn send_message(
    conversation_id: String,
    content: String,
    attachments: Option<Vec<models::AttachmentMeta>>,
    search_enabled: bool,
    search_engine: Option<String>,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let att_count = attachments.as_ref().map_or(0, |a| a.len());
    log::info!(
        "RS::CMD::chat::send | conv={} len={} attachments={} search={} engine={:?}",
        conversation_id, content.len(), att_count, search_enabled, search_engine
    );

    // 1. Build the full content for LLM (user text + attachment contents).
    let llm_content = if let Some(ref atts) = attachments {
        if atts.is_empty() {
            content.clone()
        } else {
            let mut parts = vec![content.trim().to_string()];
            for att in atts {
                let file_content = att.content.as_deref().unwrap_or("");
                let lang = crate::chat::engine::guess_language(&att.name);
                let size_kb = (file_content.len() as f64 / 1024.0).round();
                parts.push(format!(
                    "---\n### 📎 附件: {} ({} KB)\n```{}\n{}\n```",
                    att.name, size_kb, lang, file_content
                ));
            }
            parts.join("\n\n")
        }
    } else {
        content.clone()
    };

    // 2. Create and persist the user message (store display content + attachments).
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
        search_results: None,
        thinking_content: None,
        attachments,
    };
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        store::create_message(&db, &msg).map_err(|e| e.to_string())?;
    }

    // 3. If search is enabled, find a running search MCP instance and call it.
    let (search_context, search_results) = if search_enabled {
        match chat::search::perform_search(&state, &llm_content, search_engine.as_deref()) {
            Ok((text, results)) => {
                log::info!("RS::CMD::chat | search ok | results={} text_len={}", results.len(), text.len());
                (Some(text), Some(results))
            }
            Err(e) => {
                log::warn!("RS::CMD::chat | search failed, skip | err={}", e);
                (None, None)
            }
        }
    } else {
        (None, None)
    };

    do_generate(&app, &state, &conversation_id, &llm_content, search_context, search_results).await
}

/// Regenerate the last assistant response without creating a new user message.
#[tauri::command]
pub async fn regenerate_message(
    conversation_id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!(
        "RS::CMD::chat::regen | conv={}",
        conversation_id
    );

    do_generate(&app, &state, &conversation_id, "", None, None).await
}

/// Core generation flow shared by `send_message` and `regenerate_message`.
async fn do_generate(
    app: &AppHandle,
    state: &State<'_, AppState>,
    conversation_id: &str,
    llm_content: &str,
    search_context: Option<String>,
    search_results: Option<Vec<models::SearchResult>>,
) -> Result<(), String> {
    // 1. Gather context (history + system prompt).
    let mut ctx = chat::engine::gather_context(state, conversation_id)?;
    if let Some(ref sys) = ctx.system_prompt {
        log::debug!("RS::CMD::chat | inject sys prompt | len={}", sys.len());
    }

    // 2. Replace the last user message content with llm_content (which includes attachments).
    if let Some(last) = ctx.messages.last_mut() {
        if last.role == "user" {
            last.content = llm_content.to_string();
        }
    }

    // 2. If we have search results, inject as a system message after system prompt.
    if let Some(ref search_text) = search_context {
        let search_msg = models::Message {
            id: "search-result".to_string(),
            conversation_id: conversation_id.to_string(),
            role: "system".to_string(),
            content: format!(
                "以下是联网搜索结果，每条结果前有编号如 [1] [2] 等。\n\
                 回答时必须引用这些来源：在相关陈述后面加上对应的编号角标，例如\"Rust 是一门系统编程语言[1]\"。\n\
                 如果一句话综合了多个来源，可以写 [1][2]。不要编造不存在的来源编号。\n\n\
                 {}",
                search_text
            ),
            created_at: 0,
            token_count: None,
            search_results: None,
            thinking_content: None,
            attachments: None,
        };
        // Insert after system prompt (index 0 if present) or at the beginning
        let insert_pos = if ctx.system_prompt.is_some() { 1 } else { 0 };
        ctx.messages.insert(insert_pos, search_msg);
        log::debug!("RS::CMD::chat | inject search | role=system len={}", search_text.len());
    }

    // 2. Resolve LLM provider config.
    let cfg = chat::engine::resolve_llm_config(state, conversation_id)?;
    log::info!("RS::CMD::chat | model={} provider={}", cfg.model, cfg.base_url);

    // 3. Set up cancellation token.
    let cancel = chat::engine::setup_cancel_token(state)?;

    // 4. Generate assistant message ID.
    let message_id = uuid::Uuid::new_v4().to_string();

    // 5. Stream the LLM response.
    let (full_text, thinking_content, usage_tokens) = match chat::engine::execute_stream(
        app, conversation_id, &message_id,
        &cfg, &ctx.messages, cancel,
    ).await? {
        Some(result) => result,
        None => return Ok(()), // Error already emitted via chat:error
    };

    // 6. Persist and signal completion.
    chat::engine::finalize_response(
        app, state,
        conversation_id.to_string(),
        message_id,
        full_text,
        thinking_content,
        usage_tokens,
        search_results,
    )
}

/// Stop the currently streaming LLM response.
#[tauri::command]
pub async fn stop_stream(state: State<'_, AppState>) -> Result<(), String> {
    log::info!("RS::CMD::chat::stop | user stop");
    let mut c = state.cancel.lock().map_err(|e| e.to_string())?;
    if let Some(token) = c.take() {
        token.cancel();
    } else {
        log::debug!("RS::CMD::chat::stop | no active stream");
    }
    Ok(())
}

/// Retrieve paginated messages belonging to a conversation.
#[tauri::command]
pub fn get_messages(
    conversation_id: String,
    offset: Option<i64>,
    limit: Option<i64>,
    state: State<'_, AppState>,
) -> Result<crate::models::MessagesPage, String> {
    log::debug!(
        "RS::CMD::chat::get | conv={} offset={:?} limit={:?}",
        conversation_id, offset, limit
    );
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let offset = offset.unwrap_or(0);
    let limit = limit.unwrap_or(30);
    store::list_messages_paginated(&db, &conversation_id, offset, limit).map_err(|e| e.to_string())
}

/// Delete a single message by its ID.
#[tauri::command]
pub fn delete_message(
    message_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!(
        "RS::CMD::chat::del | id={}",
        message_id
    );
    let db = state.db.lock().map_err(|e| e.to_string())?;
    store::delete_message(&db, &message_id).map_err(|e| e.to_string())
}
