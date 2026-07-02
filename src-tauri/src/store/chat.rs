use rusqlite::{params, Connection};

use crate::error::AppError;
use crate::models::{Conversation, ConversationConfig, ConversationView, Message};

/// List all conversations with their configs, ordered by most recently updated first.
pub fn list_conversations(conn: &Connection) -> Result<Vec<ConversationView>, AppError> {
    log::debug!("RS::list_conversations");
    let mut stmt = conn.prepare(
        "SELECT c.id, c.title, c.created_at, c.updated_at, c.pinned,
                COALESCE(cfg.provider_id, ''), COALESCE(cfg.model, ''), cfg.prompt_id, COALESCE(cfg.search_enabled, 0), COALESCE(cfg.search_engine, '')
         FROM conversations c
         LEFT JOIN conversation_configs cfg ON cfg.conversation_id = c.id
         ORDER BY c.pinned DESC, c.updated_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(ConversationView {
            id: row.get(0)?,
            title: row.get(1)?,
            created_at: row.get(2)?,
            updated_at: row.get(3)?,
            pinned: row.get::<_, i64>(4)? != 0,
            provider_id: row.get(5)?,
            model: row.get(6)?,
            prompt_id: row.get(7)?,
            search_enabled: row.get::<_, i64>(8)? != 0,
            search_engine: row.get(9)?,
        })
    })?;
    let mut conversations = Vec::new();
    for row in rows {
        conversations.push(row?);
    }
    Ok(conversations)
}

/// Insert a new conversation and its config into the database.
pub fn create_conversation(conn: &Connection, conv: &Conversation, config: &ConversationConfig) -> Result<(), AppError> {
    log::info!("RS::create_conversation | id={}", conv.id);
    // Include legacy columns (provider_id, model, system_prompt) for old databases
    // where they have NOT NULL constraints. Empty defaults satisfy the constraint.
    conn.execute(
        "INSERT INTO conversations (id, title, created_at, updated_at, pinned, provider_id, model, system_prompt) VALUES (?1, ?2, ?3, ?4, ?5, '', '', '')",
        params![conv.id, conv.title, conv.created_at, conv.updated_at, conv.pinned as i64],
    )?;
    conn.execute(
        "INSERT INTO conversation_configs (conversation_id, provider_id, model, prompt_id, search_enabled, search_engine) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![config.conversation_id, config.provider_id, config.model, config.prompt_id, config.search_enabled as i64, config.search_engine],
    )?;
    Ok(())
}

/// Retrieve a single conversation with its config by ID.
pub fn get_conversation(conn: &Connection, id: &str) -> Result<Option<ConversationView>, AppError> {
    log::debug!("RS::get_conversation | id={}", id);
    let mut stmt = conn.prepare(
        "SELECT c.id, c.title, c.created_at, c.updated_at, c.pinned,
                COALESCE(cfg.provider_id, ''), COALESCE(cfg.model, ''), cfg.prompt_id, COALESCE(cfg.search_enabled, 0), COALESCE(cfg.search_engine, '')
         FROM conversations c
         LEFT JOIN conversation_configs cfg ON cfg.conversation_id = c.id
         WHERE c.id = ?1",
    )?;
    let mut rows = stmt.query_map(params![id], |row| {
        Ok(ConversationView {
            id: row.get(0)?,
            title: row.get(1)?,
            created_at: row.get(2)?,
            updated_at: row.get(3)?,
            pinned: row.get::<_, i64>(4)? != 0,
            provider_id: row.get(5)?,
            model: row.get(6)?,
            prompt_id: row.get(7)?,
            search_enabled: row.get::<_, i64>(8)? != 0,
            search_engine: row.get(9)?,
        })
    })?;
    match rows.next() {
        Some(Ok(conv)) => Ok(Some(conv)),
        Some(Err(e)) => Err(AppError::DbError(e.to_string())),
        None => Ok(None),
    }
}

/// Update conversation core fields (title, pinned, timestamps).
pub fn update_conversation(conn: &Connection, conv: &Conversation) -> Result<(), AppError> {
    log::debug!("RS::update_conversation | id={}", conv.id);
    conn.execute(
        "UPDATE conversations SET title = ?1, pinned = ?2, updated_at = ?3 WHERE id = ?4",
        params![conv.title, conv.pinned as i64, conv.updated_at, conv.id],
    )?;
    Ok(())
}

/// Update conversation config fields.
pub fn update_conversation_config(conn: &Connection, config: &ConversationConfig) -> Result<(), AppError> {
    log::debug!("RS::update_conversation_config | id={}", config.conversation_id);
    conn.execute(
        "UPDATE conversation_configs SET provider_id = ?1, model = ?2, prompt_id = ?3, search_enabled = ?4, search_engine = ?5 WHERE conversation_id = ?6",
        params![config.provider_id, config.model, config.prompt_id, config.search_enabled as i64, config.search_engine, config.conversation_id],
    )?;
    Ok(())
}

/// Delete a conversation and all its associated messages (via ON DELETE CASCADE).
pub fn delete_conversation(conn: &Connection, id: &str) -> Result<(), AppError> {
    log::info!("RS::delete_conversation | id={}", id);
    conn.execute("DELETE FROM conversations WHERE id = ?1", params![id])?;
    Ok(())
}

/// Pin a conversation (set pinned = 1).
pub fn pin_conversation(conn: &Connection, id: &str) -> Result<(), AppError> {
    log::info!("RS::pin_conversation | id={}", id);
    conn.execute(
        "UPDATE conversations SET pinned = 1 WHERE id = ?1",
        params![id],
    )?;
    Ok(())
}

/// Unpin a conversation (set pinned = 0).
pub fn unpin_conversation(conn: &Connection, id: &str) -> Result<(), AppError> {
    log::info!("RS::unpin_conversation | id={}", id);
    conn.execute(
        "UPDATE conversations SET pinned = 0 WHERE id = ?1",
        params![id],
    )?;
    Ok(())
}

/// List all messages belonging to a conversation, ordered chronologically.
pub fn list_messages_by_conversation(
    conn: &Connection,
    conversation_id: &str,
) -> Result<Vec<Message>, AppError> {
    log::debug!("RS::list_messages | conv={}", conversation_id);
    let mut stmt = conn.prepare(
        "SELECT id, conversation_id, role, content, created_at, token_count, search_results, thinking_content, attachments \
         FROM messages WHERE conversation_id = ?1 ORDER BY created_at ASC",
    )?;
    let rows = stmt.query_map(params![conversation_id], |row| {
        let search_results_json: Option<String> = row.get(6)?;
        let search_results = search_results_json
            .and_then(|json| serde_json::from_str::<Vec<crate::models::SearchResult>>(&json).ok());
        let thinking_content: Option<String> = row.get(7)?;
        let attachments_json: Option<String> = row.get(8)?;
        let attachments = attachments_json
            .and_then(|json| serde_json::from_str::<Vec<crate::models::AttachmentMeta>>(&json).ok());
        Ok(Message {
            id: row.get(0)?,
            conversation_id: row.get(1)?,
            role: row.get(2)?,
            content: row.get(3)?,
            created_at: row.get(4)?,
            token_count: row.get(5)?,
            search_results,
            thinking_content,
            attachments,
        })
    })?;
    let mut messages = Vec::new();
    for row in rows {
        messages.push(row?);
    }
    Ok(messages)
}

/// List messages with pagination, ordered by created_at DESC (newest first).
pub fn list_messages_paginated(
    conn: &Connection,
    conversation_id: &str,
    offset: i64,
    limit: i64,
) -> Result<crate::models::MessagesPage, AppError> {
    log::debug!("RS::list_messages_paginated | conv={} offset={} limit={}", conversation_id, offset, limit);

    // 1. Get total count
    let total: i64 = conn.query_row(
        "SELECT COUNT(*) FROM messages WHERE conversation_id = ?1",
        params![conversation_id],
        |row| row.get(0),
    )?;

    // 2. Query with pagination (DESC order, then reverse for display)
    let mut stmt = conn.prepare(
        "SELECT id, conversation_id, role, content, created_at, token_count, search_results, thinking_content, attachments \
         FROM messages WHERE conversation_id = ?1 ORDER BY created_at DESC LIMIT ?2 OFFSET ?3",
    )?;
    let rows = stmt.query_map(params![conversation_id, limit, offset], |row| {
        let search_results_json: Option<String> = row.get(6)?;
        let search_results = search_results_json
            .and_then(|json| serde_json::from_str::<Vec<crate::models::SearchResult>>(&json).ok());
        let thinking_content: Option<String> = row.get(7)?;
        let attachments_json: Option<String> = row.get(8)?;
        let attachments = attachments_json
            .and_then(|json| serde_json::from_str::<Vec<crate::models::AttachmentMeta>>(&json).ok());
        Ok(Message {
            id: row.get(0)?,
            conversation_id: row.get(1)?,
            role: row.get(2)?,
            content: row.get(3)?,
            created_at: row.get(4)?,
            token_count: row.get(5)?,
            search_results,
            thinking_content,
            attachments,
        })
    })?;

    let mut messages: Vec<Message> = Vec::new();
    for row in rows {
        messages.push(row?);
    }
    // Reverse to get chronological order (oldest first) for display
    messages.reverse();

    let has_more = (offset + limit) < total;
    log::debug!("RS::list_messages_paginated | total={} returned={} has_more={}", total, messages.len(), has_more);

    Ok(crate::models::MessagesPage { messages, total, has_more })
}

/// Insert a single message into the database.
pub fn create_message(conn: &Connection, message: &Message) -> Result<(), AppError> {
    log::debug!("RS::create_message | conv={} role={}", message.conversation_id, message.role);
    let search_results_json = message.search_results.as_ref()
        .map(|sr| serde_json::to_string(sr).unwrap_or_default());
    let attachments_json = message.attachments.as_ref()
        .map(|a| serde_json::to_string(a).unwrap_or_default());
    conn.execute(
        "INSERT INTO messages (id, conversation_id, role, content, created_at, token_count, search_results, thinking_content, attachments) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            message.id,
            message.conversation_id,
            message.role,
            message.content,
            message.created_at,
            message.token_count,
            search_results_json,
            message.thinking_content,
            attachments_json,
        ],
    )?;
    Ok(())
}

/// Insert multiple messages in a single transaction.
pub fn batch_create_messages(conn: &Connection, messages: &[Message]) -> Result<(), AppError> {
    log::debug!("RS::batch_create_messages | count={}", messages.len());
    let tx = conn.unchecked_transaction()?;
    for msg in messages {
        let search_results_json = msg.search_results.as_ref()
            .map(|sr| serde_json::to_string(sr).unwrap_or_default());
        let attachments_json = msg.attachments.as_ref()
            .map(|a| serde_json::to_string(a).unwrap_or_default());
        tx.execute(
            "INSERT INTO messages (id, conversation_id, role, content, created_at, token_count, search_results, thinking_content, attachments) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                msg.id,
                msg.conversation_id,
                msg.role,
                msg.content,
                msg.created_at,
                msg.token_count,
                search_results_json,
                msg.thinking_content,
                attachments_json,
            ],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// Delete all messages belonging to a specific conversation.
pub fn delete_messages_by_conversation(
    conn: &Connection,
    conversation_id: &str,
) -> Result<(), AppError> {
    log::debug!("RS::delete_messages | conv={}", conversation_id);
    conn.execute(
        "DELETE FROM messages WHERE conversation_id = ?1",
        params![conversation_id],
    )?;
    Ok(())
}

/// Delete a single message by its ID.
pub fn delete_message(conn: &Connection, message_id: &str) -> Result<(), AppError> {
    log::info!("RS::delete_message | id={}", message_id);
    conn.execute("DELETE FROM messages WHERE id = ?1", params![message_id])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS conversations (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                pinned INTEGER NOT NULL DEFAULT 0,
                provider_id TEXT NOT NULL DEFAULT '',
                model TEXT NOT NULL DEFAULT '',
                system_prompt TEXT NOT NULL DEFAULT ''
            );
            CREATE TABLE IF NOT EXISTS conversation_configs (
                conversation_id TEXT PRIMARY KEY,
                provider_id TEXT NOT NULL DEFAULT '',
                model TEXT NOT NULL DEFAULT '',
                prompt_id TEXT,
                search_enabled INTEGER NOT NULL DEFAULT 0,
                search_engine TEXT NOT NULL DEFAULT '',
                FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                token_count INTEGER,
                search_results TEXT,
                thinking_content TEXT,
                FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
            );",
        ).unwrap();
        conn
    }

    #[test]
    fn conversation_with_provider_id() {
        let conn = setup_db();
        create_conversation(&conn,
            &Conversation { id: "c1".into(), title: "test".into(), created_at: 0, updated_at: 0, pinned: false },
            &ConversationConfig { conversation_id: "c1".into(), provider_id: "prov-1".into(), model: "gpt-4".into(), prompt_id: None, search_enabled: false, search_engine: String::new() },
        ).unwrap();

        let conv = get_conversation(&conn, "c1").unwrap().unwrap();
        assert_eq!(conv.provider_id, "prov-1");
        assert_eq!(conv.model, "gpt-4");

        let list = list_conversations(&conn).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].provider_id, "prov-1");
    }

    #[test]
    fn conversation_provider_id_defaults_to_empty() {
        let conn = setup_db();
        create_conversation(&conn,
            &Conversation { id: "c-old".into(), title: "old".into(), created_at: 0, updated_at: 0, pinned: false },
            &ConversationConfig { conversation_id: "c-old".into(), provider_id: "".into(), model: "deepseek-chat".into(), prompt_id: None, search_enabled: false, search_engine: String::new() },
        ).unwrap();

        let conv = get_conversation(&conn, "c-old").unwrap().unwrap();
        assert_eq!(conv.provider_id, "");
    }

    #[test]
    fn conversation_config_update() {
        let conn = setup_db();
        create_conversation(&conn,
            &Conversation { id: "c1".into(), title: "test".into(), created_at: 0, updated_at: 0, pinned: false },
            &ConversationConfig { conversation_id: "c1".into(), provider_id: "prov-1".into(), model: "gpt-4".into(), prompt_id: None, search_enabled: false, search_engine: String::new() },
        ).unwrap();

        // Update config
        update_conversation_config(&conn, &ConversationConfig {
            conversation_id: "c1".into(),
            provider_id: "prov-2".into(),
            model: "deepseek-chat".into(),
            prompt_id: Some("p1".into()),
            search_enabled: true,
            search_engine: String::new(),
        }).unwrap();

        let conv = get_conversation(&conn, "c1").unwrap().unwrap();
        assert_eq!(conv.provider_id, "prov-2");
        assert_eq!(conv.model, "deepseek-chat");
        assert_eq!(conv.prompt_id, Some("p1".into()));
        assert!(conv.search_enabled);
    }

    #[test]
    fn message_search_results_persistence() {
        let conn = setup_db();
        let conv = Conversation {
            id: "conv-sr".into(),
            title: "Test".into(),
            created_at: 1000,
            updated_at: 1000,
            pinned: false,
        };
        let config = ConversationConfig {
            conversation_id: "conv-sr".into(),
            provider_id: "prov-1".into(),
            model: "test".into(),
            prompt_id: None,
            search_enabled: false,
            search_engine: String::new(),
        };
        create_conversation(&conn, &conv, &config).unwrap();

        let search_results = vec![
            crate::models::SearchResult {
                title: "Rust 语言官网".into(),
                url: "https://www.rust-lang.org".into(),
                snippet: Some("Rust 是一门系统编程语言".into()),
            },
            crate::models::SearchResult {
                title: "Cargo 手册".into(),
                url: "https://doc.rust-lang.org/cargo".into(),
                snippet: None,
            },
        ];

        let msg = Message {
            id: "msg-sr".into(),
            conversation_id: "conv-sr".into(),
            role: "assistant".into(),
            content: "根据搜索结果...".into(),
            created_at: 2000,
            token_count: Some(100),
            search_results: Some(search_results),
            thinking_content: None,
            attachments: None,
        };
        create_message(&conn, &msg).unwrap();

        let messages = list_messages_by_conversation(&conn, "conv-sr").unwrap();
        assert_eq!(messages.len(), 1);

        let sr = messages[0].search_results.as_ref().unwrap();
        assert_eq!(sr.len(), 2);
        assert_eq!(sr[0].title, "Rust 语言官网");
        assert_eq!(sr[0].url, "https://www.rust-lang.org");
        assert_eq!(sr[0].snippet, Some("Rust 是一门系统编程语言".into()));
        assert_eq!(sr[1].title, "Cargo 手册");
        assert_eq!(sr[1].snippet, None);
    }

    #[test]
    fn message_without_search_results_returns_none() {
        let conn = setup_db();
        let conv = Conversation {
            id: "conv-nosr".into(),
            title: "Test".into(),
            created_at: 1000,
            updated_at: 1000,
            pinned: false,
        };
        let config = ConversationConfig {
            conversation_id: "conv-nosr".into(),
            provider_id: "prov-1".into(),
            model: "test".into(),
            prompt_id: None,
            search_enabled: false,
            search_engine: String::new(),
        };
        create_conversation(&conn, &conv, &config).unwrap();

        let msg = Message {
            id: "msg-nosr".into(),
            conversation_id: "conv-nosr".into(),
            role: "user".into(),
            content: "你好".into(),
            created_at: 2000,
            token_count: None,
            search_results: None,
            thinking_content: None,
            attachments: None,
        };
        create_message(&conn, &msg).unwrap();

        let messages = list_messages_by_conversation(&conn, "conv-nosr").unwrap();
        assert_eq!(messages.len(), 1);
        assert!(messages[0].search_results.is_none());
    }
}
