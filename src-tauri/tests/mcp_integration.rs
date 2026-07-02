//! Integration tests for MCP runtime (spawn, initialize, call_tool).

use std::path::PathBuf;
use talkie::mcp::pool::McpPool;
use talkie::models::McpInstance;

fn mock_instance(id: &str) -> McpInstance {
    McpInstance {
        id: id.to_string(),
        server_id: "mock-search".to_string(),
        name: "Mock Search".to_string(),
        enabled: true,
        transport: "stdio".to_string(),
        command: Some("node".to_string()),
        args: Some(vec![
            "tests/fixtures/mock-mcp-server.js".to_string(),
        ]),
        env: None,
        url: None,
        installed_at: 1000,
    }
}

fn test_pool() -> McpPool {
    McpPool::new(PathBuf::from("."))
}

#[tokio::test]
async fn test_mcp_spawn_initialize_list_tools() {
    let pool = test_pool();
    let inst = mock_instance("test-1");

    // Start the instance
    pool.start(&inst).await.expect("start should succeed");

    // List tools
    let tools = pool.list_tools("test-1").await.expect("list_tools should succeed");
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "mock_search");
    assert!(tools[0].description.is_some());

    // Stop
    pool.stop("test-1").await.expect("stop should succeed");
    assert!(!pool.is_running("test-1").await);
}

#[tokio::test]
async fn test_mcp_call_tool() {
    let pool = test_pool();
    let inst = mock_instance("test-2");

    pool.start(&inst).await.expect("start should succeed");

    let result = pool.call_tool(
        "test-2",
        "mock_search",
        serde_json::json!({"query": "hello world"}),
    ).await.expect("call_tool should succeed");

    // Parse the result
    let content = result.get("content").expect("result should have content");
    let arr = content.as_array().expect("content should be array");
    assert_eq!(arr.len(), 1);
    let text = arr[0].get("text").expect("content item should have text");
    let text_str = text.as_str().expect("text should be string");
    assert!(text_str.contains("hello world"));

    pool.stop("test-2").await.expect("stop should succeed");
}

#[tokio::test]
async fn test_mcp_double_start_fails() {
    let pool = test_pool();
    let inst = mock_instance("test-3");

    pool.start(&inst).await.expect("first start should succeed");
    let result = pool.start(&inst).await;
    assert!(result.is_err(), "second start should fail");

    pool.stop("test-3").await.expect("stop should succeed");
}

#[tokio::test]
async fn test_mcp_stop_non_running_is_ok() {
    let pool = test_pool();
    pool.stop("nonexistent").await.expect("stop non-running should succeed");
}

#[tokio::test]
async fn test_mcp_call_tool_on_stopped_instance_fails() {
    let pool = test_pool();
    let result = pool.call_tool("nonexistent", "tool", serde_json::json!({})).await;
    assert!(result.is_err());
}
