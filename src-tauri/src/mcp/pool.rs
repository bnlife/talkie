use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::mcp::jsonrpc::McpTool;
use crate::mcp::runtime::McpProcess;
use crate::models::McpInstance;

/// Manages all running MCP server processes.
pub struct McpPool {
    processes: Mutex<HashMap<String, McpProcess>>,
    app_data_dir: PathBuf,
}

impl McpPool {
    /// Create a new empty pool.
    pub fn new(app_data_dir: PathBuf) -> Self {
        Self {
            processes: Mutex::new(HashMap::new()),
            app_data_dir,
        }
    }

    /// Start an MCP instance: spawn process, initialize, store in pool.
    pub fn start(&self, instance: &McpInstance) -> Result<(), String> {
        let mut procs = self.processes.lock().map_err(|e| e.to_string())?;
        if procs.contains_key(&instance.id) {
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
        let proc = McpProcess::spawn(&resolved)?;
        log::info!("RS::mcp::pool | start | spawned ok | id={} | inserting into pool", instance.id);
        procs.insert(instance.id.clone(), proc);
        log::info!("RS::mcp::pool | start | done | id={} | pool size={}", instance.id, procs.len());
        Ok(())
    }

    /// Stop a running instance: shutdown process, remove from pool.
    pub fn stop(&self, id: &str) -> Result<(), String> {
        let mut procs = self.processes.lock().map_err(|e| e.to_string())?;
        if let Some(mut proc) = procs.remove(id) {
            proc.shutdown();
            log::info!("RS::mcp::pool | stopped | id={}", id);
            Ok(())
        } else {
            log::debug!("RS::mcp::pool | not running | id={}", id);
            Ok(())
        }
    }

    /// Check if an instance is running.
    pub fn is_running(&self, id: &str) -> bool {
        self.processes.lock().map_or(false, |p| p.contains_key(id))
    }

    /// List tools for a running instance.
    pub fn list_tools(&self, id: &str) -> Result<Vec<McpTool>, String> {
        log::debug!("RS::mcp::pool | list_tools | id={}", id);
        let mut procs = self.processes.lock().map_err(|e| e.to_string())?;
        log::debug!("RS::mcp::pool | list_tools | pool size={} | id={}", procs.len(), id);
        for key in procs.keys() {
            log::debug!("RS::mcp::pool | list_tools | pool has: {}", key);
        }
        let proc = procs.get_mut(id).ok_or(format!("MCP 实例未运行: {}", id))?;
        proc.list_tools()
    }

    /// Call a tool on a running instance.
    pub fn call_tool(&self, instance_id: &str, tool_name: &str, args: serde_json::Value) -> Result<serde_json::Value, String> {
        let mut procs = self.processes.lock().map_err(|e| e.to_string())?;
        let proc = procs.get_mut(instance_id).ok_or(format!("MCP 实例未运行: {}", instance_id))?;
        proc.call_tool(tool_name, args)
    }
}
