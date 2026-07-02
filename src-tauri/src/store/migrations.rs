use rusqlite::{params, Connection};

use crate::error::AppError;

/// Run all database migrations: create tables, apply ALTER TABLE, seed data.
pub fn run_migrations(conn: &Connection) -> Result<(), AppError> {
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

    // 迁移：为 messages 表添加 thinking_content 列（思维链内容）
    let _ = conn.execute(
        "ALTER TABLE messages ADD COLUMN thinking_content TEXT",
        [],
    );

    // 迁移：为 conversation_configs 表添加 search_engine 列（搜索引擎 server_id）
    let _ = conn.execute(
        "ALTER TABLE conversation_configs ADD COLUMN search_engine TEXT NOT NULL DEFAULT ''",
        [],
    );

    // 迁移：为 messages 表添加 attachments 列（JSON 数组，存储附件元数据+内容）
    let _ = conn.execute(
        "ALTER TABLE messages ADD COLUMN attachments TEXT",
        [],
    );

    // Seed built-in MCP registry data (skip if already populated).
    seed_mcp_registry(conn)?;

    Ok(())
}

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
    type ServerEntry<'a> = (&'a str, &'a str, &'a str, &'a str, &'a str, &'a str, &'a str, Option<&'a str>, Option<&'a str>, i64);
    let servers: Vec<ServerEntry> = vec![
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
        ("search", "tavily-search", "Tavily Search", "Tavily AI 搜索引擎，实时高质量搜索（免费 1000 次/月）", "Tavily", "npm", "tavily-mcp",
         Some(r#"[{"name":"TAVILY_API_KEY","description":"Tavily API Key（在 tavily.com 获取，免费注册）","required":true,"secret":true}]"#), None, 50000),
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
