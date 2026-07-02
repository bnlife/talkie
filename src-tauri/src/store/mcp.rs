use rusqlite::{params, Connection};

use crate::crypto;
use crate::error::AppError;
use crate::models::{McpCategory, McpInstance, McpServer};

/// Encrypt all env values in-place.
fn encrypt_env_values(env: &mut std::collections::HashMap<String, String>) {
    for (_, value) in env.iter_mut() {
        match crypto::ensure_encrypted(value) {
            Ok(encrypted) => *value = encrypted,
            Err(e) => log::error!("RS::mcp::store | encrypt env fail: {}", e),
        }
    }
}

/// Decrypt all env values in-place.
fn decrypt_env_values(env: &mut std::collections::HashMap<String, String>) {
    for (_, value) in env.iter_mut() {
        match crypto::ensure_decrypted(value) {
            Ok(decrypted) => *value = decrypted,
            Err(e) => {
                log::error!("RS::mcp::store | decrypt env fail: {}", e);
                value.clear();
            }
        }
    }
}

/// List all MCP market categories.
pub fn list_mcp_categories(conn: &Connection) -> Result<Vec<McpCategory>, AppError> {
    log::debug!("RS::list_mcp_categories");
    let mut stmt = conn.prepare(
        "SELECT id, name, icon FROM mcp_categories ORDER BY name",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(McpCategory {
            id: row.get(0)?,
            name: row.get(1)?,
            icon: row.get(2)?,
        })
    })?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

/// List MCP servers, optionally filtered by category.
pub fn list_mcp_servers(conn: &Connection, category_id: Option<&str>) -> Result<Vec<McpServer>, AppError> {
    log::debug!("RS::list_mcp_servers | cat={:?}", category_id);
    let (sql, param_values): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = match category_id {
        Some(cid) => (
            "SELECT id, category_id, name, description, publisher, registry_type, identifier, \
             transport, env_vars_json, args_json, github_stars \
             FROM mcp_servers WHERE category_id = ?1 ORDER BY name",
            vec![Box::new(cid.to_string())],
        ),
        None => (
            "SELECT id, category_id, name, description, publisher, registry_type, identifier, \
             transport, env_vars_json, args_json, github_stars \
             FROM mcp_servers ORDER BY name",
            vec![],
        ),
    };
    let mut stmt = conn.prepare(sql)?;
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
    let rows = stmt.query_map(param_refs.as_slice(), |row| {
        let env_json: Option<String> = row.get(8)?;
        let args_json: Option<String> = row.get(9)?;
        Ok(McpServer {
            id: row.get(0)?,
            category_id: row.get(1)?,
            name: row.get(2)?,
            description: row.get(3)?,
            publisher: row.get(4)?,
            registry_type: row.get(5)?,
            identifier: row.get(6)?,
            transport: row.get(7)?,
            env_vars: env_json.and_then(|j| serde_json::from_str(&j).ok()),
            args: args_json.and_then(|j| serde_json::from_str(&j).ok()),
            github_stars: row.get(10)?,
        })
    })?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

/// List all installed MCP instances.
pub fn list_mcp_instances(conn: &Connection) -> Result<Vec<McpInstance>, AppError> {
    log::debug!("RS::list_mcp_instances");
    let mut stmt = conn.prepare(
        "SELECT id, server_id, name, enabled, transport, command, args_json, env_json, url, installed_at \
         FROM mcp_instances ORDER BY installed_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        let args_json: Option<String> = row.get(6)?;
        let env_json: Option<String> = row.get(7)?;
        Ok(McpInstance {
            id: row.get(0)?,
            server_id: row.get(1)?,
            name: row.get(2)?,
            enabled: row.get::<_, i64>(3)? != 0,
            transport: row.get(4)?,
            command: row.get(5)?,
            args: args_json.and_then(|j| serde_json::from_str(&j).ok()),
            env: env_json.and_then(|j| serde_json::from_str(&j).ok()),
            url: row.get(8)?,
            installed_at: row.get(9)?,
        })
    })?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    // Decrypt env values
    for inst in &mut result {
        if let Some(ref mut env) = inst.env {
            decrypt_env_values(env);
        }
    }
    Ok(result)
}

/// Get a single MCP instance by ID.
pub fn get_mcp_instance(conn: &Connection, id: &str) -> Result<Option<McpInstance>, AppError> {
    log::debug!("RS::get_mcp_instance | id={}", id);
    let mut stmt = conn.prepare(
        "SELECT id, server_id, name, enabled, transport, command, args_json, env_json, url, installed_at \
         FROM mcp_instances WHERE id = ?1",
    )?;
    let mut rows = stmt.query_map(params![id], |row| {
        let args_json: Option<String> = row.get(6)?;
        let env_json: Option<String> = row.get(7)?;
        Ok(McpInstance {
            id: row.get(0)?,
            server_id: row.get(1)?,
            name: row.get(2)?,
            enabled: row.get::<_, i64>(3)? != 0,
            transport: row.get(4)?,
            command: row.get(5)?,
            args: args_json.and_then(|j| serde_json::from_str(&j).ok()),
            env: env_json.and_then(|j| serde_json::from_str(&j).ok()),
            url: row.get(8)?,
            installed_at: row.get(9)?,
        })
    })?;
    match rows.next() {
        Some(Ok(mut inst)) => {
            // Decrypt env values
            if let Some(ref mut env) = inst.env {
                decrypt_env_values(env);
            }
            Ok(Some(inst))
        }
        Some(Err(e)) => Err(AppError::DbError(e.to_string())),
        None => Ok(None),
    }
}

/// Create (install) a new MCP instance.
pub fn create_mcp_instance(conn: &Connection, inst: &McpInstance) -> Result<(), AppError> {
    log::info!("RS::create_mcp_instance | id={} name={}", inst.id, inst.name);
    let args_json = inst.args.as_ref().map(|a| serde_json::to_string(a).unwrap_or_default());
    // Encrypt env values before storing
    let env_json = inst.env.as_ref().map(|e| {
        let mut encrypted = e.clone();
        encrypt_env_values(&mut encrypted);
        serde_json::to_string(&encrypted).unwrap_or_default()
    });
    conn.execute(
        "INSERT INTO mcp_instances (id, server_id, name, enabled, transport, command, args_json, env_json, url, installed_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            inst.id,
            inst.server_id,
            inst.name,
            inst.enabled as i64,
            inst.transport,
            inst.command,
            args_json,
            env_json,
            inst.url,
            inst.installed_at,
        ],
    )?;
    Ok(())
}

/// Delete an MCP instance by ID.
pub fn delete_mcp_instance(conn: &Connection, id: &str) -> Result<(), AppError> {
    log::info!("RS::delete_mcp_instance | id={}", id);
    conn.execute("DELETE FROM mcp_instances WHERE id = ?1", params![id])?;
    Ok(())
}

/// Toggle an MCP instance enabled/disabled.
pub fn toggle_mcp_instance(conn: &Connection, id: &str, enabled: bool) -> Result<(), AppError> {
    log::info!("RS::toggle_mcp_instance | id={} enabled={}", id, enabled);
    conn.execute(
        "UPDATE mcp_instances SET enabled = ?1 WHERE id = ?2",
        params![enabled as i64, id],
    )?;
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
            "CREATE TABLE IF NOT EXISTS mcp_categories (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                icon TEXT NOT NULL DEFAULT ''
            );
            CREATE TABLE IF NOT EXISTS mcp_servers (
                id TEXT PRIMARY KEY,
                category_id TEXT NOT NULL,
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                publisher TEXT NOT NULL DEFAULT '',
                registry_type TEXT NOT NULL DEFAULT 'npm',
                identifier TEXT NOT NULL,
                transport TEXT NOT NULL DEFAULT 'stdio',
                env_vars_json TEXT,
                args_json TEXT,
                github_stars INTEGER
            );
            CREATE TABLE IF NOT EXISTS mcp_instances (
                id TEXT PRIMARY KEY,
                server_id TEXT NOT NULL,
                name TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 0,
                transport TEXT NOT NULL DEFAULT 'stdio',
                command TEXT,
                args_json TEXT,
                env_json TEXT,
                url TEXT,
                installed_at INTEGER NOT NULL
            );",
        ).unwrap();
        conn
    }

    fn seed_mcp_data(conn: &Connection) {
        // Insert categories
        conn.execute("INSERT INTO mcp_categories (id, name, icon) VALUES ('fs', '文件系统', '📁')", []).unwrap();
        conn.execute("INSERT INTO mcp_categories (id, name, icon) VALUES ('search', '搜索引擎', '🔍')", []).unwrap();

        // Insert servers
        conn.execute(
            "INSERT INTO mcp_servers (id, category_id, name, description, publisher, registry_type, identifier, transport, env_vars_json, args_json, github_stars) \
             VALUES ('filesystem', 'fs', 'Filesystem', '读写本地文件', 'Anthropic', 'npm', '@modelcontextprotocol/server-filesystem', 'stdio', NULL, NULL, 75000)",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO mcp_servers (id, category_id, name, description, publisher, registry_type, identifier, transport, env_vars_json, args_json, github_stars) \
             VALUES ('brave-search', 'search', 'Brave Search', 'Brave 搜索引擎', 'Anthropic', 'npm', '@modelcontextprotocol/server-brave-search', 'stdio', \
             '[{\"name\":\"BRAVE_API_KEY\",\"description\":\"API Key\",\"required\":true,\"secret\":true}]', NULL, 75000)",
            [],
        ).unwrap();
    }

    #[test]
    fn mcp_list_categories() {
        let conn = setup_db();
        seed_mcp_data(&conn);

        let cats = list_mcp_categories(&conn).unwrap();
        assert_eq!(cats.len(), 2);
        // ORDER BY name: "搜索引擎" < "文件系统" (Unicode order)
        assert_eq!(cats[0].id, "search");
        assert_eq!(cats[1].id, "fs");
    }

    #[test]
    fn mcp_list_servers_all() {
        let conn = setup_db();
        seed_mcp_data(&conn);

        let servers = list_mcp_servers(&conn, None).unwrap();
        assert_eq!(servers.len(), 2);
    }

    #[test]
    fn mcp_list_servers_by_category() {
        let conn = setup_db();
        seed_mcp_data(&conn);

        let servers = list_mcp_servers(&conn, Some("fs")).unwrap();
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].name, "Filesystem");
    }

    #[test]
    fn mcp_instance_crud() {
        let conn = setup_db();

        // Create
        let inst = McpInstance {
            id: "inst-1".into(),
            server_id: "filesystem".into(),
            name: "我的文件".into(),
            enabled: false,
            transport: "stdio".into(),
            command: Some("npx".into()),
            args: Some(vec!["-y".into(), "@modelcontextprotocol/server-filesystem".into(), "/docs".into()]),
            env: None,
            url: None,
            installed_at: 1000,
        };
        create_mcp_instance(&conn, &inst).unwrap();

        // Read
        let list = list_mcp_instances(&conn).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "我的文件");
        assert!(!list[0].enabled);

        // Toggle
        toggle_mcp_instance(&conn, "inst-1", true).unwrap();
        let inst = get_mcp_instance(&conn, "inst-1").unwrap().unwrap();
        assert!(inst.enabled);

        // Delete
        delete_mcp_instance(&conn, "inst-1").unwrap();
        let list = list_mcp_instances(&conn).unwrap();
        assert_eq!(list.len(), 0);
    }
}
