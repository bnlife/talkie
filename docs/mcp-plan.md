# MCP 服务管理实现方案

## 一句话

插件市场式 MCP 管理页面：左侧分类浏览 + 右侧卡片列表 + 自动生成配置表单 + 本地缓存/在线 registry 双数据源。

---

## MCP 是什么（简述）

MCP (Model Context Protocol) 让 LLM 调用外部工具。本质：

- **一个 MCP Server = 一个 npm/Python 包**，提供多个 Tool
- **"安装" = 保存一条配置**（命令+参数 或 URL），没有真正的安装过程
- **调用流程**：用户发消息 → AI 返回 tool_calls → 我们的程序转发给 MCP 进程 → 返回结果 → AI 继续生成

两种传输方式：
- **stdio**：本地运行子进程（`npx -y @xxx/server`）
- **HTTP/SSE**：远程 URL 连接

---

## 官方 Registry API

官方提供了结构化 API：`registry.modelcontextprotocol.io`

```
GET /v0.1/servers?limit=100     → 服务器列表
GET /v0.1/servers/{name}/versions/latest  → 服务器详情
```

返回数据包含：
- 服务器元信息（name, description, title, version）
- 包信息（registryType: npm/pypi/cargo, identifier）
- 环境变量定义（name, description, isRequired, isSecret, default, choices）
- 参数定义（positional/named, description, default, isRequired）
- 远程连接信息（url, headers）

**这意味着可以自动生成配置表单，不需要逐个手动维护。**

---

## 页面布局

```
┌─ 左侧 ───────────────────┐  ┌─ 右侧 ──────────────────────────────────────┐
│                           │  │                                              │
│  [+ 添加自定义服务]        │  │  ┌─────────────────────────────────────────┐ │
│                           │  │  │ 🔍 搜索 MCP 服务...                     │ │
│  ── 市场 ──────────────   │  │  └─────────────────────────────────────────┘ │
│  📁 文件系统       ← 选中 │  │                                              │
│  🗄 数据库                │  │  ── 文件系统 ──────────────────────────────   │
│  🔍 搜索引擎              │  │                                              │
│  💻 开发工具              │  │  ┌──────────────┐ ┌──────────────┐           │
│  ☁ 云服务                 │  │  │ 📁           │ │ 📁           │           │
│  💬 通讯                  │  │  │ Filesystem   │ │ Memory       │           │
│  ⚡ 更多...               │  │  │ Anthropic    │ │ Anthropic    │           │
│                           │  │  │ 读写本地文件  │ │ 知识图谱存储  │           │
│  ───────────────────────  │  │  │ ⭐ 75k       │ │ ⭐ 75k       │           │
│                           │  │  │ [添加] ✓已添加│ │ [添加]       │           │
│  ── 我的服务 ──────────   │  │  └──────────────┘ └──────────────┘           │
│  📁 文件读写  [● 运行中]  │  │                                              │
│  🔀 Git工具   [○ 已暂停]  │  │  ┌──────────────┐ ┌──────────────┐           │
│  🔍 代码搜索  [○ 已暂停]  │  │  │ 📁           │ │ 📁           │           │
│                           │  │  │ S3           │ │ GCS          │           │
│                           │  │  │ Community    │ │ Community    │           │
│                           │  │  │ AWS S3 存储  │ │ Google 云存储 │           │
│                           │  │  │ ⭐ 2k        │ │ ⭐ 1k        │           │
│                           │  │  │ [添加]       │ │ [添加]       │           │
│                           │  │  └──────────────┘ └──────────────┘           │
└───────────────────────────┘  └──────────────────────────────────────────────┘
```

右侧切换逻辑：

| 左侧点击 | 右侧显示 |
|----------|----------|
| 市场分类（如"文件系统"） | 该分类下的 server 卡片列表 |
| 我的服务中的某个服务 | 该服务的详情页（配置/工具列表/状态/移除按钮） |
| "添加自定义服务" | 手动填写配置表单（command/args/url） |

---

## 用户操作流程

### 流程一：从市场添加（本地缓存 + 在线 registry 都走这个流程）

```
用户点击左侧"📁 文件系统"
    │
    ▼
右侧显示该分类下的 server 列表（卡片）
    │
    ├── 数据来源：默认读本地缓存 JSON
    ├── 点击"刷新"按钮：从 registry API 拉取最新，合并到列表
    │
    ▼
用户点击某个 server 卡片上的"添加"按钮
    │
    ▼
弹出配置表单（自动生成）
    │
    ├── 无必填参数（如 Git）→ 直接添加，无需弹窗
    ├── 有必填参数（如文件系统需要路径）→ 显示输入框
    ├── 有密钥（如 Brave Search 需要 API Key）→ 密码输入框
    │
    ▼
用户填写参数，点击"确认添加"
    │
    ▼
配置存入数据库
左侧"我的服务"出现该服务
状态：已暂停
    │
    ▼
用户点击左侧该服务的开关 → 切换为"运行中"
    │
    ▼
Rust 后端 spawn 子进程（npx -y xxx）
MCP 服务启动，等待被调用
```

### 流程二：添加自定义服务（不走市场，用户自己填）

```
用户点击左上角"+ 添加自定义服务"
    │
    ▼
弹出手动配置表单：
    ┌──────────────────────────────────┐
    │  添加自定义 MCP 服务              │
    │                                  │
    │  名称: [                    ]    │
    │                                  │
    │  类型: (●) 本地命令  ( ) 远程 URL │
    │                                  │
    │  -- 本地命令 --                   │
    │  命令: [npx               ]      │
    │  参数: [-y @xxx/server    ]      │
    │  环境变量:                        │
    │    KEY: [        ] VALUE: [    ] │
    │    [+ 添加环境变量]               │
    │                                  │
    │  -- 远程 URL --                   │
    │  地址: [https://...        ]     │
    │                                  │
    │       [取消]    [确认添加]        │
    └──────────────────────────────────┘
    │
    ▼
配置存入数据库
出现在左侧"我的服务"列表
```

### 流程三：管理已安装的服务

```
用户点击左侧"我的服务"中的某个服务
    │
    ▼
右侧显示服务详情页：
    ┌─────────────────────────────────────┐
    │  📁 文件读写                [运行中] │
    │                                     │
    │  ── 基本信息 ────────────────────    │
    │  来源: Anthropic                     │
    │  包名: @modelcontextprotocol/server-fs│
    │  类型: 本地 stdio                    │
    │                                     │
    │  ── 配置 ────────────────────────    │
    │  命令: npx -y @mcp/server-fs        │
    │  参数: /Users/me/docs               │
    │  环境变量: LOG_LEVEL=info           │
    │                                     │
    │  ── 提供的工具 ──────────────────    │
    │  • read_file   读取文件内容          │
    │  • write_file  写入文件              │
    │  • list_dir    列出目录              │
    │  • search      搜索文件              │
    │                                     │
    │  [编辑配置]  [移除服务]              │
    └─────────────────────────────────────┘
```

### 流程四：刷新市场数据

```
用户在市场分类页点击"刷新"按钮
    │
    ▼
调用 registry API: GET /v0.1/servers?limit=100
    │
    ▼
解析返回数据，按分类整理
    │
    ├── 新增的 server → 标记为"新"，追加到列表
    ├── 已有的 server → 更新版本号/描述
    ├── 本地已安装的 server → 标记为"已添加 ✓"
    │
    ▼
列表更新，用户看到最新数据
```

---

## 数据模型

```typescript
// MCP 服务分类（市场左侧）
interface McpCategory {
  id: string          // "filesystem" | "database" | "search" | ...
  name: string        // "文件系统"
  icon: string        // "📁"
}

// MCP 服务定义（市场右侧卡片）
interface McpServer {
  id: string           // "filesystem"
  category_id: string  // "filesystem"
  name: string         // "Filesystem"
  description: string  // "读写本地文件、搜索内容"
  publisher: string    // "Anthropic"
  registry_type: string // "npm" | "pypi" | "cargo"
  identifier: string   // "@modelcontextprotocol/server-filesystem"
  transport: 'stdio' | 'sse' | 'http'
  env_vars?: McpEnvVar[]       // 需要的环境变量定义
  args?: McpArg[]              // 需要的参数定义
  github_stars?: number
}

// 环境变量定义（自动生成表单用）
interface McpEnvVar {
  name: string         // "BRAVE_API_KEY"
  description: string  // "Brave Search API Key"
  required: boolean
  secret: boolean      // 密码框
  default?: string
  choices?: string[]   // 下拉选择
}

// 参数定义（自动生成表单用）
interface McpArg {
  type: 'positional' | 'named'
  name?: string        // named: "--port"
  valueHint?: string   // positional: "target_dir"
  description: string
  required: boolean
  default?: string
  choices?: string[]
  repeated?: boolean   // 可多次传入
}

// 已安装的实例（左侧"我的服务"）
interface McpInstance {
  id: string
  server_id: string    // 关联 McpServer.id，或 null（自定义）
  name: string         // 用户自定义名称
  enabled: boolean     // 启动/暂停
  transport: 'stdio' | 'sse' | 'http'
  command?: string     // stdio: "npx"
  args?: string[]      // stdio: ["-y", "@mcp/server-fs", "/path"]
  env?: Record<string, string>  // 用户填写的环境变量值
  url?: string         // sse/http: "https://..."
  installed_at: number
}
```

---

## 数据来源

### 本地缓存（内置 JSON）

从官方 registry 一次性拉取，整理为分类 JSON 文件：

```
src/data/mcp-registry.json
```

包含 20-30 个常用 server，按分类组织。离线可用。

### 在线 registry API

```
GET https://registry.modelcontextprotocol.io/v0.1/servers?limit=100
```

点击"刷新"按钮时调用，返回数据解析后合并到本地缓存。

### 数据来源策略

| 场景 | 数据来源 |
|------|----------|
| 首次打开市场 | 读本地缓存 JSON |
| 点击"刷新" | 调 registry API → 更新本地缓存 → 刷新页面 |
| 网络不可用 | 只用本地缓存，隐藏"刷新"按钮或显示提示 |

---

## 自动配置表单生成

根据 registry 数据中的 `environmentVariables` 和 `packageArguments` 自动生成表单字段：

| 数据类型 | 表单控件 |
|----------|----------|
| `isRequired: true` | 必填输入框，带红色 * |
| `isSecret: true` | 密码输入框（显示/隐藏） |
| `default` 有值 | 输入框预填默认值 |
| `choices` 有值 | 下拉选择框 |
| `type: "positional"` | 普通输入框 |
| `type: "named"` | 输入框，label 带参数名（如 `--port`） |
| `isRepeated: true` | 可动态添加多个值的输入框 |

示例：文件系统 server 的自动生成表单

```
┌──────────────────────────────────┐
│  添加: Filesystem                │
│                                  │
│  访问路径 * (target_dir):         │
│  [/Users/me/docs             ]   │
│                                  │
│  日志级别 (LOG_LEVEL):            │
│  [info ▼]                        │
│                                  │
│       [取消]    [确认添加]        │
└──────────────────────────────────┘
```

---

## 市场分类

| 分类 ID | 名称 | 包含的 server |
|---------|------|--------------|
| filesystem | 文件系统 | filesystem, memory, S3, GCS |
| database | 数据库 | postgres, sqlite, mysql, mongodb, redis |
| search | 搜索引擎 | brave-search, duckduckgo, perplexity |
| devtools | 开发工具 | git, github, gitlab, playwright, docker |
| cloud | 云服务 | aws, supabase, terraform, vercel |
| comms | 通讯 | slack, discord, telegram |
| productivity | 生产力 | notion, linear, jira, google-drive |
| ai | AI/ML | openai, replicate, huggingface |

---

## 数据流

```
┌─────────────┐     ┌──────────────┐     ┌───────────────┐
│  官方 Registry │────▶│  本地缓存 JSON │────▶│  市场页面展示   │
│  API (在线)   │     │  (内置+刷新)  │     │  (分类+搜索)   │
└─────────────┘     └──────────────┘     └───────┬───────┘
                                                  │ 用户点击"添加"
                                                  ▼
                    ┌──────────────┐     ┌───────────────┐
                    │  配置表单     │◀────│  自动生成表单   │
                    │  (用户填写)   │     │  (读取参数定义)  │
                    └──────┬───────┘     └───────────────┘
                           │ 确认
                           ▼
                    ┌──────────────┐
                    │  数据库       │
                    │  (已安装配置)  │
                    └──────┬───────┘
                           │ 启动开关
                           ▼
                    ┌──────────────┐
                    │  Rust 后端    │
                    │  spawn 子进程 │
                    └──────────────┘
```

---

## Rust 后端 MCP 进程管理（Phase 2）

### 核心流程

```
启动服务：
  spawn("npx", ["-y", "@mcp/server-fs", "/path"])
  → 子进程的 stdin/stdout 建立 JSON-RPC 通道
  → 发送 initialize 请求
  → 发送 tools/list 请求，获取工具列表
  → 服务就绪

调用工具：
  收到 AI 的 tool_call(name, arguments)
  → 找到对应的 MCP 子进程
  → 通过 stdin 发送 tools/call 请求
  → 从 stdout 读取结果
  → 返回给 AI

停止服务：
  关闭 stdin → 子进程自动退出
  或 kill 子进程
```

### JSON-RPC 协议

MCP 使用 JSON-RPC 2.0 over stdio：

```json
// 请求（我们发给 MCP 进程）
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{...}}

// 响应（M 进程返回）
{"jsonrpc":"2.0","id":1,"result":{...}}

// 通知（无 id，不需要响应）
{"jsonrpc":"2.0","method":"notifications/initialized"}
```

关键方法：
- `initialize` → 初始化握手
- `tools/list` → 获取工具列表
- `tools/call` → 调用工具

---

## LLM 工具调用集成（Phase 3）

### 流程

```
1. 用户发消息
2. 构建请求时，将已启用的 MCP 工具列表注入到 API 请求的 tools 参数
3. AI 返回 tool_calls（如调用 read_file）
4. 找到对应的 MCP 子进程
5. 发送 tools/call 请求
6. 获取结果
7. 将结果作为 tool role 消息注入对话
8. 继续调用 AI，直到 AI 返回最终文本回复
```

### API 请求格式（带工具）

```json
{
  "model": "gpt-4",
  "messages": [...],
  "tools": [
    {
      "type": "function",
      "function": {
        "name": "read_file",
        "description": "Read file content",
        "parameters": {
          "type": "object",
          "properties": {
            "path": { "type": "string", "description": "File path" }
          },
          "required": ["path"]
        }
      }
    }
  ]
}
```

---

## 开发计划

| Phase | 内容 | 工作量 |
|-------|------|--------|
| **1a** | 数据模型 + 数据库表（mcp_servers, mcp_instances） | 0.5 天 |
| **1b** | 内置热门 server JSON（20-30 个，从 registry 一次性拉取整理） | 0.5 天 |
| **1c** | MCP 管理页面 UI（左侧列表 + 右侧卡片 + 搜索） | 1 天 |
| **1d** | 配置表单自动生成（根据 environmentVariables + packageArguments） | 0.5 天 |
| **1e** | 添加自定义服务表单 | 0.5 天 |
| **2a** | 在线 registry API 对接（刷新按钮） | 0.5 天 |
| **2b** | Rust 后端 MCP 进程管理（spawn/kill/stdin-out JSON-RPC） | 2 天 |
| **2c** | LLM tool_calls 集成（AI 返回工具调用 → 路由到 MCP → 注入结果） | 1 天 |

**Phase 1（页面+数据）：3 天**
**Phase 2（运行时+调用）：3.5 天**

---

## 常用 MCP Server 参考（内置列表候选）

| 名称 | 包名 | 分类 | 说明 |
|------|------|------|------|
| Filesystem | @modelcontextprotocol/server-filesystem | 文件 | 读写本地文件 |
| Memory | @modelcontextprotocol/server-memory | 文件 | 知识图谱存储 |
| Git | @modelcontextprotocol/server-git | 开发 | Git 操作 |
| GitHub | @modelcontextprotocol/server-github | 开发 | GitHub API |
| SQLite | @modelcontextprotocol/server-sqlite | 数据库 | SQLite 查询 |
| PostgreSQL | @modelcontextprotocol/server-postgres | 数据库 | PostgreSQL 查询 |
| Brave Search | @modelcontextprotocol/server-brave-search | 搜索 | Brave 搜索 |
| DuckDuckGo | @nickclyde/duckduckgo-mcp-server | 搜索 | DDG 搜索 |
| Puppeteer | @modelcontextprotocol/server-puppeteer | 开发 | 浏览器自动化 |
| Playwright | @microsoft/mcp-server-playwright | 开发 | 浏览器测试 |
| Slack | @modelcontextprotocol/server-slack | 通讯 | Slack API |
| Docker | @modelcontextprotocol/server-docker | 开发 | Docker 管理 |
| Fetch | @modelcontextprotocol/server-fetch | 工具 | HTTP 请求 |
| Context7 | @upstash/context7-mcp | 工具 | API 文档注入 |
| Sequential Thinking | @modelcontextprotocol/server-sequential-thinking | 工具 | 分步推理 |
