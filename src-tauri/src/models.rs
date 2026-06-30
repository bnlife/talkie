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

/// A conversation (chat session) that groups multiple messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub model: String,
    pub system_prompt: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub pinned: bool,
}

/// Application settings persisted in a JSON config file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub temperature: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_active_conversation_id: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            base_url: "https://api.deepseek.com/v1".to_string(),
            api_key: String::new(),
            model: "deepseek-chat".to_string(),
            temperature: 0.7,
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
