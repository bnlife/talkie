use serde::{Deserialize, Serialize};

/// A single chat message within a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    /// "user", "assistant", or "system"
    pub role: String,
    pub content: String,
    pub created_at: i64,
    pub token_count: Option<i64>,
}

/// A model provider (e.g. OpenAI, DeepSeek, Ollama).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProvider {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub icon: Option<String>,
    pub base_url: String,
    pub api_key: String,
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub models: Vec<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

/// Core conversation identity (title, pinned, timestamps).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub pinned: bool,
}

/// Per-conversation configuration (provider, model, prompt, search).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationConfig {
    pub conversation_id: String,
    #[serde(default)]
    pub provider_id: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub prompt_id: Option<String>,
    #[serde(default)]
    pub search_enabled: bool,
}

/// Merged view of Conversation + ConversationConfig (returned to frontend).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationView {
    // Conversation fields
    pub id: String,
    pub title: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub pinned: bool,
    // Config fields
    #[serde(default)]
    pub provider_id: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub prompt_id: Option<String>,
    #[serde(default)]
    pub search_enabled: bool,
}

/// Application settings persisted in a JSON config file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub providers: Vec<ModelProvider>,
    #[serde(default)]
    pub active_provider_id: String,
    pub temperature: f32,
    #[serde(default = "default_top_p")]
    pub top_p: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_active_conversation_id: Option<String>,
}

fn default_top_p() -> f32 {
    1.0
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            providers: Vec::new(),
            active_provider_id: String::new(),
            temperature: 0.7,
            top_p: 1.0,
            last_active_conversation_id: None,
        }
    }
}

/// A prompt template for system prompts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub id: String,
    pub name: String,
    pub content: String,
    pub is_default: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

// ---------------------------------------------------------------------------
// MCP models
// ---------------------------------------------------------------------------

/// A market category grouping MCP servers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpCategory {
    pub id: String,
    pub name: String,
    pub icon: String,
}

/// Environment variable definition for an MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpEnvVar {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub secret: bool,
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub choices: Option<Vec<String>>,
}

/// Argument definition for an MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpArg {
    #[serde(rename = "type")]
    pub arg_type: String, // "positional" or "named"
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub value_hint: Option<String>,
    pub description: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub choices: Option<Vec<String>>,
    #[serde(default)]
    pub repeated: bool,
}

/// An MCP server definition from the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    pub id: String,
    pub category_id: String,
    pub name: String,
    pub description: String,
    pub publisher: String,
    pub registry_type: String,
    pub identifier: String,
    pub transport: String, // "stdio", "sse", "http"
    #[serde(default)]
    pub env_vars: Option<Vec<McpEnvVar>>,
    #[serde(default)]
    pub args: Option<Vec<McpArg>>,
    #[serde(default)]
    pub github_stars: Option<i64>,
}

/// An installed MCP service instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpInstance {
    pub id: String,
    pub server_id: String,
    pub name: String,
    pub enabled: bool,
    pub transport: String, // "stdio", "sse", "http"
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub args: Option<Vec<String>>,
    #[serde(default)]
    pub env: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    pub url: Option<String>,
    pub installed_at: i64,
}
