use tauri::{AppHandle, Emitter};

use crate::{llm, models, store, AppState};
use tokio_util::sync::CancellationToken;

/// Guess the code language identifier from a filename extension.
pub fn guess_language(filename: &str) -> &'static str {
    let ext = filename.rfind('.').map(|i| &filename[i..]).unwrap_or("");
    match ext {
        ".js" | ".mjs" | ".cjs" | ".jsx" => "javascript",
        ".ts" | ".tsx" => "typescript",
        ".py" => "python",
        ".rs" => "rust",
        ".go" => "go",
        ".java" => "java",
        ".c" | ".h" => "c",
        ".cpp" | ".cc" | ".cxx" | ".hpp" => "cpp",
        ".cs" => "csharp",
        ".swift" => "swift",
        ".kt" => "kotlin",
        ".scala" => "scala",
        ".lua" => "lua",
        ".r" => "r",
        ".rb" => "ruby",
        ".php" => "php",
        ".css" | ".scss" | ".less" => "css",
        ".html" | ".htm" => "html",
        ".xml" | ".svg" => "xml",
        ".json" | ".jsonl" => "json",
        ".yaml" | ".yml" => "yaml",
        ".toml" => "toml",
        ".sql" => "sql",
        ".sh" | ".bash" | ".zsh" => "bash",
        ".bat" | ".cmd" => "batch",
        ".ps1" => "powershell",
        ".md" | ".markdown" => "markdown",
        ".vue" => "vue",
        ".svelte" => "svelte",
        _ => "",
    }
}

/// Conversation context: history messages + optional system prompt.
pub struct ConversationContext {
    pub messages: Vec<models::Message>,
    pub system_prompt: Option<String>,
}

/// Gather conversation history and system prompt, building the full message
/// array (with system prompt prepended if present).
pub fn gather_context(
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
            thinking_content: None,
            attachments: None,
        });
    }
    messages.extend(history.iter().cloned());

    Ok(ConversationContext { messages, system_prompt })
}

/// Resolved LLM configuration from the conversation's provider.
pub struct LlmConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub headers: std::collections::HashMap<String, String>,
    pub temperature: f32,
    pub top_p: f32,
}

/// Resolve the LLM provider and model for a conversation.
pub fn resolve_llm_config(
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
pub fn setup_cancel_token(state: &AppState) -> Result<CancellationToken, String> {
    let cancel = CancellationToken::new();
    let mut c = state.cancel.lock().map_err(|e| e.to_string())?;
    if let Some(previous) = c.replace(cancel.clone()) {
        previous.cancel();
    }
    Ok(cancel)
}

/// Invoke `stream_chat`, handle errors (emit `chat:error` on failure),
/// and return `(full_text, token_count)` on success.
pub async fn execute_stream(
    app: &AppHandle,
    conversation_id: &str,
    message_id: &str,
    cfg: &LlmConfig,
    messages: &[models::Message],
    cancel: CancellationToken,
) -> Result<Option<(String, String, Option<i64>)>, String> {
    let app_handle = app.clone();
    let mid = message_id.to_string();
    let app_handle_thinking = app.clone();
    let mid_thinking = message_id.to_string();
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
        move |delta| {
            let _ = app_handle_thinking.emit(
                "chat:thinking-chunk",
                serde_json::json!({
                    "message_id": mid_thinking,
                    "delta": delta,
                }),
            );
        },
    )
    .await;

    match result {
        Ok(stream_result) => Ok(Some((stream_result.content, stream_result.thinking, stream_result.tokens))),
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
pub fn finalize_response(
    app: &AppHandle,
    state: &AppState,
    conversation_id: String,
    message_id: String,
    full_text: String,
    thinking_content: String,
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
        thinking_content: if thinking_content.is_empty() { None } else { Some(thinking_content) },
        attachments: None,
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
