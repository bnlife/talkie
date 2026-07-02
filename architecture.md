# Talkie — 架构地图

## 概览

Talkie 是一个桌面端 AI 聊天应用，基于 **Tauri v2** 构建（Rust 后端 + Vue 3 前端）。支持多家 LLM 提供商（OpenAI 兼容协议），对话本地存储（SQLite），以及提示词模板管理。

## 前端架构

```
src/
│
├── main.ts                           ← 前端入口：创建 Vue 应用、挂载 Pinia、全局错误捕获
├── App.vue                           ← 根组件：Toolstrip 导航 + 视图切换（chat/settings/knowledge/prompt/mcp）
│
├── stores/
│   ├── chatStore.ts                  ← 对话列表、当前对话、消息流的响应式状态
│   ├── settingsStore.ts              ← 设置项（Provider 列表、暗黑模式、全局参数）
│   ├── promptStore.ts                ← 提示词模板 CRUD 状态
│   └── mcpStore.ts                   ← MCP 实例管理 + 事件监听
│
├── pages/
│   ├── chat/
│   │   ├── ChatView.vue              ← 聊天页面主容器：Header + Sidebar + MessageList + Input
│   │   ├── ChatHeader.vue            ← 聊天头部：标题 + 窗口控制按钮
│   │   ├── Sidebar.vue               ← 对话列表侧栏：搜索 / 新建 / 重命名 / 右键菜单
│   │   ├── MessageList.vue           ← 消息滚动容器 + 自动滚底
│   │   ├── MessageItem.vue           ← 单条消息容器：组装子组件
│   │   ├── ChatInput.vue             ← 输入区容器：组装子组件
│   │   ├── ThinkingBlock.vue         ← 思考过程折叠展示
│   │   └── useChatEvents.ts          ← 聊天事件 composable：监听流式 chunks + 错误
│   ├── settings/
│   │   ├── SettingsView.vue          ← 设置页面容器：Sidebar + Panel
│   │   └── SettingsPanel.vue         ← Provider 编辑面板（开关 / 密钥 / 地址 / 参数 / 模型）
│   ├── knowledge/
│   │   └── KnowledgeView.vue         ← 知识库页面（占位）
│   ├── prompt/
│   │   └── PromptView.vue            ← 提示词管理：列表 + 编辑区（失焦自动保存）
│   └── mcp/
│       ├── McpView.vue               ← MCP 管理页面容器
│       ├── McpSidebar.vue            ← MCP 侧栏（已安装实例列表）
│       ├── McpMarketGrid.vue         ← MCP 市场网格（可安装服务）
│       ├── McpInstanceDetail.vue     ← MCP 实例详情（工具列表 + 调试）
│       ├── McpInstallDialog.vue      ← MCP 安装对话框
│       └── McpCustomDialog.vue       ← MCP 自定义服务器添加对话框
│
├── components/
│   ├── app/
│   │   └── Toolstrip.vue             ← 左侧工具条：导航按钮（chat/knowledge/prompt/mcp）+ 暗黑模式 + 设置
│   ├── chat/                         ← 聊天相关子组件
│   │   ├── MessageHeader.vue         ← 消息头像
│   │   ├── MessageContent.vue        ← 消息内容渲染（用户文本/AI Markdown）
│   │   ├── MessageActions.vue        ← 操作按钮（复制、删除、重新生成）+ Token 计数
│   │   ├── AttachmentList.vue        ← 附件标签和下载功能
│   │   ├── SearchSources.vue         ← 搜索来源列表和展开/折叠
│   │   ├── SearchSelect.vue          ← 搜索引擎选择器
│   │   ├── ModelSelect.vue           ← 模型选择器
│   │   └── PromptSelect.vue          ← 提示词选择器
│   └── ui/                           ← shadcn-vue 组件（通过 MCP CLI 安装）
│       ├── avatar/
│       ├── badge/
│       ├── button/
│       ├── card/
│       ├── dialog/
│       ├── dropdown-menu/
│       ├── input/
│       ├── label/
│       ├── scroll-area/
│       ├── separator/
│       ├── slider/
│       └── textarea/
│
├── bridge/                           ← Tauri invoke 桥接层：前端调用后端的薄封装
│   ├── chat.ts                       ← send_message / stop_stream / get_messages 等
│   ├── conversation.ts               ← list / create / update / delete / pin
│   ├── prompt.ts                     ← list / create / update / delete / set_default
│   ├── settings.ts                   ← get / update / test_connection / fetch_models
│   ├── mcp.ts                        ← list / add / remove / start / stop / call_tool
│   └── log.ts                        ← log 命令封装
│
├── composables/                      ← 组合式函数
│   ├── useAttachment.ts              ← 附件管理：添加/删除/拖拽/构建内容
│   ├── useMessageRender.ts           ← 消息渲染：Markdown + 内联引用
│   ├── useSearchSelect.ts            ← 搜索引擎选择逻辑
│   ├── useModelSelect.ts             ← 模型选择逻辑
│   └── usePromptSelect.ts            ← 提示词选择逻辑
│
├── lib/                              ← 工具函数库
│   ├── utils.ts                      ← cn() 类名合并、通用工具
│   ├── markdown.ts                   ← Markdown → HTML 渲染 + DOMPurify 净化
│   ├── attachment.ts                 ← 附件处理
│   └── events.ts                     ← 事件总线 / Tauri 事件监听
│
├── types/
│   └── index.ts                      ← 全局 TypeScript 类型定义
│
├── styles/
│   ├── global.css                    ← 全局样式 + Tailwind v4 主题变量（@theme inline）
│   └── markdown.css                  ← Markdown 渲染专用样式
│
└── __tests__/                        ← vitest 单元测试
    └── setup.ts                      ← 测试环境配置
```

## 后端架构（Rust）

```
src-tauri/
│
├── src/
│   ├── main.rs                       ← 后端入口：调用 lib::run()
│   ├── lib.rs                        ← Tauri Builder 配置、AppState 初始化、命令注册
│   ├── models.rs                     ← 数据模型：Message / Conversation / ModelProvider / Settings / Prompt / MCP 相关结构体
│   ├── config.rs                     ← config.json 读写 + 旧格式迁移
│   ├── llm.rs                        ← HTTP 客户端 + SSE 流解析 + 请求取消 + 自定义 headers
│   ├── error.rs                      ← AppError 统一错误类型
│   │
│   ├── chat/
│   │   ├── mod.rs                    ← 聊天模块入口
│   │   ├── engine.rs                 ← 核心生成引擎：gather_context / resolve_llm_config / execute_stream / finalize_response
│   │   └── search.rs                 ← MCP 搜索集成：perform_search / parse_search_results
│   │
│   ├── commands/                     ← Tauri 命令层（薄控制器）
│   │   ├── mod.rs                    ← 模块入口 + re-export
│   │   ├── chat.rs                   ← send_message / stop_stream / get_messages / delete_message / regenerate_message
│   │   ├── conversation.rs           ← list / create / update / delete / pin / unpin
│   │   ├── prompt.rs                 ← list / create / update / delete / set_default
│   │   ├── settings.rs               ← get / update / test_connection / fetch_models / log / open_url
│   │   └── mcp.rs                    ← list_categories / list_servers / list_instances / add / remove / toggle / start / stop / call_tool / test
│   │
│   ├── mcp/                          ← MCP（Model Context Protocol）运行时
│   │   ├── mod.rs                    ← 模块入口
│   │   ├── jsonrpc.rs                ← JSON-RPC 2.0 协议解析
│   │   ├── pool.rs                   ← MCP 服务器进程池管理（启动 / 停止 / 生命周期）
│   │   └── runtime.rs                ← 单个 MCP 服务器运行时（stdin/stdout 通信）
│   │
│   └── store/                        ← SQLite 数据持久层
│       ├── mod.rs                    ← 数据库初始化 + 表创建入口
│       ├── migrations.rs             ← DDL 迁移 + MCP 种子数据
│       ├── chat.rs                   ← conversation + message CRUD
│       ├── prompt.rs                 ← prompt CRUD
│       └── mcp.rs                    ← mcp_category / mcp_server / mcp_instance CRUD
│
├── mcp-servers/                      ← 内置 MCP 服务器脚本
│   └── bocha-search/
│       └── index.js                  ← 博查搜索 MCP 服务
│
├── capabilities/
│   └── default.json                  ← Tauri 权限声明（窗口控制、拖拽等）
│
├── tauri.conf.json                   ← Tauri 应用配置（窗口尺寸、构建、打包）
├── Cargo.toml                        ← Rust 依赖清单
└── build.rs                          ← Tauri 构建脚本
```

## 数据流

```
用户输入 → ChatInput → 子组件(ModelSelect/SearchSelect/PromptSelect) → emit('send') → ChatView → chatStore.sendMessage()
                                                                                                        ↓
                                                                                              chatBridge.sendMessage() → Tauri invoke → Rust
                                                                                                        ↓
                                                                                              读取对话 → 获取 provider_id + model → 查找 Provider
                                                                                                        ↓
                                                                                              llm::stream_chat(base_url, api_key, model, headers, temperature, top_p, messages)
                                                                                                        ↓
                                                                                              流式响应 → Tauri event "chat:stream-chunk"
                                                                                                        ↓
                                                                                              chatStore.appendStreamChunk() → 响应式更新 → MessageItem → 子组件渲染
```

## 关键设计决策

→ [docs/key-design-decisions.md](docs/key-design-decisions.md)

## UI 规范

→ [docs/ui-spec.md](docs/ui-spec.md)
