pub mod chat;
pub mod migrations;
pub mod mcp;
pub mod prompt;

use std::path::PathBuf;

use rusqlite::Connection;

use crate::error::AppError;

// Re-export all public functions so external callers can use `store::xxx` unchanged.
pub use chat::{
    batch_create_messages, create_conversation, create_message, delete_conversation,
    delete_message, delete_messages_by_conversation, get_conversation, list_conversations,
    list_messages_by_conversation, list_messages_paginated, pin_conversation, unpin_conversation,
    update_conversation, update_conversation_config,
};
pub use mcp::{
    create_mcp_instance, delete_mcp_instance, get_mcp_instance, list_mcp_categories,
    list_mcp_instances, list_mcp_servers, toggle_mcp_instance,
};
pub use prompt::{
    create_prompt, delete_prompt, get_default_prompt, get_prompt_by_id, list_prompts,
    set_default_prompt, update_prompt,
};

/// Open or create the SQLite database and ensure all tables exist.
///
/// Foreign key constraints are enabled automatically.
pub fn init(db_path: &PathBuf) -> Result<Connection, AppError> {
    log::info!("RS::init | path={:?}", db_path);
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let conn = Connection::open(db_path)?;
    migrations::run_migrations(&conn)?;

    log::info!("RS::init | ok");
    Ok(conn)
}
