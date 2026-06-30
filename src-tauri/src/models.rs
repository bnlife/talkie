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

/// A conversation (chat session) that groups multiple messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub provider_id: String,
    pub model: String,
    pub system_prompt: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub pinned: bool,
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
