use tokio_util::sync::CancellationToken;

use crate::models;

/// Send a streaming chat-completion request to an OpenAI-compatible endpoint,
/// parse the SSE response line‑by‑line, invoke `on_chunk` for each content
/// delta, and return the full accumulated text.
///
/// # Arguments
///
/// * `base_url` - Base URL of the API (e.g. `https://api.openai.com/v1`)
/// * `api_key`  - API key used for Bearer authentication
/// * `model`    - Model identifier (e.g. `gpt-4o`)
/// * `messages` - Conversation history in the OpenAI message format
/// * `cancel`   - `CancellationToken` – checked before each chunk; when
///                cancelled the function returns early with an error
/// * `on_chunk` - Callback invoked for every non‑empty content delta received
///
/// # Returns
///
/// The full concatenated response text on success, or an error message.
pub async fn stream_chat<F>(
    base_url: &str,
    api_key: &str,
    model: &str,
    messages: &[models::Message],
    cancel: CancellationToken,
    on_chunk: F,
) -> Result<String, String>
where
    F: Fn(String) + Send + 'static,
{
    // ── Early cancellation check (no HTTP request if already cancelled) ──
    if cancel.is_cancelled() {
        log::warn!("Rust::llm::stream_chat | 流式请求被取消");
        return Err("请求已取消".to_string());
    }

    log::info!(
        "Rust::llm::stream_chat | 开始流式请求 | url={} model={} messages={}",
        base_url,
        model,
        messages.len()
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    // Build the OpenAI‑compatible streaming request body, converting our
    // internal `Message` struct to the expected JSON format.
    let msg_values: Vec<serde_json::Value> = messages
        .iter()
        .map(|m| {
            serde_json::json!({
                "role": m.role,
                "content": m.content,
            })
        })
        .collect();

    let body = serde_json::json!({
        "model": model,
        "messages": msg_values,
        "stream": true,
    });

    let body_str =
        serde_json::to_string(&body).map_err(|e| format!("序列化请求体失败: {}", e))?;

    // ── Send the POST request ──
    let mut response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .body(body_str)
        .send()
        .await
        .map_err(|e| {
            let msg = if e.is_timeout() {
                "请求超时".to_string()
            } else if e.is_connect() {
                format!("无法连接到服务器: {}", e)
            } else {
                format!("请求失败: {}", e)
            };
            log::error!("Rust::llm::stream_chat | ERR_net_connect | 请求失败: {}", msg);
            msg
        })?;

    // ── Non‑success HTTP status → error ──
    let status = response.status();
    log::debug!("Rust::llm::stream_chat | HTTP 响应 | status={}", status);
    if !status.is_success() {
        return Err(format!("HTTP error: {}", status));
    }

    // ── Stream the response body chunk‑by‑chunk, parsing SSE lines ──
    let mut accumulated = String::new();
    let mut buffer = String::new();

    // Use reqwest's chunk() API to read the response incrementally.
    // This avoids pulling in the `futures-util` crate.
    loop {
        // ── Check cancellation before processing each chunk ──
        if cancel.is_cancelled() {
            log::warn!("Rust::llm::stream_chat | 流式请求被取消");
            return Err("请求已取消".to_string());
        }

        let chunk = response.chunk().await.map_err(|e| format!("读取响应分块失败: {}", e))?;

        let chunk = match chunk {
            Some(c) => c,
            None => break, // stream ended normally
        };

        log::trace!("Rust::llm::stream_chat | 收到 chunk | len={}", chunk.len());

        let text = String::from_utf8_lossy(&chunk);

        // Append new data to the line buffer and process complete lines
        buffer.push_str(&text);

        while let Some(newline_pos) = buffer.find('\n') {
            let line = buffer[..newline_pos].trim().to_string();
            buffer = buffer[newline_pos + 1..].to_string();

            if line.is_empty() {
                continue;
            }

            // SSE data lines begin with "data: "
            if let Some(data) = line.strip_prefix("data: ") {
                let data = data.trim();

                // "[DONE]" signals the end of the stream
                if data == "[DONE]" {
                    log::info!("Rust::llm::stream_chat | 流式完成 | total_chars={}", accumulated.len());
                    return Ok(accumulated);
                }

                // Try to parse the JSON payload inside the "data:" field
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                    if let Some(choices) = json.get("choices").and_then(|c| c.as_array()) {
                        for choice in choices {
                            if let Some(delta) = choice.get("delta") {
                                if let Some(content) =
                                    delta.get("content").and_then(|c| c.as_str())
                                {
                                    if !content.is_empty() {
                                        on_chunk(content.to_string());
                                        accumulated.push_str(content);
                                    }
                                }
                            }
                        }
                    }
                } else {
                    log::warn!("Rust::llm::stream_chat | SSE 解析异常 | line={}", data);
                }
            }
            // Lines that don't start with "data:" (e.g. SSE comments `: ...`)
            // are ignored per the SSE spec.
        }
    }

    // If we exhausted the stream without seeing `[DONE]`, return whatever we
    // accumulated (the server may not send a termination event).
    log::info!("Rust::llm::stream_chat | 流式完成 | total_chars={}", accumulated.len());
    Ok(accumulated)
}
