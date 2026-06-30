# Talkie — Technical Architecture

## Overview

Talkie is a desktop chat application built with **Tauri** (Rust backend + Vue 3 frontend). It communicates with LLM APIs (OpenAI-compatible) and stores conversations locally via SQLite.

## Frontend Architecture

| Layer | Technology | Purpose |
|-------|-----------|---------|
| UI Components | **shadcn-vue** (Reka UI + Tailwind v4) | Pre-built accessible components |
| State | **Pinia** | Reactive stores for chat and settings |
| Bridge | **Tauri invoke** | Async calls to Rust backend |
| Icons | **lucide-vue-next** | All icons come from this library |
| Styling | **Tailwind v4** + CSS variables | Shadcn-vue color tokens via `--color-*` |

### Component Tree

```
App.vue
├── Sidebar.vue — 对话列表（搜索/排序/右键菜单）
│   └── Modal context menu (teleport to body)
│
├── Top bar (inline in App.vue)
│   ├── Sidebar toggle button
│   ├── Conversation title tag
│   └── Window controls (minimize/maximize/close)
│
├── ChatPage.vue
│   ├── MessageList.vue — 滚动容器 + 自动滚底
│   │   └── MessageItem.vue × N — 消息气泡
│   └── ChatInput.vue — textarea + 发送/停止按钮
│
└── Dialog (settings)
    └── SettingsPanel.vue — 设置表单
```

### Data Flow

```
User Input → ChatInput → emit('send') → ChatPage → chatStore.sendMessage()
                                                         ↓
                                              chatBridge.sendMessage() → Tauri invoke → Rust
                                                         ↓
                                              streaming response → Tauri event
                                                         ↓
                                              chatStore.appendStreamChunk() → reactive update → UI
```

## Backend Architecture (Rust)

| Module | Responsibility |
|--------|---------------|
| `main.rs` | App entry, window setup |
| `lib.rs` | Tauri command registration |
| `store.rs` | SQLite: conversations & messages CRUD |
| `config.rs` | JSON config file read/write |
| `commands/` | Tauri commands organized by domain |

## Key Design Decisions

1. **shadcn-vue over TDesign**: After a failed migration attempt, shadcn-vue was chosen for its simpler CSS variable-based theming, smaller bundle size (24KB vs 454KB), and better compatibility with Tailwind v4.

2. **CSS Variable Theming**: Colors live in `src/styles/global.css` as CSS variables (`--color-background`, `--color-primary`, etc.) and are exposed to Tailwind via `@theme inline`. This allows theme switching by overriding variable values.

3. **@layer base requirement**: All shadcn-vue components depend on `* { @apply border-border }` inside `@layer base`. Missing this causes borders to render black instead of the intended light gray.

4. **MCP-first component addition**: New shadcn-vue components must be added via the shadcn-vue MCP CLI before they can be used in Vue files. Sub-agents must not create component files themselves.

## UI Style Rules

See `.config/opencode/skills/uidesign/SKILL.md` for the complete set of UI principles enforced by AI agents.
