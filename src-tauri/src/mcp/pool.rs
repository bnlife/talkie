use std::collections::HashMap;
use std::path::PathBuf;

use tokio::sync::Mutex;

use crate::mcp::client::McpClient;
use crate::models::McpInstance;

/// Manages all running MCP server processes.
pub struct McpPool {
    clients: Mutex<HashMap<String, McpClient>>,
    app_data_dir: PathBuf,
}

impl McpPool {
    /// Create a new empty pool.
    pub fn new(app_data_dir: PathBuf) -> Self {
        Self {
            clients: Mutex::new(HashMap::new()),
            app_data_dir,
        }
    }

    /// Start an MCP instance: spawn process, initialize, store in pool.
    pub async fn start(&self, instance: &McpInstance) -> Result<(), String> {
        let mut clients = self.clients.lock().await;
        if clients.contains_key(&instance.id) {
            log::debug!("RS::mcp::pool | start | already running | id={}", instance.id);
            return Err(format!("MCP 实例已在运行: {}", instance.id));
        }

        // Resolve local: scripts to actual paths
        let mut resolved = instance.clone();
        if instance.server_id.starts_with("local:") {
            let name = instance.server_id.strip_prefix("local:").unwrap_or(&instance.server_id);
            let script_path = self.app_data_dir.join("mcp-servers").join(name).join("index.js");
            resolved.command = Some("node".to_string());
            resolved.args = Some(vec![script_path.to_string_lossy().to_string()]);
            log::info!("RS::mcp::pool | local script | path={}", script_path.display());
        }

        log::info!("RS::mcp::pool | start | spawning | id={} name={}", instance.id, instance.name);
        let client = McpClient::connect_stdio(&resolved).await?;
        log::info!("RS::mcp::pool | start | connected | id={}", instance.id);
        clients.insert(instance.id.clone(), client);
        log::info!("RS::mcp::pool | start | done | id={} | pool size={}", instance.id, clients.len());
        Ok(())
    }

    /// Stop a running instance: remove from pool (drop will cleanup).
    pub async fn stop(&self, id: &str) -> Result<(), String> {
        let mut clients = self.clients.lock().await;
        if clients.remove(id).is_some() {
            log::info!("RS::mcp::pool | stopped | id={}", id);
        } else {
            log::debug!("RS::mcp::pool | not running | id={}", id);
        }
        Ok(())
    }

    /// Check if an instance is running.
    pub async fn is_running(&self, id: &str) -> bool {
        self.clients.lock().await.contains_key(id)
    }

    /// List tools for a running instance.
    pub async fn list_tools(&self, id: &str) -> Result<Vec<rmcp::model::Tool>, String> {
        log::debug!("RS::mcp::pool | list_tools | id={}", id);
        let clients = self.clients.lock().await;
        log::debug!("RS::mcp::pool | list_tools | pool size={} | id={}", clients.len(), id);
        let client = clients.get(id).ok_or(format!("MCP 实例未运行: {}", id))?;
        client.list_tools().await
    }

    /// Call a tool on a running instance.
    pub async fn call_tool(&self, instance_id: &str, tool_name: &str, args: serde_json::Value) -> Result<serde_json::Value, String> {
        let clients = self.clients.lock().await;
        let client = clients.get(instance_id).ok_or(format!("MCP 实例未运行: {}", instance_id))?;
        client.call_tool(tool_name, args).await
    }
}
