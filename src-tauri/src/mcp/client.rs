use rmcp::model::CallToolRequestParams;
use rmcp::service::{RunningService, RoleClient};
use rmcp::transport::TokioChildProcess;
use rmcp::ServiceExt;
use serde_json::Value;
use tokio::process::Command;

use crate::models::McpInstance;

/// MCP client wrapper using rmcp SDK.
pub struct McpClient {
    service: RunningService<RoleClient, ()>,
}

impl McpClient {
    /// Connect to an MCP server via stdio (child process).
    pub async fn connect_stdio(instance: &McpInstance) -> Result<Self, String> {
        log::info!("RS::mcp::client | connect_stdio | id={} name={}", instance.id, instance.name);

        let (cmd, args) = match instance.transport.as_str() {
            "stdio" => {
                let command = instance.command.as_deref().unwrap_or("npx");
                let args = instance.args.clone().unwrap_or_default();
                (command.to_string(), args)
            }
            _ => return Err(format!("不支持的传输方式: {}", instance.transport)),
        };

        log::info!("RS::mcp::client | cmd={} args={:?}", cmd, args);

        let mut command = Command::new(&cmd);
        command.args(&args);

        // Set environment variables from instance config
        if let Some(ref env) = instance.env {
            for (key, value) in env {
                command.env(key, value);
            }
        }

        let process = TokioChildProcess::new(command)
            .map_err(|e| {
                log::error!("RS::mcp::client | spawn err: {}", e);
                format!("MCP 进程启动失败: {}", e)
            })?;

        let service = ().serve(process).await
            .map_err(|e| {
                log::error!("RS::mcp::client | serve err: {}", e);
                format!("MCP 服务启动失败: {}", e)
            })?;

        log::info!("RS::mcp::client | connected | id={}", instance.id);
        Ok(Self { service })
    }

    /// List available tools from this MCP server.
    pub async fn list_tools(&self) -> Result<Vec<rmcp::model::Tool>, String> {
        log::debug!("RS::mcp::client | list_tools");

        let tools = self.service.list_all_tools().await
            .map_err(|e| {
                log::error!("RS::mcp::client | list_tools err: {}", e);
                format!("列出工具失败: {}", e)
            })?;

        log::debug!("RS::mcp::client | list_tools | count={}", tools.len());
        Ok(tools)
    }

    /// Call a specific tool with arguments.
    pub async fn call_tool(&self, name: &str, args: Value) -> Result<Value, String> {
        log::debug!("RS::mcp::client | call_tool | name={}", name);

        let request = match args {
            Value::Object(map) => CallToolRequestParams::new(name.to_string()).with_arguments(map),
            _ => CallToolRequestParams::new(name.to_string()),
        };

        let result = self.service.call_tool(request).await
            .map_err(|e| {
                log::error!("RS::mcp::client | call_tool err: {}", e);
                format!("调用工具失败: {}", e)
            })?;

        let value = serde_json::to_value(&result)
            .map_err(|e| {
                log::error!("RS::mcp::client | serialize err: {}", e);
                format!("序列化结果失败: {}", e)
            })?;

        log::debug!("RS::mcp::client | call_tool | done");
        Ok(value)
    }
}
