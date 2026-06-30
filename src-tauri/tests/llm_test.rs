//! Tests for SSE streaming parsing (the real `llm::stream_chat`).
//!
//! Uses `talkie::llm::stream_chat` with `tokio_util::sync::CancellationToken`
//! and `mockito` for HTTP mocking.

use std::sync::Arc;

use talkie::llm::stream_chat;
use talkie::models::Message;
use tokio_util::sync::CancellationToken;

fn empty_headers() -> std::collections::HashMap<String, String> {
    std::collections::HashMap::new()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// 1. **单行完整响应**
///
/// mockito returns a single SSE event with content *and* finish_reason,
/// followed by `data: [DONE]`.
#[tokio::test]
async fn test_single_chunk() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/chat/completions")
        .with_status(200)
        .with_header("Content-Type", "text/event-stream")
        .with_body(
            "data: {\"choices\":[{\"delta\":{\"content\":\"你好\"}},{\"finish_reason\":\"stop\"}]}\n\
             data: [DONE]\n",
        )
        .create_async()
        .await;

    let cancel = CancellationToken::new();

    // Shared buffer to assert on_chunk invocations
    let chunks = Arc::new(std::sync::Mutex::new(Vec::new()));
    let chunks_clone = chunks.clone();

    let messages = vec![Message {
        id: "test-1".into(),
        conversation_id: "conv-1".into(),
        role: "user".into(),
        content: "你好".into(),
        created_at: 1000,
        token_count: None,
    }];

    let result = stream_chat(
        &server.url(),
        "test-key",
        "test-model",
        &empty_headers(),
        0.7,
        1.0,
        &messages,
        cancel,
        move |chunk| {
            chunks_clone.lock().unwrap().push(chunk);
        },
    )
    .await;

    // Verify return value
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "你好");

    // Verify on_chunk was called exactly once with the correct content
    let captured = chunks.lock().unwrap();
    assert_eq!(captured.len(), 1, "on_chunk should be called 1 time");
    assert_eq!(captured[0], "你好");

    mock.assert_async().await;
}

/// 2. **多行分块流式**
///
/// mockito returns several SSE events, each carrying a fragment of the full
/// response.  on_chunk should be invoked for every non‑empty delta, and the
/// final return value should be the concatenation of all deltas.
#[tokio::test]
async fn test_multi_chunk_stream() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/chat/completions")
        .with_status(200)
        .with_header("Content-Type", "text/event-stream")
        .with_body(
            "data: {\"choices\":[{\"delta\":{\"content\":\"你好\"}}]}\n\
             data: {\"choices\":[{\"delta\":{\"content\":\"世界\"}}]}\n\
             data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\
             data: [DONE]\n",
        )
        .create_async()
        .await;

    let cancel = CancellationToken::new();

    let chunks = Arc::new(std::sync::Mutex::new(Vec::new()));
    let chunks_clone = chunks.clone();

    let messages = vec![Message {
        id: "test-2".into(),
        conversation_id: "conv-1".into(),
        role: "user".into(),
        content: "你好世界".into(),
        created_at: 1000,
        token_count: None,
    }];

    let result = stream_chat(
        &server.url(),
        "test-key",
        "test-model",
        &empty_headers(),
        0.7,
        1.0,
        &messages,
        cancel,
        move |chunk| {
            chunks_clone.lock().unwrap().push(chunk);
        },
    )
    .await;

    // Final accumulated text
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "你好世界");

    // Per‑delta callbacks
    let captured = chunks.lock().unwrap();
    assert_eq!(captured.len(), 2, "on_chunk should be called 2 times");
    assert_eq!(captured[0], "你好");
    assert_eq!(captured[1], "世界");

    mock.assert_async().await;
}

/// 3. **提前取消**
///
/// Create a pre‑cancelled `CancellationToken` and verify that `stream_chat`
/// returns `Err` immediately **without** issuing any HTTP request.
#[tokio::test]
async fn test_cancel_before_request() {
    let cancel = CancellationToken::new();
    cancel.cancel();

    let messages = vec![Message {
        id: "test-3".into(),
        conversation_id: "conv-1".into(),
        role: "user".into(),
        content: "hi".into(),
        created_at: 1000,
        token_count: None,
    }];

    // Use a deliberately broken URL — the cancellation check runs first, so
    // no connection attempt is ever made.
    let result = stream_chat(
        "http://localhost:0",
        "key",
        "model",
        &empty_headers(),
        0.7,
        1.0,
        &messages,
        cancel,
        |_| {},
    )
    .await;

    assert!(result.is_err(), "Expected Err for cancelled request");
}

/// 4. **HTTP 错误**
///
/// Server responds with a non‑2xx status code (401, 500).  `stream_chat`
/// should return `Err` with a message that includes the status code.
#[tokio::test]
async fn test_http_error_401() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/chat/completions")
        .with_status(401)
        .with_body(r#"{"error":"invalid api key"}"#)
        .create_async()
        .await;

    let messages = vec![Message {
        id: "test-4".into(),
        conversation_id: "conv-1".into(),
        role: "user".into(),
        content: "hi".into(),
        created_at: 1000,
        token_count: None,
    }];

    let result = stream_chat(
        &server.url(),
        "bad-key",
        "model",
        &empty_headers(),
        0.7,
        1.0,
        &messages,
        CancellationToken::new(),
        |_| {},
    )
    .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("401"),
        "Error message should mention HTTP 401, got: {}",
        err
    );

    mock.assert_async().await;
}

#[tokio::test]
async fn test_http_error_500() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/chat/completions")
        .with_status(500)
        .with_body(r#"{"error":"internal error"}"#)
        .create_async()
        .await;

    let messages = vec![Message {
        id: "test-5".into(),
        conversation_id: "conv-1".into(),
        role: "user".into(),
        content: "hi".into(),
        created_at: 1000,
        token_count: None,
    }];

    let result = stream_chat(
        &server.url(),
        "key",
        "model",
        &empty_headers(),
        0.7,
        1.0,
        &messages,
        CancellationToken::new(),
        |_| {},
    )
    .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("500"),
        "Error message should mention HTTP 500, got: {}",
        err
    );

    mock.assert_async().await;
}

/// 5. **SSE 格式错误**
///
/// The server returns a mix of valid and malformed SSE lines.  The parser
/// should gracefully skip the malformed parts and extract whatever valid
/// content is present.
#[tokio::test]
async fn test_malformed_sse() {
    let mut server = mockito::Server::new_async().await;
    let body = "\
data: not-valid-json
data: {\"choices\":[{\"delta\":{\"content\":\"部分有效\"}}]}
: this is an SSE comment (should be ignored)
data: [DONE]
";
    let mock = server
        .mock("POST", "/chat/completions")
        .with_status(200)
        .with_header("Content-Type", "text/event-stream")
        .with_body(body)
        .create_async()
        .await;

    let chunks = Arc::new(std::sync::Mutex::new(Vec::new()));
    let chunks_clone = chunks.clone();

    let messages = vec![Message {
        id: "test-6".into(),
        conversation_id: "conv-1".into(),
        role: "user".into(),
        content: "测试".into(),
        created_at: 1000,
        token_count: None,
    }];

    let result = stream_chat(
        &server.url(),
        "key",
        "model",
        &empty_headers(),
        0.7,
        1.0,
        &messages,
        CancellationToken::new(),
        move |chunk| {
            chunks_clone.lock().unwrap().push(chunk);
        },
    )
    .await;

    // Even with malformed lines, valid content must be extracted
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "部分有效");

    let captured = chunks.lock().unwrap();
    assert_eq!(captured.len(), 1);
    assert_eq!(captured[0], "部分有效");

    mock.assert_async().await;
}
