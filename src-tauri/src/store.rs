use std::path::PathBuf;

use rusqlite::{params, Connection};

use crate::error::AppError;
use crate::models::{Conversation, Message, Prompt};

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
            provider_id TEXT NOT NULL DEFAULT '',
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
        );
        CREATE TABLE IF NOT EXISTS prompts (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            content TEXT NOT NULL,
            is_default INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );",
    )?;

    // 迁移：为旧数据库添加 pinned 列（如已存在则忽略）
    let _ = conn.execute(
        "ALTER TABLE conversations ADD COLUMN pinned INTEGER NOT NULL DEFAULT 0",
        [],
    );

    // 迁移：为旧数据库添加 provider_id 列（如已存在则忽略）
    let _ = conn.execute(
        "ALTER TABLE conversations ADD COLUMN provider_id TEXT NOT NULL DEFAULT ''",
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
        "SELECT id, title, provider_id, model, system_prompt, created_at, updated_at, pinned \
         FROM conversations ORDER BY pinned DESC, updated_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Conversation {
            id: row.get(0)?,
            title: row.get(1)?,
            provider_id: row.get(2)?,
            model: row.get(3)?,
            system_prompt: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
            pinned: row.get::<_, i64>(7)? != 0,
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
        "INSERT INTO conversations (id, title, provider_id, model, system_prompt, created_at, updated_at, pinned) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            conversation.id,
            conversation.title,
            conversation.provider_id,
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
        "SELECT id, title, provider_id, model, system_prompt, created_at, updated_at, pinned \
         FROM conversations WHERE id = ?1",
    )?;
    let mut rows = stmt.query_map(params![id], |row| {
        Ok(Conversation {
            id: row.get(0)?,
            title: row.get(1)?,
            provider_id: row.get(2)?,
            model: row.get(3)?,
            system_prompt: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
            pinned: row.get::<_, i64>(7)? != 0,
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
        "UPDATE conversations SET title = ?1, provider_id = ?2, model = ?3, system_prompt = ?4, updated_at = ?5 \
         WHERE id = ?6",
        params![
            conversation.title,
            conversation.provider_id,
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

// ---------------------------------------------------------------------------
// Prompt CRUD
// ---------------------------------------------------------------------------

/// List all prompts, ordered by most recently updated first.
pub fn list_prompts(conn: &Connection) -> Result<Vec<Prompt>, AppError> {
    log::debug!("Rust::store::list_prompts | 查询所有提示词");
    let mut stmt = conn.prepare(
        "SELECT id, name, content, is_default, created_at, updated_at \
         FROM prompts ORDER BY updated_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Prompt {
            id: row.get(0)?,
            name: row.get(1)?,
            content: row.get(2)?,
            is_default: row.get::<_, i64>(3)? != 0,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    })?;
    let mut prompts = Vec::new();
    for row in rows {
        prompts.push(row?);
    }
    Ok(prompts)
}

/// Insert a new prompt into the database.
pub fn create_prompt(conn: &Connection, prompt: &Prompt) -> Result<(), AppError> {
    log::info!("Rust::store::create_prompt | 创建提示词 | id={} name={}", prompt.id, prompt.name);
    conn.execute(
        "INSERT INTO prompts (id, name, content, is_default, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            prompt.id,
            prompt.name,
            prompt.content,
            prompt.is_default as i64,
            prompt.created_at,
            prompt.updated_at,
        ],
    )?;
    Ok(())
}

/// Update name, content, and updated_at of an existing prompt.
pub fn update_prompt(conn: &Connection, prompt: &Prompt) -> Result<(), AppError> {
    log::debug!("Rust::store::update_prompt | 更新提示词 | id={}", prompt.id);
    conn.execute(
        "UPDATE prompts SET name = ?1, content = ?2, updated_at = ?3 \
         WHERE id = ?4",
        params![
            prompt.name,
            prompt.content,
            prompt.updated_at,
            prompt.id,
        ],
    )?;
    Ok(())
}

/// Delete a prompt by its ID.
pub fn delete_prompt(conn: &Connection, id: &str) -> Result<(), AppError> {
    log::info!("Rust::store::delete_prompt | 删除提示词 | id={}", id);
    conn.execute("DELETE FROM prompts WHERE id = ?1", params![id])?;
    Ok(())
}

/// Set a prompt as default (clear others, set this one).
pub fn set_default_prompt(conn: &Connection, id: &str) -> Result<(), AppError> {
    log::info!("Rust::store::set_default_prompt | 设置默认提示词 | id={}", id);
    conn.execute("UPDATE prompts SET is_default = 0 WHERE is_default = 1", [])?;
    conn.execute("UPDATE prompts SET is_default = 1 WHERE id = ?1", params![id])?;
    Ok(())
}

/// Get the default prompt, if one exists.
pub fn get_default_prompt(conn: &Connection) -> Result<Option<Prompt>, AppError> {
    log::debug!("Rust::store::get_default_prompt | 查询默认提示词");
    let mut stmt = conn.prepare(
        "SELECT id, name, content, is_default, created_at, updated_at \
         FROM prompts WHERE is_default = 1 LIMIT 1",
    )?;
    let mut rows = stmt.query_map([], |row| {
        Ok(Prompt {
            id: row.get(0)?,
            name: row.get(1)?,
            content: row.get(2)?,
            is_default: row.get::<_, i64>(3)? != 0,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    })?;
    match rows.next() {
        Some(Ok(prompt)) => Ok(Some(prompt)),
        Some(Err(e)) => Err(AppError::DbError(e.to_string())),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        conn.execute_batch(
            "            CREATE TABLE IF NOT EXISTS conversations (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                provider_id TEXT NOT NULL DEFAULT '',
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
            );
            CREATE TABLE IF NOT EXISTS prompts (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                content TEXT NOT NULL,
                is_default INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );",
        ).unwrap();
        conn
    }

    #[test]
    fn create_and_list_prompts() {
        let conn = setup_db();
        let p = Prompt {
            id: "p1".into(),
            name: "翻译".into(),
            content: "你是翻译助手".into(),
            is_default: false,
            created_at: 1000,
            updated_at: 1000,
        };
        create_prompt(&conn, &p).unwrap();
        let list = list_prompts(&conn).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "翻译");
    }

    #[test]
    fn update_prompt_changes_content() {
        let conn = setup_db();
        let p = Prompt {
            id: "p1".into(),
            name: "翻译".into(),
            content: "旧内容".into(),
            is_default: false,
            created_at: 1000,
            updated_at: 1000,
        };
        create_prompt(&conn, &p).unwrap();

        let updated = Prompt {
            id: "p1".into(),
            name: "翻译".into(),
            content: "新内容".into(),
            is_default: false,
            created_at: 1000,
            updated_at: 2000,
        };
        update_prompt(&conn, &updated).unwrap();

        let list = list_prompts(&conn).unwrap();
        assert_eq!(list[0].content, "新内容");
    }

    #[test]
    fn delete_prompt_removes_it() {
        let conn = setup_db();
        let p = Prompt {
            id: "p1".into(),
            name: "翻译".into(),
            content: "内容".into(),
            is_default: false,
            created_at: 1000,
            updated_at: 1000,
        };
        create_prompt(&conn, &p).unwrap();
        delete_prompt(&conn, "p1").unwrap();
        let list = list_prompts(&conn).unwrap();
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn set_default_prompt_sets_only_one() {
        let conn = setup_db();
        create_prompt(&conn, &Prompt { id: "p1".into(), name: "A".into(), content: "a".into(), is_default: false, created_at: 0, updated_at: 0 }).unwrap();
        create_prompt(&conn, &Prompt { id: "p2".into(), name: "B".into(), content: "b".into(), is_default: false, created_at: 0, updated_at: 0 }).unwrap();

        set_default_prompt(&conn, "p1").unwrap();
        let def = get_default_prompt(&conn).unwrap().unwrap();
        assert_eq!(def.id, "p1");

        set_default_prompt(&conn, "p2").unwrap();
        let def = get_default_prompt(&conn).unwrap().unwrap();
        assert_eq!(def.id, "p2");

        // p1 should no longer be default
        let list = list_prompts(&conn).unwrap();
        let p1 = list.iter().find(|p| p.id == "p1").unwrap();
        assert!(!p1.is_default);
    }

    #[test]
    fn get_default_prompt_returns_none_when_none_set() {
        let conn = setup_db();
        create_prompt(&conn, &Prompt { id: "p1".into(), name: "A".into(), content: "a".into(), is_default: false, created_at: 0, updated_at: 0 }).unwrap();
        let result = get_default_prompt(&conn).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn system_prompt_injection_uses_default_prompt() {
        let conn = setup_db();

        // Create a conversation with no system_prompt
        create_conversation(&conn, &Conversation {
            id: "c1".into(),
            title: "test".into(),
            provider_id: "".into(),
            model: "gpt-4".into(),
            system_prompt: "".into(),
            created_at: 0,
            updated_at: 0,
            pinned: false,
        }).unwrap();

        // Create a default prompt
        create_prompt(&conn, &Prompt {
            id: "p1".into(),
            name: "翻译".into(),
            content: "你是翻译AI助手，所有输入都翻译成中文".into(),
            is_default: false,
            created_at: 0,
            updated_at: 0,
        }).unwrap();
        set_default_prompt(&conn, "p1").unwrap();

        // Simulate the injection logic from send_message
        let conv = get_conversation(&conn, "c1").unwrap().unwrap();
        let from_conv = if conv.system_prompt.is_empty() { None } else { Some(conv.system_prompt.clone()) };
        let system_prompt = if from_conv.is_some() {
            from_conv
        } else {
            get_default_prompt(&conn).unwrap().map(|p| p.content)
        };

        assert!(system_prompt.is_some());
        assert_eq!(system_prompt.unwrap(), "你是翻译AI助手，所有输入都翻译成中文");
    }

    #[test]
    fn system_prompt_injection_prefers_conversation_prompt() {
        let conn = setup_db();

        // Create a conversation WITH system_prompt
        create_conversation(&conn, &Conversation {
            id: "c1".into(),
            title: "test".into(),
            provider_id: "".into(),
            model: "gpt-4".into(),
            system_prompt: "对话专属提示词".into(),
            created_at: 0,
            updated_at: 0,
            pinned: false,
        }).unwrap();

        // Also create a default prompt
        create_prompt(&conn, &Prompt {
            id: "p1".into(),
            name: "翻译".into(),
            content: "默认提示词".into(),
            is_default: false,
            created_at: 0,
            updated_at: 0,
        }).unwrap();
        set_default_prompt(&conn, "p1").unwrap();

        // Simulate the injection logic
        let conv = get_conversation(&conn, "c1").unwrap().unwrap();
        let from_conv = if conv.system_prompt.is_empty() { None } else { Some(conv.system_prompt.clone()) };
        let system_prompt = if from_conv.is_some() {
            from_conv
        } else {
            get_default_prompt(&conn).unwrap().map(|p| p.content)
        };

        // Should use conversation's prompt, not the default
        assert_eq!(system_prompt.unwrap(), "对话专属提示词");
    }

    #[test]
    fn conversation_with_provider_id() {
        let conn = setup_db();
        create_conversation(&conn, &Conversation {
            id: "c1".into(),
            title: "test".into(),
            provider_id: "prov-1".into(),
            model: "gpt-4".into(),
            system_prompt: "".into(),
            created_at: 0,
            updated_at: 0,
            pinned: false,
        }).unwrap();

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
        // Simulate old data without provider_id by inserting with empty
        create_conversation(&conn, &Conversation {
            id: "c-old".into(),
            title: "old".into(),
            provider_id: "".into(),
            model: "deepseek-chat".into(),
            system_prompt: "".into(),
            created_at: 0,
            updated_at: 0,
            pinned: false,
        }).unwrap();

        let conv = get_conversation(&conn, "c-old").unwrap().unwrap();
        assert_eq!(conv.provider_id, "");
    }
}
