use tokio_util::sync::CancellationToken;

use crate::models;

/// Result of a streaming chat completion.
#[derive(Debug, PartialEq)]
pub struct StreamResult {
    pub content: String,
    pub thinking: String,
    pub tokens: Option<i64>,
}

/// Send a streaming chat-completion request to an OpenAI-compatible endpoint,
/// parse the SSE response line‑by‑line, invoke `on_chunk` for each content
/// delta, `on_thinking` for each thinking delta, and return the accumulated text.
pub async fn stream_chat<F, G>(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    model: &str,
    headers: &std::collections::HashMap<String, String>,
    temperature: f32,
    top_p: f32,
    messages: &[models::Message],
    cancel: CancellationToken,
    on_chunk: F,
    on_thinking: G,
) -> Result<StreamResult, String>
where
    F: Fn(String) + Send + 'static,
    G: Fn(String) + Send + 'static,
{
    if cancel.is_cancelled() {
        log::warn!("RS::llm | cancelled");
        return Err("请求已取消".to_string());
    }

    let has_system = messages.first().is_some_and(|m| m.role == "system");
    log::info!(
        "RS::llm::stream | url={} model={} msgs={} temp={} top_p={} sys={}",
        base_url, model, messages.len(), temperature, top_p, has_system
    );

    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

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
        "temperature": temperature,
        "top_p": top_p,
        "stream": true,
        "stream_options": {
            "include_usage": true
        },
        "thinking": {
            "type": "enabled"
        },
    });

    let body_str =
        serde_json::to_string(&body).map_err(|e| format!("序列化请求体失败: {}", e))?;

    let mut request = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json");

    for (key, value) in headers {
        log::trace!("RS::llm | header | {}={}", key, value);
        request = request.header(key.as_str(), value.as_str());
    }

    let mut response = request
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
            log::error!("RS::llm | net err: {}", msg);
            msg
        })?;

    let status = response.status();
    log::debug!("RS::llm | HTTP status={}", status);
    if !status.is_success() {
        return Err(format!("HTTP error: {}", status));
    }

    let mut accumulated = String::new();
    let mut thinking_accumulated = String::new();
    let mut buffer = String::new();
    let mut total_tokens: Option<i64> = None;

    loop {
        if cancel.is_cancelled() {
            log::warn!("RS::llm | cancelled");
            return Err("请求已取消".to_string());
        }

        let chunk = match tokio::time::timeout(
            std::time::Duration::from_secs(60),
            response.chunk()
        ).await {
            Ok(result) => result.map_err(|e| format!("读取响应分块失败: {}", e))?,
            Err(_) => {
                log::error!("RS::llm | chunk timeout (60s)");
                return Err("读取响应超时 (60秒无数据)".to_string());
            }
        };
        let chunk = match chunk {
            Some(c) => c,
            None => break,
        };

        log::trace!("RS::llm | chunk len={}", chunk.len());

        let text = String::from_utf8_lossy(&chunk);
        buffer.push_str(&text);

        while let Some(newline_pos) = buffer.find('\n') {
            let line = buffer[..newline_pos].trim().to_string();
            buffer = buffer[newline_pos + 1..].to_string();

            if line.is_empty() {
                continue;
            }

            if let Some(data) = line.strip_prefix("data: ") {
                let data = data.trim();

                if data == "[DONE]" {
                    log::info!("RS::llm::stream | done | content={} thinking={} tokens={:?}", accumulated.len(), thinking_accumulated.len(), total_tokens);
                    return Ok(StreamResult {
                        content: accumulated,
                        thinking: thinking_accumulated,
                        tokens: total_tokens,
                    });
                }

                if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                    // Parse usage from streaming response (sent when stream_options.include_usage = true)
                    if let Some(usage) = json.get("usage") {
                        if let Some(total) = usage.get("total_tokens").and_then(|t| t.as_i64()) {
                            total_tokens = Some(total);
                        }
                    }
                    if let Some(choices) = json.get("choices").and_then(|c| c.as_array()) {
                        for choice in choices {
                            if let Some(delta) = choice.get("delta") {
                                // Thinking/reasoning content (DeepSeek R1, MiMo)
                                if let Some(thinking) =
                                    delta.get("reasoning_content").and_then(|c| c.as_str())
                                {
                                    if !thinking.is_empty() {
                                        on_thinking(thinking.to_string());
                                        thinking_accumulated.push_str(thinking);
                                    }
                                }
                                // Final answer content
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
                    log::warn!("RS::llm | SSE parse err | line={}", data);
                }
            }
        }
    }

    log::info!("RS::llm::stream | done | content={} thinking={} tokens={:?}", accumulated.len(), thinking_accumulated.len(), total_tokens);
    Ok(StreamResult {
        content: accumulated,
        thinking: thinking_accumulated,
        tokens: total_tokens,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Message;

    fn test_client() -> reqwest::Client {
        reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap()
    }

    fn run_with_mock<F>(test_fn: F)
    where
        F: FnOnce(mockito::ServerGuard) + Send + 'static,
    {
        let handle = std::thread::spawn(move || {
            let server = mockito::Server::new();
            test_fn(server);
        });
        handle.join().unwrap();
    }

    #[test]
    fn stream_chat_with_params_and_sse_parsing() {
        run_with_mock(|mut server| {
            let sse_body = "data: {\"choices\":[{\"delta\":{\"content\":\"Hi\"}}]}\n\ndata: [DONE]\n\n";
            let mock = server.mock("POST", "/chat/completions")
                .match_request(|req| {
                    let body = req.utf8_lossy_body().unwrap_or_default();
                    let v: serde_json::Value = serde_json::from_str(&body).unwrap_or_default();
                    v["model"] == "gpt-4o"
                        && (v["temperature"].as_f64().unwrap_or(0.0) - 0.8).abs() < 0.01
                        && (v["top_p"].as_f64().unwrap_or(0.0) - 0.95).abs() < 0.01
                        && v["stream"] == true
                        && v["messages"].as_array().map_or(false, |m| m.len() == 2)
                        && v["messages"][0]["role"] == "system"
                        && v["messages"][0]["content"] == "你是翻译助手"
                        && v["messages"][1]["role"] == "user"
                        && v["messages"][1]["content"] == "hello"
                })
                .with_status(200)
                .with_header("content-type", "text/event-stream")
                .with_body(sse_body)
                .create();

            let messages = vec![
                Message { id: "system".into(), conversation_id: "c1".into(), role: "system".into(), content: "你是翻译助手".into(), created_at: 0, token_count: None, search_results: None, thinking_content: None, attachments: None },
                Message { id: "u1".into(), conversation_id: "c1".into(), role: "user".into(), content: "hello".into(), created_at: 0, token_count: None, search_results: None, thinking_content: None, attachments: None },
            ];

            let client = test_client();
            let mut headers = std::collections::HashMap::new();
            headers.insert("X-Custom".to_string(), "test".to_string());

            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(stream_chat(
                &client, &server.url(), "sk-test", "gpt-4o", &headers,
                0.8, 0.95, &messages, CancellationToken::new(), |_| {}, |_| {},
            ));

            assert!(result.is_ok());
            mock.assert();
        });
    }

    #[test]
    fn request_body_without_system_prompt() {
        run_with_mock(|mut server| {
            let sse_body = "data: {\"choices\":[{\"delta\":{\"content\":\"OK\"}}]}\n\ndata: [DONE]\n\n";
            let mock = server.mock("POST", "/chat/completions")
                .match_request(|req| {
                    let body = req.utf8_lossy_body().unwrap_or_default();
                    let v: serde_json::Value = serde_json::from_str(&body).unwrap_or_default();
                    v["model"] == "deepseek-chat"
                        && (v["temperature"].as_f64().unwrap_or(0.0) - 0.5).abs() < 0.01
                        && (v["top_p"].as_f64().unwrap_or(0.0) - 1.0).abs() < 0.01
                        && v["messages"].as_array().map_or(false, |m| m.len() == 1)
                        && v["messages"][0]["role"] == "user"
                })
                .with_status(200)
                .with_header("content-type", "text/event-stream")
                .with_body(sse_body)
                .create();

            let messages = vec![
                Message { id: "u1".into(), conversation_id: "c1".into(), role: "user".into(), content: "hi".into(), created_at: 0, token_count: None, search_results: None, thinking_content: None, attachments: None },
            ];

            let client = test_client();
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(stream_chat(
                &client, &server.url(), "sk-test", "deepseek-chat", &std::collections::HashMap::new(),
                0.5, 1.0, &messages, CancellationToken::new(), |_| {}, |_| {},
            ));

            assert!(result.is_ok());
            mock.assert();
        });
    }

    #[test]
    fn custom_headers_are_sent() {
        run_with_mock(|mut server| {
            let sse_body = "data: [DONE]\n\n";
            let mock = server.mock("POST", "/chat/completions")
                .match_header("X-Custom", "test-value")
                .with_status(200)
                .with_header("content-type", "text/event-stream")
                .with_body(sse_body)
                .expect(1)
                .create();

            let messages = vec![];
            let client = test_client();
            let mut headers = std::collections::HashMap::new();
            headers.insert("X-Custom".to_string(), "test-value".to_string());

            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(stream_chat(
                &client, &server.url(), "sk-test", "gpt-4o", &headers,
                0.7, 1.0, &messages, CancellationToken::new(), |_| {}, |_| {},
            ));

            assert!(result.is_ok());
            mock.assert();
        });
    }
}
