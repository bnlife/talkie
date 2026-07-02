/// Integration tests for `verify_connection` in `commands/settings.rs`.
///
/// Uses mockito to simulate the LLM API server, and a raw TCP listener
/// to simulate a network timeout.
///
/// Note: mockito's sync `Server::new()` cannot be used inside `#[tokio::test]`
/// because it tries to create a nested runtime. Always use the `_async` variants.

use reqwest::Client;

/// Create a shared HTTP client for tests.
fn test_client() -> Client {
    Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap()
}

/// Test that a 200 response from the API returns `Ok("连接成功")`.
#[tokio::test]
async fn test_connection_success() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/chat/completions")
        .with_status(200)
        .with_body(r#"{"id":"test"}"#)
        .create_async()
        .await;

    let client = test_client();
    let headers = std::collections::HashMap::new();
    let result =
        talkie::commands::settings::verify_connection(&client, &server.url(), "test-key", "test-model", &headers)
            .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "连接成功");
    mock.assert_async().await;
}

/// Test that a 401 response returns an error containing "API Key" or "认证".
#[tokio::test]
async fn test_connection_unauthorized() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/chat/completions")
        .with_status(401)
        .with_body(r#"{"error":"unauthorized"}"#)
        .create_async()
        .await;

    let client = test_client();
    let headers = std::collections::HashMap::new();
    let result =
        talkie::commands::settings::verify_connection(&client, &server.url(), "invalid-key", "test-model", &headers)
            .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("API Key") || err.contains("认证"),
        "Expected error about API key / 认证, got: {}",
        err
    );
    mock.assert_async().await;
}

/// Test that a 404 response returns an error containing "地址" or "URL".
#[tokio::test]
async fn test_connection_not_found() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/chat/completions")
        .with_status(404)
        .with_body(r#"{"error":"not found"}"#)
        .create_async()
        .await;

    let client = test_client();
    let headers = std::collections::HashMap::new();
    let result =
        talkie::commands::settings::verify_connection(&client, &server.url(), "test-key", "test-model", &headers)
            .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("地址") || err.contains("URL"),
        "Expected error about 地址 / URL, got: {}",
        err
    );
    mock.assert_async().await;
}

/// Test that a connection which accepts TCP but never sends an HTTP response
/// returns an error containing "超时" or "timeout".
#[tokio::test]
async fn test_connection_timeout() {
    // Start a raw TCP listener on a random port. It accepts one connection
    // but holds it open without sending any data, simulating a dead server.
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let base_url = format!("http://{}", addr);

    // Accept the connection and hold it for longer than the client's 10s timeout
    std::thread::spawn(move || {
        if let Ok((_stream, _)) = listener.accept() {
            std::thread::sleep(std::time::Duration::from_secs(20));
        }
    });

    // Brief delay so the listener thread is ready
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // Wrap the call in a top-level timeout so the test never hangs indefinitely
    let client = test_client();
    let headers = std::collections::HashMap::new();
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(25),
        talkie::commands::settings::verify_connection(&client, &base_url, "test-key", "test-model", &headers),
    )
    .await;

    match result {
        Ok(Err(err)) => {
            assert!(
                err.contains("超时") || err.contains("timeout"),
                "Expected error about 超时 / timeout, got: {}",
                err
            );
        }
        Ok(Ok(ok)) => panic!("Expected timeout error, got Ok({})", ok),
        Err(_elapsed) => {
            panic!(
                "verify_connection did not complete within 25s (timeout test failed to trigger)"
            )
        }
    }
}
