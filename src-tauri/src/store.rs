use std::path::PathBuf;

use rusqlite::{params, Connection};

use crate::error::AppError;
use crate::models::{Conversation, ConversationConfig, ConversationView, McpCategory, McpInstance, McpServer, Message, Prompt};

/// Open or create the SQLite database and ensure all tables exist.
///
/// Foreign key constraints are enabled automatically.
pub fn init(db_path: &PathBuf) -> Result<Connection, AppError> {
    log::info!("RS::init | path={:?}", db_path);
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let conn = Connection::open(db_path)?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
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
            FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
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
        );
        CREATE TABLE IF NOT EXISTS mcp_categories (
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
            github_stars INTEGER,
            FOREIGN KEY (category_id) REFERENCES mcp_categories(id)
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
    )?;

    // 迁移：为旧数据库添加 pinned 列（如已存在则忽略）
    let _ = conn.execute(
        "ALTER TABLE conversations ADD COLUMN pinned INTEGER NOT NULL DEFAULT 0",
        [],
    );

    // 迁移：确保 conversation_configs 表存在（旧数据库可能没有）
    let _ = conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS conversation_configs (
            conversation_id TEXT PRIMARY KEY,
            provider_id TEXT NOT NULL DEFAULT '',
            model TEXT NOT NULL DEFAULT '',
            prompt_id TEXT,
            search_enabled INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
        );",
    );

    // 迁移：将旧 conversations 表中的 provider_id/model/search_enabled 数据迁移到 conversation_configs
    // 只迁移还没有 config 记录的对话
    let _ = conn.execute(
        "INSERT OR IGNORE INTO conversation_configs (conversation_id, provider_id, model, search_enabled)
         SELECT id, COALESCE(provider_id, ''), COALESCE(model, ''), COALESCE(search_enabled, 0)
         FROM conversations
         WHERE id NOT IN (SELECT conversation_id FROM conversation_configs)",
        [],
    );

    // 迁移：为 messages 表添加 search_results 列（JSON 数组，存储搜索来源）
    let _ = conn.execute(
        "ALTER TABLE messages ADD COLUMN search_results TEXT",
        [],
    );

    // Seed built-in MCP registry data (skip if already populated).
    seed_mcp_registry(&conn)?;

    log::info!("RS::init | ok");
    Ok(conn)
}

// ---------------------------------------------------------------------------
// MCP registry seed
// ---------------------------------------------------------------------------

/// Seed built-in MCP categories and servers. Idempotent (INSERT OR IGNORE).
fn seed_mcp_registry(conn: &Connection) -> Result<(), AppError> {
    log::debug!("RS::seed_mcp_registry | start");

    // Categories
    let categories: Vec<(&str, &str, &str)> = vec![
        ("filesystem", "文件系统", "📁"),
        ("database", "数据库", "🗄"),
        ("search", "搜索引擎", "🔍"),
        ("devtools", "开发工具", "💻"),
        ("cloud", "云服务", "☁"),
        ("comms", "通讯", "💬"),
        ("productivity", "生产力", "⚡"),
    ];
    for (id, name, icon) in &categories {
        conn.execute(
            "INSERT OR IGNORE INTO mcp_categories (id, name, icon) VALUES (?1, ?2, ?3)",
            params![id, name, icon],
        )?;
    }

    // Servers (built-in popular ones)
    let servers: Vec<(&str, &str, &str, &str, &str, &str, &str, Option<&str>, Option<&str>, i64)> = vec![
        ("filesystem", "filesystem", "Filesystem", "读写本地文件、搜索内容、目录管理", "Anthropic", "npm", "@modelcontextprotocol/server-filesystem",
         None, Some(r#"[{"type":"positional","valueHint":"target_dir","description":"访问路径","required":true,"repeated":true}]"#), 75000),
        ("filesystem", "memory", "Memory", "知识图谱持久化存储", "Anthropic", "npm", "@modelcontextprotocol/server-memory",
         None, None, 75000),
        ("database", "sqlite", "SQLite", "查询和管理 SQLite 数据库", "Anthropic", "npm", "@modelcontextprotocol/server-sqlite",
         None, Some(r#"[{"type":"positional","valueHint":"db_path","description":"数据库文件路径","required":true}]"#), 75000),
        ("database", "postgres", "PostgreSQL", "连接 PostgreSQL 数据库", "Anthropic", "npm", "@modelcontextprotocol/server-postgres",
         Some(r#"[{"name":"DATABASE_URL","description":"PostgreSQL 连接字符串","required":true,"secret":true}]"#), None, 75000),
        ("search", "brave-search", "Brave Search", "使用 Brave 搜索引擎搜索网页", "Anthropic", "npm", "@modelcontextprotocol/server-brave-search",
         Some(r#"[{"name":"BRAVE_API_KEY","description":"Brave Search API Key","required":true,"secret":true}]"#), None, 75000),
        ("search", "duckduckgo", "DuckDuckGo", "使用 DuckDuckGo 搜索网页（无需 API Key）", "Community", "npm", "@nickclyde/duckduckgo-mcp-server",
         None, None, 2000),
        ("search", "local:bocha-search", "博查搜索", "博查 AI 搜索引擎，中国可用（自建 MCP server）", "本地", "local", "local:bocha-search",
         Some(r#"[{"name":"BOCHA_API_KEY","description":"博查 API Key（在 open.bochaai.com 获取）","required":true,"secret":true}]"#), None, 100),
        ("devtools", "git", "Git", "Git 仓库操作：提交、分支、diff", "Anthropic", "npm", "@modelcontextprotocol/server-git",
         None, Some(r#"[{"type":"positional","valueHint":"repo_path","description":"Git 仓库路径","required":true}]"#), 75000),
        ("devtools", "github", "GitHub", "GitHub API：仓库、Issue、PR", "Anthropic", "npm", "@modelcontextprotocol/server-github",
         Some(r#"[{"name":"GITHUB_TOKEN","description":"GitHub Personal Access Token","required":true,"secret":true}]"#), None, 75000),
        ("devtools", "playwright", "Playwright", "浏览器自动化测试", "Microsoft", "npm", "@microsoft/mcp-server-playwright",
         None, None, 5000),
        ("devtools", "puppeteer", "Puppeteer", "Chrome 浏览器自动化", "Anthropic", "npm", "@modelcontextprotocol/server-puppeteer",
         None, None, 75000),
        ("devtools", "docker", "Docker", "Docker 容器管理", "Community", "npm", "@modelcontextprotocol/server-docker",
         None, None, 3000),
        ("cloud", "aws", "AWS", "AWS 云服务操作", "Community", "npm", "@modelcontextprotocol/server-aws",
         Some(r#"[{"name":"AWS_ACCESS_KEY_ID","description":"AWS Access Key","required":true,"secret":false},{"name":"AWS_SECRET_ACCESS_KEY","description":"AWS Secret Key","required":true,"secret":true}]"#), None, 2000),
        ("comms", "slack", "Slack", "Slack 消息和频道管理", "Anthropic", "npm", "@modelcontextprotocol/server-slack",
         Some(r#"[{"name":"SLACK_BOT_TOKEN","description":"Slack Bot Token","required":true,"secret":true},{"name":"SLACK_TEAM_ID","description":"Slack Team ID","required":true,"secret":false}]"#), None, 75000),
        ("productivity", "fetch", "Fetch", "HTTP 请求工具，抓取网页内容", "Anthropic", "npm", "@modelcontextprotocol/server-fetch",
         None, None, 75000),
        ("productivity", "context7", "Context7", "实时 API 文档注入（开发辅助）", "Upstash", "npm", "@upstash/context7-mcp",
         None, None, 48000),
    ];
    for (cat_id, id, name, desc, pub_name, reg_type, ident, env, args, stars) in &servers {
        conn.execute(
            "INSERT OR IGNORE INTO mcp_servers (id, category_id, name, description, publisher, registry_type, identifier, transport, env_vars_json, args_json, github_stars) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'stdio', ?8, ?9, ?10)",
            params![id, cat_id, name, desc, pub_name, reg_type, ident, env, args, stars],
        )?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Conversation CRUD
// ---------------------------------------------------------------------------

/// List all conversations with their configs, ordered by most recently updated first.
pub fn list_conversations(conn: &Connection) -> Result<Vec<ConversationView>, AppError> {
    log::debug!("RS::list_conversations");
    let mut stmt = conn.prepare(
        "SELECT c.id, c.title, c.created_at, c.updated_at, c.pinned,
                COALESCE(cfg.provider_id, ''), COALESCE(cfg.model, ''), cfg.prompt_id, COALESCE(cfg.search_enabled, 0)
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
        "INSERT INTO conversation_configs (conversation_id, provider_id, model, prompt_id, search_enabled) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![config.conversation_id, config.provider_id, config.model, config.prompt_id, config.search_enabled as i64],
    )?;
    Ok(())
}

/// Retrieve a single conversation with its config by ID.
pub fn get_conversation(conn: &Connection, id: &str) -> Result<Option<ConversationView>, AppError> {
    log::debug!("RS::get_conversation | id={}", id);
    let mut stmt = conn.prepare(
        "SELECT c.id, c.title, c.created_at, c.updated_at, c.pinned,
                COALESCE(cfg.provider_id, ''), COALESCE(cfg.model, ''), cfg.prompt_id, COALESCE(cfg.search_enabled, 0)
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
        "UPDATE conversation_configs SET provider_id = ?1, model = ?2, prompt_id = ?3, search_enabled = ?4 WHERE conversation_id = ?5",
        params![config.provider_id, config.model, config.prompt_id, config.search_enabled as i64, config.conversation_id],
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

// ---------------------------------------------------------------------------
// Message CRUD
// ---------------------------------------------------------------------------

/// List all messages belonging to a conversation, ordered chronologically.
pub fn list_messages_by_conversation(
    conn: &Connection,
    conversation_id: &str,
) -> Result<Vec<Message>, AppError> {
    log::debug!("RS::list_messages | conv={}", conversation_id);
    let mut stmt = conn.prepare(
        "SELECT id, conversation_id, role, content, created_at, token_count, search_results \
         FROM messages WHERE conversation_id = ?1 ORDER BY created_at ASC",
    )?;
    let rows = stmt.query_map(params![conversation_id], |row| {
        let search_results_json: Option<String> = row.get(6)?;
        let search_results = search_results_json
            .and_then(|json| serde_json::from_str::<Vec<crate::models::SearchResult>>(&json).ok());
        Ok(Message {
            id: row.get(0)?,
            conversation_id: row.get(1)?,
            role: row.get(2)?,
            content: row.get(3)?,
            created_at: row.get(4)?,
            token_count: row.get(5)?,
            search_results,
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
    log::debug!("RS::create_message | conv={} role={}", message.conversation_id, message.role);
    let search_results_json = message.search_results.as_ref()
        .map(|sr| serde_json::to_string(sr).unwrap_or_default());
    conn.execute(
        "INSERT INTO messages (id, conversation_id, role, content, created_at, token_count, search_results) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            message.id,
            message.conversation_id,
            message.role,
            message.content,
            message.created_at,
            message.token_count,
            search_results_json,
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
        tx.execute(
            "INSERT INTO messages (id, conversation_id, role, content, created_at, token_count, search_results) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                msg.id,
                msg.conversation_id,
                msg.role,
                msg.content,
                msg.created_at,
                msg.token_count,
                search_results_json,
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

// ---------------------------------------------------------------------------
// Prompt CRUD
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// MCP CRUD
// ---------------------------------------------------------------------------

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
        Some(Ok(inst)) => Ok(Some(inst)),
        Some(Err(e)) => Err(AppError::DbError(e.to_string())),
        None => Ok(None),
    }
}

/// Create (install) a new MCP instance.
pub fn create_mcp_instance(conn: &Connection, inst: &McpInstance) -> Result<(), AppError> {
    log::info!("RS::create_mcp_instance | id={} name={}", inst.id, inst.name);
    let args_json = inst.args.as_ref().map(|a| serde_json::to_string(a).unwrap_or_default());
    let env_json = inst.env.as_ref().map(|e| serde_json::to_string(e).unwrap_or_default());
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
            "            CREATE TABLE IF NOT EXISTS conversations (
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
                FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS prompts (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                content TEXT NOT NULL,
                is_default INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS mcp_categories (
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

        // Create a conversation with no prompt_id
        create_conversation(&conn,
            &Conversation { id: "c1".into(), title: "test".into(), created_at: 0, updated_at: 0, pinned: false },
            &ConversationConfig { conversation_id: "c1".into(), provider_id: "".into(), model: "gpt-4".into(), prompt_id: None, search_enabled: false },
        ).unwrap();

        // Create a default prompt
        create_prompt(&conn, &Prompt {
            id: "p1".into(), name: "翻译".into(), content: "你是翻译AI助手".into(),
            is_default: false, created_at: 0, updated_at: 0,
        }).unwrap();
        set_default_prompt(&conn, "p1").unwrap();

        // Simulate the new injection logic: prompt_id → default → none
        let conv = get_conversation(&conn, "c1").unwrap().unwrap();
        let system_prompt = match &conv.prompt_id {
            Some(id) if id == "default" => get_default_prompt(&conn).unwrap().map(|p| p.content),
            Some(id) => get_prompt_by_id(&conn, id).unwrap().map(|p| p.content),
            None => get_default_prompt(&conn).unwrap().map(|p| p.content),
        };

        assert!(system_prompt.is_some());
        assert_eq!(system_prompt.unwrap(), "你是翻译AI助手");
    }

    #[test]
    fn system_prompt_injection_uses_prompt_id() {
        let conn = setup_db();

        // Create a conversation with prompt_id pointing to a specific prompt
        create_conversation(&conn,
            &Conversation { id: "c1".into(), title: "test".into(), created_at: 0, updated_at: 0, pinned: false },
            &ConversationConfig { conversation_id: "c1".into(), provider_id: "".into(), model: "gpt-4".into(), prompt_id: Some("p2".into()), search_enabled: false },
        ).unwrap();

        // Create two prompts
        create_prompt(&conn, &Prompt {
            id: "p1".into(), name: "翻译".into(), content: "默认提示词".into(),
            is_default: true, created_at: 0, updated_at: 0,
        }).unwrap();
        create_prompt(&conn, &Prompt {
            id: "p2".into(), name: "代码".into(), content: "你是代码助手".into(),
            is_default: false, created_at: 0, updated_at: 0,
        }).unwrap();

        // Simulate the injection logic
        let conv = get_conversation(&conn, "c1").unwrap().unwrap();
        let system_prompt = match &conv.prompt_id {
            Some(id) if id == "default" => get_default_prompt(&conn).unwrap().map(|p| p.content),
            Some(id) => get_prompt_by_id(&conn, id).unwrap().map(|p| p.content),
            None => get_default_prompt(&conn).unwrap().map(|p| p.content),
        };

        // Should use prompt_id's prompt, not the default
        assert_eq!(system_prompt.unwrap(), "你是代码助手");
    }

    #[test]
    fn conversation_with_provider_id() {
        let conn = setup_db();
        create_conversation(&conn,
            &Conversation { id: "c1".into(), title: "test".into(), created_at: 0, updated_at: 0, pinned: false },
            &ConversationConfig { conversation_id: "c1".into(), provider_id: "prov-1".into(), model: "gpt-4".into(), prompt_id: None, search_enabled: false },
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
            &ConversationConfig { conversation_id: "c-old".into(), provider_id: "".into(), model: "deepseek-chat".into(), prompt_id: None, search_enabled: false },
        ).unwrap();

        let conv = get_conversation(&conn, "c-old").unwrap().unwrap();
        assert_eq!(conv.provider_id, "");
    }

    #[test]
    fn conversation_config_update() {
        let conn = setup_db();
        create_conversation(&conn,
            &Conversation { id: "c1".into(), title: "test".into(), created_at: 0, updated_at: 0, pinned: false },
            &ConversationConfig { conversation_id: "c1".into(), provider_id: "prov-1".into(), model: "gpt-4".into(), prompt_id: None, search_enabled: false },
        ).unwrap();

        // Update config
        update_conversation_config(&conn, &ConversationConfig {
            conversation_id: "c1".into(),
            provider_id: "prov-2".into(),
            model: "deepseek-chat".into(),
            prompt_id: Some("p1".into()),
            search_enabled: true,
        }).unwrap();

        let conv = get_conversation(&conn, "c1").unwrap().unwrap();
        assert_eq!(conv.provider_id, "prov-2");
        assert_eq!(conv.model, "deepseek-chat");
        assert_eq!(conv.prompt_id, Some("p1".into()));
        assert!(conv.search_enabled);
    }

    // -----------------------------------------------------------------------
    // MCP tests
    // -----------------------------------------------------------------------

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
        };
        create_message(&conn, &msg).unwrap();

        let messages = list_messages_by_conversation(&conn, "conv-nosr").unwrap();
        assert_eq!(messages.len(), 1);
        assert!(messages[0].search_results.is_none());
    }
}
