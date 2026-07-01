//! Integration tests for chat commands (`send_message`, `stop_stream`).
//!
//! These tests verify the behavior of underlying components without depending
//! on a Tauri runtime environment. Since `State` and `AppHandle` cannot be
//! constructed in unit tests, we test the building blocks directly:
//!
//! - **Store layer**: message persistence via `store::create_message` and
//!   `store::list_messages_by_conversation`.
//! - **Cancellation token**: `CancellationToken` creation, cancellation, and
//!   state inspection.
//! - **AppState interaction**: token storage and retrieval via a
//!   `Mutex<Option<CancellationToken>>`, simulating the `cancel` field on
//!   `AppState`.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio_util::sync::CancellationToken;

use talkie::models::{Conversation, ConversationConfig, Message};
use talkie::store;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create an in-memory SQLite database with all tables initialised.
fn setup_db() -> rusqlite::Connection {
    store::init(&PathBuf::from(":memory:"))
        .expect("failed to initialise in-memory database")
}

/// Insert a minimal conversation so that messages referencing it pass the FK
/// constraint.
fn insert_conv(conn: &rusqlite::Connection, id: &str) {
    let conv = Conversation {
        id: id.to_string(),
        title: "Test Conversation".into(),
        created_at: 1000,
        updated_at: 1000,
        pinned: false,
    };
    let config = ConversationConfig {
        conversation_id: id.to_string(),
        provider_id: "prov-1".into(),
        model: "test-model".into(),
        prompt_id: None,
        search_enabled: false,
        search_engine: String::new(),
    };
    store::create_conversation(conn, &conv, &config).unwrap();
}

// ===========================================================================
// Store-layer tests
//
// These verify that `send_message`'s first step — persisting the user message
// to SQLite — works correctly.
// ===========================================================================

/// A single user message can be saved and later retrieved.
#[test]
fn test_store_save_user_message() {
    let conn = setup_db();
    insert_conv(&conn, "conv-1");

    let msg = Message {
        id: "msg-1".into(),
        conversation_id: "conv-1".into(),
        role: "user".into(),
        content: "你好，世界！".into(),
        created_at: 1001,
        token_count: None,
        search_results: None,
    };
    store::create_message(&conn, &msg).unwrap();

    let messages = store::list_messages_by_conversation(&conn, "conv-1").unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].role, "user");
    assert_eq!(messages[0].content, "你好，世界！");
    assert_eq!(messages[0].conversation_id, "conv-1");
}

/// Messages are returned in chronological order (ascending `created_at`).
/// This is important because the LLM needs the conversation history in the
/// correct order when building the prompt.
#[test]
fn test_store_messages_chronological_order() {
    let conn = setup_db();
    insert_conv(&conn, "conv-order");

    for i in 0..5 {
        let msg = Message {
            id: format!("msg-{}", i),
            conversation_id: "conv-order".into(),
            role: if i % 2 == 0 { "user".into() } else { "assistant".into() },
            content: format!("Message {}", i),
            created_at: 1000 + i,
            token_count: Some(i * 10),
            search_results: None,
        };
        store::create_message(&conn, &msg).unwrap();
    }

    let messages = store::list_messages_by_conversation(&conn, "conv-order").unwrap();
    assert_eq!(messages.len(), 5);
    for (idx, m) in messages.iter().enumerate() {
        assert_eq!(m.created_at, 1000 + idx as i64, "messages should be in chronological order");
    }
}

/// A conversation with no messages returns an empty list (not an error).
#[test]
fn test_store_empty_conversation_returns_empty_list() {
    let conn = setup_db();
    insert_conv(&conn, "conv-empty");

    let messages = store::list_messages_by_conversation(&conn, "conv-empty").unwrap();
    assert!(messages.is_empty());
}

/// Multiple messages can be inserted in a single batch transaction.
/// `send_message` may use this when persisting the assistant response
/// alongside the user message.
#[test]
fn test_store_batch_create_messages() {
    let conn = setup_db();
    insert_conv(&conn, "conv-batch");

    let msgs: Vec<Message> = (0..3)
        .map(|i| Message {
            id: format!("bmsg-{}", i),
            conversation_id: "conv-batch".into(),
            role: "user".into(),
            content: format!("Batch message {}", i),
            created_at: 2000 + i,
            token_count: None,
            search_results: None,
        })
        .collect();

    store::batch_create_messages(&conn, &msgs).unwrap();
    let stored = store::list_messages_by_conversation(&conn, "conv-batch").unwrap();
    assert_eq!(stored.len(), 3);
}

/// Deleting all messages for a conversation mirrors what happens when a user
/// clears the chat or when the conversation itself is deleted (cascade).
#[test]
fn test_store_delete_messages_by_conversation() {
    let conn = setup_db();
    insert_conv(&conn, "conv-del");

    let msg = Message {
        id: "dmsg-1".into(),
        conversation_id: "conv-del".into(),
        role: "user".into(),
        content: "Will be deleted".into(),
        created_at: 3000,
        token_count: None,
        search_results: None,
    };
    store::create_message(&conn, &msg).unwrap();

    assert_eq!(
        store::list_messages_by_conversation(&conn, "conv-del")
            .unwrap()
            .len(),
        1
    );

    store::delete_messages_by_conversation(&conn, "conv-del").unwrap();
    assert_eq!(
        store::list_messages_by_conversation(&conn, "conv-del")
            .unwrap()
            .len(),
        0
    );
}

/// Verify that `token_count` is correctly round-tripped through the database.
#[test]
fn test_store_message_with_token_count() {
    let conn = setup_db();
    insert_conv(&conn, "conv-tokens");

    let msg = Message {
        id: "tmsg-1".into(),
        conversation_id: "conv-tokens".into(),
        role: "assistant".into(),
        content: "Response with token tracking".into(),
        created_at: 4000,
        token_count: Some(42),
        search_results: None,
    };
    store::create_message(&conn, &msg).unwrap();

    let messages = store::list_messages_by_conversation(&conn, "conv-tokens").unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].token_count, Some(42));
}

// ===========================================================================
// Pin / Unpin tests
// ===========================================================================

/// Pinning a conversation sets pinned to 1.
#[test]
fn test_store_pin_conversation() {
    let conn = setup_db();
    insert_conv(&conn, "conv-pin");

    store::pin_conversation(&conn, "conv-pin").unwrap();

    let conv = store::get_conversation(&conn, "conv-pin")
        .unwrap()
        .expect("conversation should exist");
    assert!(conv.pinned, "conversation should be pinned");
}

/// Unpinning a conversation sets pinned to 0.
#[test]
fn test_store_unpin_conversation() {
    let conn = setup_db();
    insert_conv(&conn, "conv-unpin");

    // Pin first, then unpin.
    store::pin_conversation(&conn, "conv-unpin").unwrap();
    store::unpin_conversation(&conn, "conv-unpin").unwrap();

    let conv = store::get_conversation(&conn, "conv-unpin")
        .unwrap()
        .expect("conversation should exist");
    assert!(!conv.pinned, "conversation should not be pinned");
}

/// Pinned conversations appear before unpinned ones in list_conversations.
#[test]
fn test_store_list_orders_pinned_first() {
    let conn = setup_db();

    let now = 2000i64;
    for i in 0..3 {
        let conv = Conversation {
            id: format!("conv-{}", i),
            title: format!("Conversation {}", i),
            created_at: now + i,
            updated_at: now + i,
            pinned: false,
        };
        let config = ConversationConfig {
            conversation_id: format!("conv-{}", i),
            provider_id: "prov-1".into(),
            model: "test".into(),
            prompt_id: None,
            search_enabled: false,
            search_engine: String::new(),
        };
        store::create_conversation(&conn, &conv, &config).unwrap();
    }

    // Pin conv-1 (middle one)
    store::pin_conversation(&conn, "conv-1").unwrap();

    let list = store::list_conversations(&conn).unwrap();
    assert_eq!(list.len(), 3);
    // conv-1 (pinned) should be first
    assert_eq!(list[0].id, "conv-1");
    assert!(list[0].pinned);
    // remaining should be in updated_at DESC order
    assert_eq!(list[1].id, "conv-2");
    assert_eq!(list[2].id, "conv-0");
}

// ===========================================================================
// Cancellation token tests
//
// `send_message` creates a `CancellationToken` and stores it in AppState;
// `stop_stream` retrieves it and calls `.cancel()`. These tests verify the
// token lifecycle in isolation.
// ===========================================================================

/// A freshly created token should not be in the cancelled state.
#[test]
fn test_cancellation_token_fresh_not_cancelled() {
    let token = CancellationToken::new();
    assert!(!token.is_cancelled(), "a new token should not be cancelled");
}

/// Calling `cancel()` transitions the token to the cancelled state.
#[test]
fn test_cancellation_token_cancel_changes_state() {
    let token = CancellationToken::new();
    assert!(!token.is_cancelled());
    token.cancel();
    assert!(token.is_cancelled(), "token must be cancelled after cancel()");
}

/// Calling `cancel()` multiple times is idempotent and must not panic.
#[test]
fn test_cancellation_token_double_cancel_is_idempotent() {
    let token = CancellationToken::new();
    token.cancel();
    token.cancel(); // should not panic
    assert!(token.is_cancelled());
}

/// A cancelled token's `cancelled()` future resolves immediately.
#[tokio::test]
async fn test_cancellation_token_cancelled_future_ready() {
    let token = CancellationToken::new();
    token.cancel();

    tokio::time::timeout(Duration::from_secs(1), token.cancelled())
        .await
        .expect("cancelled() future should resolve immediately when token is already cancelled");
}

/// Cross-task cancellation: one task waits for cancellation, another triggers
/// it. This simulates the real flow where `send_message` runs the streaming
/// loop on a spawned task and `stop_stream` cancels from the command handler.
#[tokio::test]
async fn test_cancellation_token_async_cancel_signals_other_task() {
    let token = CancellationToken::new();
    let token_clone = token.clone();

    let handle = tokio::spawn(async move {
        token_clone.cancelled().await;
        true
    });

    // Give the spawned task time to start waiting on `cancelled()`.
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Cancel from the test task — the spawned task should wake up.
    token.cancel();

    let result = tokio::time::timeout(Duration::from_secs(2), handle)
        .await
        .expect("spawned task should complete within timeout")
        .expect("spawned task should not panic");
    assert!(result, "spawned task should complete after cancellation");
}

/// A child token inherits cancellation from its parent. This is useful if
/// `send_message` ever needs hierarchical cancellation (e.g. per-request
/// child tokens under a global shutdown token).
#[test]
fn test_cancellation_token_child_inherits_parent_cancel() {
    let parent = CancellationToken::new();
    let child = parent.child_token();

    assert!(!child.is_cancelled(), "child should not be cancelled initially");
    parent.cancel();
    assert!(
        child.is_cancelled(),
        "child token must be cancelled when parent is cancelled"
    );
}

// ===========================================================================
// AppState interaction tests
//
// These simulate how `send_message` and `stop_stream` interact with the
// `cancel` field on `AppState` — a `Mutex<Option<CancellationToken>>`.
// ===========================================================================

/// Initially the token slot is empty.
#[test]
fn test_appstate_cancel_slot_starts_empty() {
    let slot: Mutex<Option<CancellationToken>> = Mutex::new(None);
    assert!(slot.lock().unwrap().is_none());
}

/// `send_message` stores a token; it can later be retrieved.
#[test]
fn test_appstate_store_and_retrieve_token() {
    let slot: Mutex<Option<CancellationToken>> = Mutex::new(None);

    // Simulate send_message creating and storing a token.
    let token = CancellationToken::new();
    *slot.lock().unwrap() = Some(token.clone());

    // The slot should now contain a token.
    assert!(slot.lock().unwrap().is_some());

    // Simulate stop_stream taking the token out and cancelling it.
    let taken = slot.lock().unwrap().take();
    assert!(taken.is_some(), "token should be retrievable");
    assert!(slot.lock().unwrap().is_none(), "slot should be empty after take()");

    taken.unwrap().cancel();
    assert!(token.is_cancelled(), "retrieved token should be cancellable");
}

/// `stop_stream` must clear the slot after cancelling so that a second call
/// is a no-op (or knows there's nothing to cancel).
#[test]
fn test_appstate_clear_after_cancel() {
    let slot: Mutex<Option<CancellationToken>> = Mutex::new(None);

    // send_message stores token.
    let token = CancellationToken::new();
    *slot.lock().unwrap() = Some(token.clone());

    // stop_stream: take, cancel, clear.
    if let Some(t) = slot.lock().unwrap().take() {
        t.cancel();
    }
    // Slot must now be empty.
    assert!(slot.lock().unwrap().is_none());
    assert!(token.is_cancelled());
}

/// When a new `send_message` is issued while a previous stream is still
/// running, the old token should be cancelled (replaced). This prevents
/// duplicate / stale streams.
#[test]
fn test_appstate_new_message_replaces_old_token() {
    let slot: Mutex<Option<CancellationToken>> = Mutex::new(None);

    let old_token = CancellationToken::new();
    *slot.lock().unwrap() = Some(old_token.clone());

    // A second send_message comes in while the first stream is still active.
    let new_token = CancellationToken::new();
    if let Some(previous) = slot.lock().unwrap().replace(new_token.clone()) {
        previous.cancel();
    }

    assert!(
        old_token.is_cancelled(),
        "the previous stream's token should be cancelled"
    );
    assert!(
        !new_token.is_cancelled(),
        "the new stream's token should remain active"
    );
    // The slot holds the new token.
    assert!(slot.lock().unwrap().is_some());
}

/// Calling `stop_stream` when there is no active token must not panic.
#[test]
fn test_appstate_stop_stream_when_idle() {
    let slot: Mutex<Option<CancellationToken>> = Mutex::new(None);

    // No token stored — calling "stop" should be a safe no-op.
    let token = slot.lock().unwrap().take();
    assert!(token.is_none(), "take() on empty slot should return None");
}

/// Full round-trip: send_message (store token) → stop_stream (cancel + clear)
/// → slot is empty and token is cancelled.
#[test]
fn test_appstate_full_lifecycle() {
    let slot: Mutex<Option<CancellationToken>> = Mutex::new(None);

    // 1. send_message creates token.
    let original = CancellationToken::new();
    *slot.lock().unwrap() = Some(original.clone());
    assert!(slot.lock().unwrap().is_some());
    assert!(!original.is_cancelled());

    // 2. stop_stream takes and cancels.
    let taken = slot.lock().unwrap().take().expect("token must be present");
    taken.cancel();
    assert!(slot.lock().unwrap().is_none());

    // 3. Verify the original reference reflects the cancelled state.
    assert!(original.is_cancelled());

    // 4. Another stop_stream is safe.
    assert!(slot.lock().unwrap().take().is_none());
}

// ===========================================================================
// Real AppState construction test
//
// Verify that the `cancel` field compiles and works when `AppState` is
// constructed directly (outside the Tauri runtime).
// ===========================================================================

/// Construct a real `AppState` in an in-memory configuration and verify the
/// cancel slot behaves identically to the isolated pattern tests above.
#[test]
fn test_real_appstate_cancel_field() {
    let conn = store::init(&PathBuf::from(":memory:")).unwrap();
    let state = talkie::AppState {
        db: Mutex::new(conn),
        config: Mutex::new(talkie::models::Settings::default()),
        config_path: PathBuf::from(":memory:"),
        cancel: Mutex::new(None),
        mcp_pool: Arc::new(talkie::mcp::pool::McpPool::new(PathBuf::from(":memory:"))),
    };

    // Slot starts empty.
    assert!(state.cancel.lock().unwrap().is_none());

    // Simulate send_message: create and store a token.
    let token = CancellationToken::new();
    *state.cancel.lock().unwrap() = Some(token.clone());
    assert!(state.cancel.lock().unwrap().is_some());
    assert!(!token.is_cancelled());

    // Simulate stop_stream: take, cancel, clear.
    let taken = state.cancel.lock().unwrap().take().unwrap();
    taken.cancel();
    assert!(state.cancel.lock().unwrap().is_none());
    assert!(token.is_cancelled());
}

// ===========================================================================
// parse_search_results tests
// ===========================================================================

#[test]
fn test_parse_search_results_bocha_format() {
    // Simulate Bocha MCP response: text content with markdown-formatted results
    let mcp_response = serde_json::json!({
        "content": [{
            "type": "text",
            "text": "Search results for \"Rust 编程\":\n\n1. [Rust 语言官网](https://www.rust-lang.org)\n   Rust 是一门系统编程语言\n   Published: 2024-01-01\n\n2. [Rust 程序设计语言](https://kaisery.github.io/trpl-zh-cn)\n   Rust 程序设计语言中文翻译\n   Published: 2023-06-15"
        }]
    });

    let results = talkie::commands::chat::parse_search_results(&mcp_response);
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].title, "Rust 语言官网");
    assert_eq!(results[0].url, "https://www.rust-lang.org");
    assert_eq!(results[0].snippet, Some("Rust 是一门系统编程语言".into()));
    assert_eq!(results[1].title, "Rust 程序设计语言");
    assert_eq!(results[1].url, "https://kaisery.github.io/trpl-zh-cn");
    assert_eq!(results[1].snippet, Some("Rust 程序设计语言中文翻译".into()));
}

#[test]
fn test_parse_search_results_no_snippet() {
    let mcp_response = serde_json::json!({
        "content": [{
            "type": "text",
            "text": "Search results:\n\n1. [Example](https://example.com)\n\n2. [Another](https://another.com)"
        }]
    });

    let results = talkie::commands::chat::parse_search_results(&mcp_response);
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].title, "Example");
    assert_eq!(results[0].url, "https://example.com");
    assert_eq!(results[0].snippet, None);
}

#[test]
fn test_parse_search_results_empty() {
    let mcp_response = serde_json::json!({
        "content": [{
            "type": "text",
            "text": "No results found"
        }]
    });

    let results = talkie::commands::chat::parse_search_results(&mcp_response);
    assert_eq!(results.len(), 0);
}

#[test]
fn test_parse_search_results_fallback_json() {
    // Fallback: no text content, use raw JSON
    let mcp_response = serde_json::json!({
        "content": [{
            "type": "text",
            "text": "Some unexpected format"
        }]
    });

    let results = talkie::commands::chat::parse_search_results(&mcp_response);
    assert_eq!(results.len(), 0);
}
