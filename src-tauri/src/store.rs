use std::path::PathBuf;

use rusqlite::{params, Connection};

use crate::error::AppError;
use crate::models::{Conversation, Message};

/// Open or create the SQLite database and ensure all tables exist.
///
/// Foreign key constraints are enabled automatically.
pub fn init(db_path: &PathBuf) -> Result<Connection, AppError> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let conn = Connection::open(db_path)?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS conversations (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            model TEXT NOT NULL,
            system_prompt TEXT NOT NULL DEFAULT '',
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            conversation_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            token_count INTEGER,
            FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
        );",
    )?;

    Ok(conn)
}

// ---------------------------------------------------------------------------
// Conversation CRUD
// ---------------------------------------------------------------------------

/// List all conversations, ordered by most recently updated first.
pub fn list_conversations(conn: &Connection) -> Result<Vec<Conversation>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, title, model, system_prompt, created_at, updated_at \
         FROM conversations ORDER BY updated_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Conversation {
            id: row.get(0)?,
            title: row.get(1)?,
            model: row.get(2)?,
            system_prompt: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    })?;
    let mut conversations = Vec::new();
    for row in rows {
        conversations.push(row?);
    }
    Ok(conversations)
}

/// Insert a new conversation into the database.
pub fn create_conversation(conn: &Connection, conversation: &Conversation) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO conversations (id, title, model, system_prompt, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            conversation.id,
            conversation.title,
            conversation.model,
            conversation.system_prompt,
            conversation.created_at,
            conversation.updated_at,
        ],
    )?;
    Ok(())
}

/// Retrieve a single conversation by its ID.
pub fn get_conversation(conn: &Connection, id: &str) -> Result<Option<Conversation>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, title, model, system_prompt, created_at, updated_at \
         FROM conversations WHERE id = ?1",
    )?;
    let mut rows = stmt.query_map(params![id], |row| {
        Ok(Conversation {
            id: row.get(0)?,
            title: row.get(1)?,
            model: row.get(2)?,
            system_prompt: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    })?;
    match rows.next() {
        Some(Ok(conv)) => Ok(Some(conv)),
        Some(Err(e)) => Err(AppError::DbError(e.to_string())),
        None => Ok(None),
    }
}

/// Update title, model, system_prompt, and updated_at of an existing conversation.
pub fn update_conversation(conn: &Connection, conversation: &Conversation) -> Result<(), AppError> {
    conn.execute(
        "UPDATE conversations SET title = ?1, model = ?2, system_prompt = ?3, updated_at = ?4 \
         WHERE id = ?5",
        params![
            conversation.title,
            conversation.model,
            conversation.system_prompt,
            conversation.updated_at,
            conversation.id,
        ],
    )?;
    Ok(())
}

/// Delete a conversation and all its associated messages (via ON DELETE CASCADE).
pub fn delete_conversation(conn: &Connection, id: &str) -> Result<(), AppError> {
    conn.execute("DELETE FROM conversations WHERE id = ?1", params![id])?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Message CRUD
// ---------------------------------------------------------------------------

/// List all messages belonging to a conversation, ordered chronologically.
pub fn list_messages_by_conversation(
    conn: &Connection,
    conversation_id: &str,
) -> Result<Vec<Message>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, conversation_id, role, content, created_at, token_count \
         FROM messages WHERE conversation_id = ?1 ORDER BY created_at ASC",
    )?;
    let rows = stmt.query_map(params![conversation_id], |row| {
        Ok(Message {
            id: row.get(0)?,
            conversation_id: row.get(1)?,
            role: row.get(2)?,
            content: row.get(3)?,
            created_at: row.get(4)?,
            token_count: row.get(5)?,
        })
    })?;
    let mut messages = Vec::new();
    for row in rows {
        messages.push(row?);
    }
    Ok(messages)
}

/// Insert a single message into the database.
pub fn create_message(conn: &Connection, message: &Message) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO messages (id, conversation_id, role, content, created_at, token_count) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            message.id,
            message.conversation_id,
            message.role,
            message.content,
            message.created_at,
            message.token_count,
        ],
    )?;
    Ok(())
}

/// Insert multiple messages in a single transaction.
pub fn batch_create_messages(conn: &Connection, messages: &[Message]) -> Result<(), AppError> {
    let tx = conn.unchecked_transaction()?;
    for msg in messages {
        tx.execute(
            "INSERT INTO messages (id, conversation_id, role, content, created_at, token_count) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                msg.id,
                msg.conversation_id,
                msg.role,
                msg.content,
                msg.created_at,
                msg.token_count,
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
    conn.execute(
        "DELETE FROM messages WHERE conversation_id = ?1",
        params![conversation_id],
    )?;
    Ok(())
}
