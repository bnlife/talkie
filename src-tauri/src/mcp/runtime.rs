use std::process::{Child, ChildStdin, Command, Stdio};
use std::io::{BufRead, BufReader, Write};
use std::sync::{Arc, Mutex, mpsc};
use std::thread::JoinHandle;
use std::time::Duration;

use crate::mcp::jsonrpc::{self, JsonRpcRequest, JsonRpcResponse, McpTool};
use crate::models::McpInstance;

/// A running MCP server process with stdio JSON-RPC communication.
pub struct McpProcess {
    pub id: String,
    child: Child,
    stdin: ChildStdin,
    reader_rx: mpsc::Receiver<Result<String, String>>,
    reader_handle: Option<JoinHandle<()>>,
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

        // Persistent stdout reader thread: runs for the lifetime of the process
        let (reader_tx, reader_rx) = mpsc::channel::<Result<String, String>>();
        let reader_handle = std::thread::spawn(move || {
            let mut reader = stdout;
            loop {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) => { let _ = reader_tx.send(Err("EOF".to_string())); break; }
                    Ok(_) => { if reader_tx.send(Ok(line)).is_err() { break; } }
                    Err(e) => { let _ = reader_tx.send(Err(e.to_string())); break; }
                }
            }
        });

        let mut proc = McpProcess {
            id: instance.id.clone(),
            child,
            stdin,
            reader_rx,
            reader_handle: Some(reader_handle),
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

        log::debug!("RS::mcp::call | method={} id={} | sending request", method, id);
        let req = JsonRpcRequest::new(id, method, params);
        self.send_raw(&req.to_message())?;
        log::debug!("RS::mcp::call | method={} id={} | request sent, waiting for response", method, id);

        let result = self.read_response(id);
        log::debug!("RS::mcp::call | method={} id={} | result={:?}", method, id, result.is_ok());
        result
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

    /// Read a JSON-RPC response with the expected id (with 30s timeout).
    fn read_response(&mut self, expected_id: i64) -> Result<JsonRpcResponse, String> {
        let timeout = Duration::from_secs(30);
        let start = std::time::Instant::now();
        log::debug!("RS::mcp::read_response | expected_id={} | waiting (30s timeout)", expected_id);

        loop {
            match self.reader_rx.recv_timeout(timeout) {
                Ok(Ok(line)) => {
                    let trimmed = line.trim().to_string();
                    log::debug!("RS::mcp::read_response | got line (elapsed={:.1}s, len={})", start.elapsed().as_secs_f64(), trimmed.len());
                    if trimmed.is_empty() { continue; }
                    log::trace!("RS::mcp | <<< {}", trimmed);
                    match jsonrpc::parse_response(&trimmed) {
                        Ok(resp) if resp.id == Some(expected_id) => {
                            log::debug!("RS::mcp::read_response | matched id={} (elapsed={:.1}s)", expected_id, start.elapsed().as_secs_f64());
                            return Ok(resp);
                        }
                        Ok(resp) => {
                            log::debug!("RS::mcp | skip non-target resp | id={:?}", resp.id);
                        }
                        Err(e) => {
                            log::warn!("RS::mcp | parse: {}", e);
                        }
                    }
                }
                Ok(Err(e)) if e == "EOF" => {
                    let stderr = self.get_stderr();
                    let stderr_trimmed = stderr.trim();
                    log::error!("RS::mcp::read_response | EOF (elapsed={:.1}s) stderr={}", start.elapsed().as_secs_f64(), stderr_trimmed);
                    if stderr_trimmed.is_empty() {
                        return Err("MCP 进程已关闭 (EOF)，stderr 无输出".to_string());
                    } else {
                        return Err(format!("MCP 进程已关闭 (EOF)，stderr: {}", stderr_trimmed));
                    }
                }
                Ok(Err(e)) => {
                    log::error!("RS::mcp::read_response | read error: {} (elapsed={:.1}s)", e, start.elapsed().as_secs_f64());
                    return Err(format!("读取 stdout 失败: {}", e));
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    log::error!("RS::mcp::read_response | TIMEOUT (30s) | id={}", self.id);
                    return Err("MCP 响应超时 (30秒)，进程可能未正确启动".to_string());
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    log::error!("RS::mcp::read_response | channel disconnected (elapsed={:.1}s)", start.elapsed().as_secs_f64());
                    return Err("MCP 读取线程已退出".to_string());
                }
            }
        }
    }

    /// Gracefully shut down the process.
    pub fn shutdown(&mut self) {
        log::info!("RS::mcp::shutdown | id={}", self.id);
        let _ = self.child.kill();
        let _ = self.child.wait();
        // Wait for reader thread to finish (pipe closes after kill, read_line returns)
        if let Some(handle) = self.reader_handle.take() {
            let _ = handle.join();
        }
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
