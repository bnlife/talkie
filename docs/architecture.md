# Talkie — Technical Architecture

## Overview

Talkie is a desktop chat application built with **Tauri v2** (Rust backend + Vue 3 frontend). It communicates with multiple LLM providers (OpenAI-compatible), stores conversations locally via SQLite, and supports prompt template management.

## Frontend Architecture

| Layer | Technology | Purpose |
|-------|-----------|---------|
| UI Components | **shadcn-vue** (Reka UI + Tailwind v4) | Pre-built accessible components |
| State | **Pinia** | Reactive stores for chat, settings, prompts |
| Bridge | **Tauri invoke** | Async calls to Rust backend |
| Icons | **lucide-vue-next** | All icons come from this library |
| Styling | **Tailwind v4** + CSS variables | Shadcn-vue color tokens via `@theme inline` |
| Testing | **vitest** + @vue/test-utils | Unit tests |

### Component Tree

```
App.vue
├── Toolstrip.vue — 左侧工具条（导航 + 夜间模式切换）
│   └── Button × 5（聊天/知识库/提示词/夜间模式/设置）
│
├── ChatView.vue — 聊天页面
│   ├── Header（标题 + 窗口控制按钮）
│   ├── Sidebar.vue — 对话列表（搜索/新建/重命名/右键菜单）
│   ├── MessageList.vue — 滚动容器 + 自动滚底
│   │   └── MessageItem.vue × N — 消息（头像 + 名称/时间 + 内容）
│   └── ChatInput.vue — Textarea + 发送/停止按钮 + 模型切换器
│
├── KnowledgeView.vue — 知识库页面（占位）
│
├── PromptView.vue — 提示词管理页面
│   ├── Header（标题 + 侧栏折叠 + 窗口控制）
│   ├── Sidebar — 提示词列表（搜索/新建/右键菜单）
│   └── Main — 编辑区（名称 + 内容，@blur 自动保存）
│
└── SettingsView.vue — 设置页面（Provider 管理）
    ├── Header（标题 + 窗口控制）
    ├── Sidebar — Provider 列表（搜索/新建/右键菜单：重命名/设为默认）
    └── SettingsPanel.vue — ProviderEditor（5 区块：标题+开关/密钥/地址/参数/模型）
```

### Layout Structure

```
┌─────────────────────────────────────────────────────────┐
│ Toolstrip │            Header (bg-muted)                │
│ (bg-muted)│  [标题]              [🌙][—][□][×]          │
│           ├─────────────────────────────────────────────┤
│           │ ┌─────────────────────────────────────────┐ │
│           │ │         Content Area (rounded border)    │ │
│           │ │  ┌─────────┬───────────────────────────┐│ │
│           │ │  │ Sidebar │      Main Content         ││ │
│           │ │  │(bg-back │      (bg-background)      ││ │
│           │ │  │ ground) │                           ││ │
│           │ │  └─────────┴───────────────────────────┘│ │
│           │ └─────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

### Data Flow

```
User Input → ChatInput → emit('send') → ChatView → chatStore.sendMessage()
                                                         ↓
                                              chatBridge.sendMessage() → Tauri invoke → Rust
                                                         ↓
                                              Rust reads conversation → finds provider → builds request
                                                         ↓
                                              llm::stream_chat(base_url, api_key, model, headers, temperature, top_p, messages)
                                                         ↓
                                              streaming response → Tauri event "chat:stream-chunk"
                                                         ↓
                                              chatStore.appendStreamChunk() → reactive update → UI
```

## Backend Architecture (Rust)

| Module | Responsibility |
|--------|---------------|
| `main.rs` | App entry, window setup |
| `lib.rs` | Tauri command registration, AppState |
| `models.rs` | Message, Conversation, ModelProvider, Settings, Prompt structs |
| `store.rs` | SQLite: conversations, messages, prompts CRUD + migrations |
| `config.rs` | JSON config read/write + old format migration |
| `commands/chat.rs` | send_message, stop_stream, get_messages, delete_message, regenerate_message |
| `commands/conversation.rs` | list, create, update, delete, pin, unpin |
| `commands/settings.rs` | get_settings, update_settings, test_provider_connection, fetch_provider_models |
| `commands/prompt.rs` | list_prompts, create_prompt, update_prompt, delete_prompt, set_default_prompt |
| `llm.rs` | HTTP client + SSE parsing + cancellation + custom headers |
| `error.rs` | AppError unified error type |

### Multi-Provider Architecture

```
Settings.providers: ModelProvider[]
    ├── Provider A (OpenAI):  base_url, api_key, headers, models[], enabled
    ├── Provider B (DeepSeek): base_url, api_key, headers, models[], enabled
    └── Provider C (Ollama):  base_url, api_key, headers, models[], enabled

Conversation.provider_id → links to a specific Provider
Conversation.model → model name within that Provider

send_message flow:
    1. Read conversation → get provider_id + model
    2. Find provider in settings.providers
    3. Use provider.base_url + provider.api_key + provider.headers
    4. Use conversation.model (or provider.models[0] as fallback)
    5. Call llm::stream_chat with temperature + top_p from global settings
```

## Key Design Decisions

1. **shadcn-vue over TDesign**: shadcn-vue chosen for its simpler CSS variable-based theming, smaller bundle size, and better compatibility with Tailwind v4.

2. **CSS Variable Theming**: Colors live in `src/styles/global.css` as CSS variables (`--background`, `--foreground`, `--muted`, `--accent`, etc.) and are exposed to Tailwind via `@theme inline`. Neutral color scheme with `--muted: hsl(0 0% 92%)` for outer areas and `--accent: hsl(0 0% 96%)` for hover states.

3. **Layout Pattern**: All pages use unified layout with `bg-muted` outer area (Toolstrip + Header) and `bg-background` content area with `rounded-lg border`. No layout dividers, only rounded border separation. All three sidebars (Chat, Prompt, Settings) use `w-60` width.

4. **MCP-first component addition**: New shadcn-vue components must be added via the shadcn-vue MCP CLI before they can be used in Vue files.

5. **Per-conversation provider binding**: Each conversation stores its own `provider_id` + `model`, allowing different conversations to use different providers. The `active_provider_id` in Settings is only the default for new conversations.

6. **Auto-save on blur**: Prompt templates and Settings provider configs save automatically when the user leaves an input field, no explicit save button needed.

7. **System prompt injection**: `send_message` and `regenerate_message` prepend a system prompt from either the conversation's `system_prompt` field or the default prompt template (priority: conversation → default template → none).

8. **Error reporting via Tauri events**: Rust emits `chat:error` events instead of returning `Err(String)`, frontend catches via listener + vue-sonner toast.

## UI Style Rules

### Color Tokens

| Token | Light Mode | Usage |
|-------|-----------|-------|
| `--background` | `hsl(0 0% 100%)` | Content area background |
| `--foreground` | `hsl(0 0% 9%)` | Main text color |
| `--muted` | `hsl(0 0% 92%)` | Outer area (Toolstrip, Header) |
| `--accent` | `hsl(0 0% 96%)` | Hover background |
| `--border` | `hsl(0 0% 90%)` | Borders |

### Hover Patterns

| Context | Hover Class | Description |
|---------|------------|-------------|
| On `bg-muted` | `hover:bg-background` | Float up effect |
| On `bg-background` | `hover:bg-foreground/5` | Subtle highlight |
| Destructive | `hover:bg-destructive hover:text-destructive-foreground` | Red warning |

### Typography

| Size | Class | Usage |
|------|-------|-------|
| 12px | `text-xs` | Labels, badges, timestamps, hints |
| 14px | `text-sm` | All content, titles, inputs, sidebar, buttons |

### Sidebar Consistency (Chat / Prompt / Settings)

All three sidebars share:
- Width: `w-60` (240px)
- Text: `text-sm text-muted-foreground`
- Hover actions: Edit2 (rename) + Trash2 (delete), `size-5` buttons, `size-3` icons
- Item hover: `hover:bg-foreground/5`
- Active item: `bg-accent text-accent-foreground`
- Right-click context menu

### Tauri Permissions

Required in `src-tauri/capabilities/default.json`:
```json
{
  "permissions": [
    "core:default",
    "core:window:allow-start-dragging",
    "core:window:allow-minimize",
    "core:window:allow-toggle-maximize",
    "core:window:allow-maximize",
    "core:window:allow-unmaximize",
    "core:window:allow-close",
    "core:window:allow-is-maximized"
  ]
}
```
