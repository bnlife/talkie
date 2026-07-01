use serde::{Deserialize, Serialize};

/// A JSON-RPC 2.0 request sent to an MCP server.
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: i64,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// A JSON-RPC 2.0 response received from an MCP server.
#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub result: Option<serde_json::Value>,
    #[serde(default)]
    pub error: Option<JsonRpcError>,
}

/// A JSON-RPC 2.0 error object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}

/// A JSON-RPC 2.0 notification (no id, no response expected).
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// Tool definition returned by `tools/list`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, rename = "inputSchema")]
    pub input_schema: Option<serde_json::Value>,
}

/// Result content item from `tools/call`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolContent {
    #[serde(rename = "type")]
    pub content_type: String,
    #[serde(default)]
    pub text: Option<String>,
}

// ---------------------------------------------------------------------------
// Constructors
// ---------------------------------------------------------------------------

impl JsonRpcRequest {
    pub fn new(id: i64, method: &str, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            method: method.into(),
            params,
        }
    }

    /// Serialize to a JSON string with trailing newline (MCP stdio protocol).
    pub fn to_message(&self) -> String {
        let mut s = serde_json::to_string(self).unwrap_or_default();
        s.push('\n');
        s
    }
}

impl JsonRpcNotification {
    pub fn new(method: &str, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            method: method.into(),
            params,
        }
    }

    pub fn to_message(&self) -> String {
        let mut s = serde_json::to_string(self).unwrap_or_default();
        s.push('\n');
        s
    }
}

/// Parse a JSON-RPC response from a line of text.
pub fn parse_response(line: &str) -> Result<JsonRpcResponse, String> {
    serde_json::from_str(line).map_err(|e| format!("JSON-RPC 解析失败: {} | line={}", e, line))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_serialization() {
        let req = JsonRpcRequest::new(1, "initialize", Some(serde_json::json!({"capabilities": {}})));
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"id\":1"));
        assert!(json.contains("\"method\":\"initialize\""));
    }

    #[test]
    fn request_to_message_has_newline() {
        let req = JsonRpcRequest::new(1, "tools/list", None);
        let msg = req.to_message();
        assert!(msg.ends_with('\n'));
        assert!(msg.contains("\"method\":\"tools/list\""));
    }

    #[test]
    fn request_with_no_params_omits_field() {
        let req = JsonRpcRequest::new(2, "tools/list", None);
        let json = serde_json::to_string(&req).unwrap();
        assert!(!json.contains("params"));
    }

    #[test]
    fn response_deserialization_success() {
        let line = r#"{"jsonrpc":"2.0","id":1,"result":{"serverInfo":{"name":"test"}}}"#;
        let resp = parse_response(line).unwrap();
        assert_eq!(resp.id, Some(1));
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
    }

    #[test]
    fn response_deserialization_error() {
        let line = r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32600,"message":"Invalid Request"}}"#;
        let resp = parse_response(line).unwrap();
        assert_eq!(resp.id, Some(1));
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, -32600);
    }

    #[test]
    fn response_deserialization_notification() {
        // Notifications have no id
        let line = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
        let resp = parse_response(line).unwrap();
        assert!(resp.id.is_none());
    }

    #[test]
    fn notification_serialization() {
        let notif = JsonRpcNotification::new("notifications/initialized", None);
        let json = serde_json::to_string(&notif).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"notifications/initialized\""));
        assert!(!json.contains("id"));
    }

    #[test]
    fn tool_deserialization() {
        let json = r#"{"name":"mcp-bocha_search","description":"Search the web","inputSchema":{"type":"object"}}"#;
        let tool: McpTool = serde_json::from_str(json).unwrap();
        assert_eq!(tool.name, "mcp-bocha_search");
        assert!(tool.description.is_some());
        assert!(tool.input_schema.is_some());
    }

    #[test]
    fn tool_content_deserialization() {
        let json = r#"{"type":"text","text":"search results here"}"#;
        let content: McpToolContent = serde_json::from_str(json).unwrap();
        assert_eq!(content.content_type, "text");
        assert_eq!(content.text.unwrap(), "search results here");
    }

    #[test]
    fn parse_invalid_json_returns_error() {
        let result = parse_response("not json at all");
        assert!(result.is_err());
    }
}
