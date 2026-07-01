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
        match conv.as_ref().and_then(|c| c.prompt_id.as_ref()).filter(|s| !s.is_empty()) {
            Some(id) if id == "default" => {
                store::get_default_prompt(&db)
                    .map_err(|e| e.to_string())?
                    .map(|p| p.content)
            }
            Some(id) => {
                store::get_prompt_by_id(&db, id)
                    .map_err(|e| e.to_string())?
                    .map(|p| p.content)
            }
            None => {
                store::get_default_prompt(&db)
                    .map_err(|e| e.to_string())?
                    .map(|p| p.content)
            }
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
            search_results: None,
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
                log::warn!("RS::CMD::chat | stream cancelled | conv={}", conversation_id);
            } else {
                log::error!("RS::CMD::chat | stream failed | err={}", e);
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
    search_results: Option<Vec<models::SearchResult>>,
) -> Result<(), String> {
    let search_results_clone = search_results.clone();
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
        search_results,
    };
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        store::create_message(&db, &assistant_msg).map_err(|e| e.to_string())?;
    }
    log::info!(
        "RS::CMD::chat | assistant msg saved | msg_id={} chars={}",
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
            "search_results": search_results_clone,
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
    search_enabled: bool,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!(
        "RS::CMD::chat::send | conv={} len={} search={}",
        conversation_id, content.len(), search_enabled
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
        search_results: None,
    };
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        store::create_message(&db, &msg).map_err(|e| e.to_string())?;
    }

    // 2. If search is enabled, find a running search MCP instance and call it.
    let (search_context, search_results) = if search_enabled {
        match perform_search(&state, &content) {
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

    do_generate(&app, &state, &conversation_id, search_context, search_results).await
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

    do_generate(&app, &state, &conversation_id, None, None).await
}

// ---------------------------------------------------------------------------
// Search integration
// ---------------------------------------------------------------------------

/// Find a running MCP search instance and call it with the user's query.
/// Returns (text_for_llm, structured_results).
pub fn perform_search(state: &AppState, query: &str) -> Result<(String, Vec<models::SearchResult>), String> {
    log::info!("RS::CMD::search | start | query={}", query);

    // Find a running MCP instance that provides search
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let instances = store::list_mcp_instances(&db).map_err(|e| e.to_string())?;
    drop(db);

    log::debug!("RS::CMD::search | installed={}", instances.len());
    for inst in &instances {
        log::debug!("RS::CMD::search | instance | id={} server_id={} enabled={}", inst.id, inst.server_id, inst.enabled);
    }

    // Look for an enabled instance whose server is a search server
    let search_instance = instances.iter().find(|i| {
        i.enabled && (i.server_id == "brave-search" || i.server_id == "duckduckgo"
            || i.server_id == "bocha-search" || i.server_id == "local:bocha-search"
            || i.server_id.contains("search"))
    });

    let instance = match search_instance {
        Some(i) => {
            log::info!("RS::CMD::search | found | id={} server_id={}", i.id, i.server_id);
            i
        }
        None => {
            log::warn!("RS::CMD::search | no search instance");
            return Err("没有启用的搜索 MCP 实例".to_string());
        }
    };

    // If instance is not running in the pool, auto-start it
    if !state.mcp_pool.is_running(&instance.id) {
        log::info!("RS::CMD::search | auto-start | id={}", instance.id);
        state.mcp_pool.start(instance).map_err(|e| {
            log::error!("RS::CMD::search | auto-start failed | err={}", e);
            format!("搜索 MCP 实例启动失败: {}", e)
        })?;
    }

    // Call the search tool
    let tool_name = if instance.server_id.contains("bocha") {
        "bocha_search"
    } else {
        "search" // Generic fallback
    };

    let args = serde_json::json!({
        "query": query,
        "count": 5,
        "freshness": "noLimit",
        "summary": true,
    });

    let result = state.mcp_pool.call_tool(&instance.id, tool_name, args)?;
    log::debug!("RS::CMD::search | raw | {}", serde_json::to_string(&result).unwrap_or_default());

    // Parse structured results and format text for LLM
    let search_results = parse_search_results(&result);
    let text = format_search_results(&result);
    for (i, sr) in search_results.iter().enumerate() {
        log::info!("RS::CMD::search | parsed[{}] | title={} url={} snippet={:?}", i, sr.title, sr.url, sr.snippet);
    }
    log::info!("RS::CMD::search | done | results={} text_len={}", search_results.len(), text.len());

    Ok((text, search_results))
}

/// Format MCP search tool results into a readable context string with numbered citations.
fn format_search_results(result: &serde_json::Value) -> String {
    // Try to extract text content from MCP tool result
    if let Some(content) = result.get("content").and_then(|c| c.as_array()) {
        let mut texts = Vec::new();
        for item in content {
            if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                texts.push(text.to_string());
            }
        }
        if !texts.is_empty() {
            return texts.join("\n");
        }
    }

    // Fallback: use the raw JSON
    serde_json::to_string_pretty(result).unwrap_or_default()
}

/// Parse MCP search tool results into structured `SearchResult` items.
///
/// Tries to extract `[title](url)` links from the text content.
/// Returns an empty vec if no structured results can be parsed.
pub fn parse_search_results(result: &serde_json::Value) -> Vec<models::SearchResult> {
    let mut results = Vec::new();

    let text = match result.get("content").and_then(|c| c.as_array()) {
        Some(items) => {
            items.iter()
                .filter_map(|item| item.get("text").and_then(|t| t.as_str()))
                .collect::<Vec<_>>()
                .join("\n")
        }
        None => return results,
    };

    // Pattern: "N. [title](url)" followed by optional snippet line (non-URL, non-numbered)
    let re = regex::Regex::new(r"(?m)^\d+\.\s+\[([^\]]+)\]\(([^)]+)\)(?:\n[ \t]+([^\n\d][^\n]*))?")
        .expect("invalid regex");

    for cap in re.captures_iter(&text) {
        let title = cap.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
        let url = cap.get(2).map(|m| m.as_str().to_string()).unwrap_or_default();
        let snippet = cap.get(3).map(|m| {
            let s = m.as_str().trim();
            // Filter out "Published:" lines from snippet
            if s.starts_with("Published:") { None } else { Some(s.to_string()) }
        }).flatten();

        if !title.is_empty() && !url.is_empty() {
            results.push(models::SearchResult { title, url, snippet });
        }
    }

    log::info!("RS::CMD::parse | count={}", results.len());
    results
}

// ---------------------------------------------------------------------------
// Shared generation logic
// ---------------------------------------------------------------------------

/// Core generation flow shared by `send_message` and `regenerate_message`.
async fn do_generate(
    app: &AppHandle,
    state: &State<'_, AppState>,
    conversation_id: &str,
    search_context: Option<String>,
    search_results: Option<Vec<models::SearchResult>>,
) -> Result<(), String> {
    // 1. Gather context (history + system prompt).
    let mut ctx = gather_context(state, conversation_id)?;
    if let Some(ref sys) = ctx.system_prompt {
        log::debug!("RS::CMD::chat | inject sys prompt | len={}", sys.len());
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
        };
        // Insert after system prompt (index 0 if present) or at the beginning
        let insert_pos = if ctx.system_prompt.is_some() { 1 } else { 0 };
        ctx.messages.insert(insert_pos, search_msg);
        log::debug!("RS::CMD::chat | inject search | role=system len={}", search_text.len());
    }

    // 2. Resolve LLM provider config.
    let cfg = resolve_llm_config(state, conversation_id)?;
    log::info!("RS::CMD::chat | model={} provider={}", cfg.model, cfg.base_url);

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

/// Retrieve all messages belonging to a conversation.
#[tauri::command]
pub fn get_messages(
    conversation_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<models::Message>, String> {
    log::debug!(
        "RS::CMD::chat::get | conv={}",
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
        "RS::CMD::chat::del | id={}",
        message_id
    );
    let db = state.db.lock().map_err(|e| e.to_string())?;
    store::delete_message(&db, &message_id).map_err(|e| e.to_string())
}
