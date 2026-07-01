use std::process::{Child, ChildStdin, Command, Stdio};
use std::io::{BufRead, BufReader, Write};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use crate::mcp::jsonrpc::{self, JsonRpcRequest, JsonRpcResponse, McpTool};
use crate::models::McpInstance;

/// A running MCP server process with stdio JSON-RPC communication.
pub struct McpProcess {
    pub id: String,
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<std::process::ChildStdout>,
    stderr_collector: Arc<Mutex<String>>,
    stderr_handle: Option<JoinHandle<()>>,
    next_id: i64,
}

impl McpProcess {
    /// Spawn an MCP server process from an instance config.
    /// Returns the process handle after sending `initialize` and `notifications/initialized`.
    pub fn spawn(instance: &McpInstance) -> Result<Self, String> {
        log::info!("RS::mcp::spawn | id={} name={}", instance.id, instance.name);

        let (cmd, args) = match instance.transport.as_str() {
            "stdio" => {
                let command = instance.command.as_deref().unwrap_or("npx");
                let args = instance.args.clone().unwrap_or_default();
                (command.to_string(), args)
            }
            _ => return Err(format!("不支持的传输方式: {}", instance.transport)),
        };

        log::info!("RS::mcp::spawn | cmd={} args={:?}", cmd, args);

        let mut command = Command::new(&cmd);
        command.args(&args);
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        // Set environment variables from instance config
        if let Some(ref env) = instance.env {
            for (key, value) in env {
                command.env(key, value);
            }
        }

        let mut child = command.spawn().map_err(|e| {
            log::error!("RS::mcp::spawn | err={}", e);
            format!("MCP 进程启动失败: {}", e)
        })?;

        let stdin = child.stdin.take().ok_or("无法获取 stdin")?;
        let stdout = child.stdout.take().ok_or("无法获取 stdout")?;
        let stderr = child.stderr.take().ok_or("无法获取 stderr")?;
        let stdout = BufReader::new(stdout);

        // Read stderr in a background thread
        let stderr_collector = Arc::new(Mutex::new(String::new()));
        let stderr_clone = Arc::clone(&stderr_collector);
        let stderr_handle = std::thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                match line {
                    Ok(l) => {
                        log::debug!("RS::mcp | [stderr] {}", l);
                        if let Ok(mut buf) = stderr_clone.lock() {
                            buf.push_str(&l);
                            buf.push('\n');
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        let mut proc = McpProcess {
            id: instance.id.clone(),
            child,
            stdin,
            stdout,
            stderr_collector,
            stderr_handle: Some(stderr_handle),
            next_id: 1,
        };

        // Initialize handshake
        proc.initialize()?;

        log::info!("RS::mcp::spawn | ready | id={}", instance.id);
        Ok(proc)
    }

    /// Get collected stderr output.
    fn get_stderr(&self) -> String {
        self.stderr_collector.lock().map(|s| s.clone()).unwrap_or_default()
    }

    /// Send the initialize request and notifications/initialized.
    fn initialize(&mut self) -> Result<(), String> {
        let resp = self.call("initialize", Some(serde_json::json!({
            "capabilities": {},
            "clientInfo": { "name": "talkie", "version": "0.1.0" }
        })))?;

        if let Some(err) = resp.error {
            return Err(format!("MCP initialize 失败: {}", err.message));
        }

        // Send initialized notification
        let notif = jsonrpc::JsonRpcNotification::new("notifications/initialized", None);
        self.send_raw(&notif.to_message())?;

        Ok(())
    }

    /// Send a JSON-RPC request and wait for the response.
    pub fn call(&mut self, method: &str, params: Option<serde_json::Value>) -> Result<JsonRpcResponse, String> {
        let id = self.next_id;
        self.next_id += 1;

        let req = JsonRpcRequest::new(id, method, params);
        self.send_raw(&req.to_message())?;

        self.read_response(id)
    }

    /// List available tools from this MCP server.
    pub fn list_tools(&mut self) -> Result<Vec<McpTool>, String> {
        let resp = self.call("tools/list", None)?;
        if let Some(err) = resp.error {
            return Err(format!("tools/list 失败: {}", err.message));
        }
        let result = resp.result.ok_or("tools/list 返回空结果")?;
        let tools = result.get("tools").ok_or("tools/list 结果缺少 tools 字段")?;
        serde_json::from_value(tools.clone()).map_err(|e| format!("tools/list 解析失败: {}", e))
    }

    /// Call a specific tool with arguments.
    pub fn call_tool(&mut self, name: &str, args: serde_json::Value) -> Result<serde_json::Value, String> {
        let params = serde_json::json!({
            "name": name,
            "arguments": args,
        });
        let resp = self.call("tools/call", Some(params))?;
        if let Some(err) = resp.error {
            return Err(format!("tools/call 失败: {}", err.message));
        }
        resp.result.ok_or("tools/call 返回空结果".to_string())
    }

    /// Write raw bytes to stdin.
    fn send_raw(&mut self, data: &str) -> Result<(), String> {
        log::trace!("RS::mcp | >>> {}", data.trim());
        self.stdin.write_all(data.as_bytes()).map_err(|e| format!("写入 stdin 失败: {}", e))?;
        self.stdin.flush().map_err(|e| format!("flush stdin 失败: {}", e))?;
        Ok(())
    }

    /// Read a JSON-RPC response with the expected id.
    fn read_response(&mut self, expected_id: i64) -> Result<JsonRpcResponse, String> {
        loop {
            let mut line = String::new();
            let bytes = self.stdout.read_line(&mut line)
                .map_err(|e| format!("读取 stdout 失败: {}", e))?;
            if bytes == 0 {
                // Process exited — read stderr for diagnostics
                let stderr_output = self.get_stderr();
                let stderr_trimmed = stderr_output.trim();
                if stderr_trimmed.is_empty() {
                    return Err("MCP 进程已关闭 (EOF)，stderr 无输出".to_string());
                } else {
                    log::error!("RS::mcp | stderr: {}", stderr_trimmed);
                    return Err(format!("MCP 进程已关闭 (EOF)，stderr: {}", stderr_trimmed));
                }
            }

            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            log::trace!("RS::mcp | <<< {}", line);

            // Try parsing as response
            match jsonrpc::parse_response(line) {
                Ok(resp) => {
                    // Match by id (notifications have no id, skip them)
                    if resp.id == Some(expected_id) {
                        return Ok(resp);
                    }
                    // Otherwise it's a notification or different response, skip
                    log::debug!("RS::mcp | skip non-target resp | id={:?}", resp.id);
                }
                Err(e) => {
                    log::warn!("RS::mcp | {}", e);
                    continue;
                }
            }
        }
    }

    /// Gracefully shut down the process.
    pub fn shutdown(&mut self) {
        log::info!("RS::mcp::shutdown | id={}", self.id);
        let _ = self.child.kill();
        let _ = self.child.wait();
        // Wait for stderr thread to finish
        if let Some(handle) = self.stderr_handle.take() {
            let _ = handle.join();
        }
        let stderr = self.get_stderr();
        if !stderr.trim().is_empty() {
            log::debug!("RS::mcp::shutdown | stderr: {}", stderr.trim());
        }
    }
}

impl Drop for McpProcess {
    fn drop(&mut self) {
        self.shutdown();
    }
}
