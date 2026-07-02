use rusqlite::{params, Connection};

use crate::error::AppError;
use crate::models::Prompt;

/// List all prompts, ordered by most recently updated first.
pub fn list_prompts(conn: &Connection) -> Result<Vec<Prompt>, AppError> {
    log::debug!("RS::list_prompts");
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
    log::info!("RS::create_prompt | id={} name={}", prompt.id, prompt.name);
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
    log::debug!("RS::update_prompt | id={}", prompt.id);
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
    log::info!("RS::delete_prompt | id={}", id);
    conn.execute("DELETE FROM prompts WHERE id = ?1", params![id])?;
    Ok(())
}

/// Set a prompt as default (clear others, set this one).
pub fn set_default_prompt(conn: &Connection, id: &str) -> Result<(), AppError> {
    log::info!("RS::set_default_prompt | id={}", id);
    conn.execute("UPDATE prompts SET is_default = 0 WHERE is_default = 1", [])?;
    conn.execute("UPDATE prompts SET is_default = 1 WHERE id = ?1", params![id])?;
    Ok(())
}

/// Get the default prompt, if one exists.
pub fn get_default_prompt(conn: &Connection) -> Result<Option<Prompt>, AppError> {
    log::debug!("RS::get_default_prompt");
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

/// Get a prompt by its ID.
pub fn get_prompt_by_id(conn: &Connection, id: &str) -> Result<Option<Prompt>, AppError> {
    log::debug!("RS::get_prompt_by_id | id={}", id);
    let mut stmt = conn.prepare(
        "SELECT id, name, content, is_default, created_at, updated_at \
         FROM prompts WHERE id = ?1",
    )?;
    let mut rows = stmt.query_map(params![id], |row| {
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
            "CREATE TABLE IF NOT EXISTS prompts (
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
}
