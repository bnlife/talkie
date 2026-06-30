use std::path::PathBuf;

use rusqlite::{params, Connection};

use crate::error::AppError;
use crate::models::{Conversation, Message};

/// Open or create the SQLite database and ensure all tables exist.
///
/// Foreign key constraints are enabled automatically.
pub fn init(db_path: &PathBuf) -> Result<Connection, AppError> {
    log::info!("Rust::store::init | 初始化数据库 | path={:?}", db_path);
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
            updated_at INTEGER NOT NULL,
            pinned INTEGER NOT NULL DEFAULT 0
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

    // 迁移：为旧数据库添加 pinned 列（如已存在则忽略）
    let _ = conn.execute(
        "ALTER TABLE conversations ADD COLUMN pinned INTEGER NOT NULL DEFAULT 0",
        [],
    );

    log::info!("Rust::store::init | 数据库初始化完成");
    Ok(conn)
}

// ---------------------------------------------------------------------------
// Conversation CRUD
// ---------------------------------------------------------------------------

/// List all conversations, ordered by most recently updated first.
pub fn list_conversations(conn: &Connection) -> Result<Vec<Conversation>, AppError> {
    log::debug!("Rust::store::list_conversations | 查询所有对话");
    let mut stmt = conn.prepare(
        "SELECT id, title, model, system_prompt, created_at, updated_at, pinned \
         FROM conversations ORDER BY pinned DESC, updated_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Conversation {
            id: row.get(0)?,
            title: row.get(1)?,
            model: row.get(2)?,
            system_prompt: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
            pinned: row.get::<_, i64>(6)? != 0,
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
    log::info!("Rust::store::create_conversation | 创建对话 | id={}", conversation.id);
    conn.execute(
        "INSERT INTO conversations (id, title, model, system_prompt, created_at, updated_at, pinned) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            conversation.id,
            conversation.title,
            conversation.model,
            conversation.system_prompt,
            conversation.created_at,
            conversation.updated_at,
            conversation.pinned as i64,
        ],
    )?;
    Ok(())
}

/// Retrieve a single conversation by its ID.
pub fn get_conversation(conn: &Connection, id: &str) -> Result<Option<Conversation>, AppError> {
    log::debug!("Rust::store::get_conversation | 查询对话 | id={}", id);
    let mut stmt = conn.prepare(
        "SELECT id, title, model, system_prompt, created_at, updated_at, pinned \
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
            pinned: row.get::<_, i64>(6)? != 0,
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
    log::debug!("Rust::store::update_conversation | 更新对话 | id={}", conversation.id);
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
    log::info!("Rust::store::delete_conversation | 删除对话及关联消息 | id={}", id);
    conn.execute("DELETE FROM conversations WHERE id = ?1", params![id])?;
    Ok(())
}

/// Pin a conversation (set pinned = 1).
pub fn pin_conversation(conn: &Connection, id: &str) -> Result<(), AppError> {
    log::info!("Rust::store::pin_conversation | 置顶对话 | id={}", id);
    conn.execute(
        "UPDATE conversations SET pinned = 1 WHERE id = ?1",
        params![id],
    )?;
    Ok(())
}

/// Unpin a conversation (set pinned = 0).
pub fn unpin_conversation(conn: &Connection, id: &str) -> Result<(), AppError> {
    log::info!("Rust::store::unpin_conversation | 取消置顶 | id={}", id);
    conn.execute(
        "UPDATE conversations SET pinned = 0 WHERE id = ?1",
        params![id],
    )?;
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
    log::debug!("Rust::store::list_messages_by_conversation | 查询消息列表 | conv={}", conversation_id);
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
    log::debug!("Rust::store::create_message | 创建消息 | conv={} role={}", message.conversation_id, message.role);
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
    log::debug!("Rust::store::batch_create_messages | 批量创建消息 | count={}", messages.len());
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
    log::debug!("Rust::store::delete_messages_by_conversation | 删除消息 | conv={}", conversation_id);
    conn.execute(
        "DELETE FROM messages WHERE conversation_id = ?1",
        params![conversation_id],
    )?;
    Ok(())
}

/// Delete a single message by its ID.
pub fn delete_message(conn: &Connection, message_id: &str) -> Result<(), AppError> {
    log::info!("Rust::store::delete_message | 删除单条消息 | id={}", message_id);
    conn.execute("DELETE FROM messages WHERE id = ?1", params![message_id])?;
    Ok(())
}
