//! End-to-end tests for search: DB → perform_search → MCP → parse → persist.
//!
//! These tests simulate the full user search flow:
//! 1. User has a conversation with search_enabled=true
//! 2. A bocha-search MCP instance is installed and enabled
//! 3. User sends a message → perform_search is called
//! 4. MCP instance auto-starts, calls tool, returns results
//! 5. Results are parsed into structured data
//! 6. Results are persisted with the assistant message

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use talkie::chat::search::perform_search;
use talkie::mcp::pool::McpPool;
use talkie::models::{Conversation, ConversationConfig, McpInstance};
use talkie::store;

fn setup_state() -> talkie::AppState {
    let conn = store::init(&PathBuf::from(":memory:")).unwrap();
    talkie::AppState {
        db: Mutex::new(conn),
        config: Mutex::new(talkie::models::Settings::default()),
        config_path: PathBuf::from(":memory:"),
        cancel: Mutex::new(None),
        mcp_pool: Arc::new(McpPool::new(PathBuf::from("."))),
    }
}

fn insert_bocha_instance(state: &talkie::AppState) {
    let inst = McpInstance {
        id: "inst-bocha".to_string(),
        server_id: "bocha-search".to_string(),
        name: "Bocha Search".to_string(),
        enabled: true,
        transport: "stdio".to_string(),
        command: Some("node".to_string()),
        args: Some(vec!["tests/fixtures/bocha-mcp-server.js".to_string()]),
        env: None,
        url: None,
        installed_at: 1000,
    };
    let db = state.db.lock().unwrap();
    store::create_mcp_instance(&db, &inst).unwrap();
}

fn insert_conversation(state: &talkie::AppState, conv_id: &str) {
    let conv = Conversation {
        id: conv_id.to_string(),
        title: "Test".into(),
        created_at: 1000,
        updated_at: 1000,
        pinned: false,
    };
    let config = ConversationConfig {
        conversation_id: conv_id.to_string(),
        provider_id: "prov-1".into(),
        model: "test".into(),
        prompt_id: None,
        search_enabled: true,
        search_engine: String::new(),
    };
    let db = state.db.lock().unwrap();
    store::create_conversation(&db, &conv, &config).unwrap();
}

// ===========================================================================
// E2E Test 1: Full search flow — DB → perform_search → MCP → parse
// ===========================================================================

#[test]
fn test_search_e2e_full_flow() {
    let state = setup_state();
    insert_conversation(&state, "conv-1");
    insert_bocha_instance(&state);

    // perform_search: find instance in DB → auto-start MCP → call_tool → parse
    let (text, results) = perform_search(&state, "test query").expect("search should succeed");

    // Verify text for LLM
    assert!(text.contains("Search results for"), "text should contain header, got: {}", text);
    assert!(text.contains("Example Domain"), "text should contain result title");

    // Verify structured results
    assert_eq!(results.len(), 3, "should parse 3 results from MCP response");

    assert_eq!(results[0].title, "Example Domain");
    assert_eq!(results[0].url, "https://example.com");
    assert_eq!(results[0].snippet, Some("This is an example domain for testing".to_string()));

    assert_eq!(results[1].title, "Rust 官网");
    assert_eq!(results[1].url, "https://www.rust-lang.org");
    assert_eq!(results[1].snippet, Some("Rust 是一门系统编程语言".to_string()));

    assert_eq!(results[2].title, "GitHub");
    assert_eq!(results[2].url, "https://github.com");
    assert_eq!(results[2].snippet, Some("Where developers build software".to_string()));

    // Verify MCP is now running in pool
    assert!(state.mcp_pool.is_running("inst-bocha"), "MCP should be auto-started");
}

// ===========================================================================
// E2E Test 2: Search + persist to DB
// ===========================================================================

#[test]
fn test_search_e2e_persist_results() {
    let state = setup_state();
    insert_conversation(&state, "conv-2");
    insert_bocha_instance(&state);

    let (text, results) = perform_search(&state, "rust lang").expect("search should succeed");

    // Simulate what finalize_response does: create assistant message with search_results
    let msg = talkie::models::Message {
        id: "msg-1".to_string(),
        conversation_id: "conv-2".to_string(),
        role: "assistant".to_string(),
        content: format!("根据搜索结果：\n\n{}", text),
        created_at: 2000,
        token_count: Some(100),
        search_results: Some(results),
    };

    {
        let db = state.db.lock().unwrap();
        store::create_message(&db, &msg).unwrap();
    }

    // Read back and verify
    let db = state.db.lock().unwrap();
    let messages = store::list_messages_by_conversation(&db, "conv-2").unwrap();
    assert_eq!(messages.len(), 1);

    let sr = messages[0].search_results.as_ref().expect("should have search_results");
    assert_eq!(sr.len(), 3);
    assert_eq!(sr[0].title, "Example Domain");
    assert_eq!(sr[1].title, "Rust 官网");
    assert_eq!(sr[2].title, "GitHub");
}

// ===========================================================================
// E2E Test 3: No search instance → error
// ===========================================================================

#[test]
fn test_search_e2e_no_instance() {
    let state = setup_state();
    insert_conversation(&state, "conv-3");
    // Don't insert any MCP instance

    let result = perform_search(&state, "test query");
    assert!(result.is_err(), "should fail when no search instance");
    let err = result.unwrap_err();
    assert!(err.contains("没有启用的搜索 MCP 实例"), "error should mention no instance, got: {}", err);
}

// ===========================================================================
// E2E Test 4: Auto-start MCP when instance is enabled but not running
// ===========================================================================

#[test]
fn test_search_e2e_auto_start() {
    let state = setup_state();
    insert_conversation(&state, "conv-4");
    insert_bocha_instance(&state);

    // Verify MCP is NOT running before search
    assert!(!state.mcp_pool.is_running("inst-bocha"), "MCP should not be running before search");

    let (text, results) = perform_search(&state, "auto start test").expect("search should succeed");

    // Verify MCP WAS auto-started
    assert!(state.mcp_pool.is_running("inst-bocha"), "MCP should be auto-started after search");

    // Verify results
    assert!(results.len() > 0, "should have results");
    assert!(text.contains("Search results for"), "text should contain header");

    // Stop and verify
    state.mcp_pool.stop("inst-bocha").unwrap();
    assert!(!state.mcp_pool.is_running("inst-bocha"), "MCP should be stopped");
}

// ===========================================================================
// E2E Test 5: Empty search results
// ===========================================================================

#[test]
fn test_search_e2e_empty_results() {
    let state = setup_state();
    insert_conversation(&state, "conv-5");

    // Insert empty-results fixture
    let inst = McpInstance {
        id: "inst-empty".to_string(),
        server_id: "bocha-search-empty".to_string(),
        name: "Bocha Empty".to_string(),
        enabled: true,
        transport: "stdio".to_string(),
        command: Some("node".to_string()),
        args: Some(vec!["tests/fixtures/bocha-mcp-server-empty.js".to_string()]),
        env: None,
        url: None,
        installed_at: 1000,
    };
    let db = state.db.lock().unwrap();
    store::create_mcp_instance(&db, &inst).unwrap();
    drop(db);

    let (text, results) = perform_search(&state, "nonexistent query").expect("search should succeed even with no results");

    // Empty results: parse_search_results returns 0 items, text contains "No results"
    assert_eq!(results.len(), 0, "empty fixture should return 0 results");
    assert!(text.contains("No results found"), "text should say no results, got: {}", text);
}
