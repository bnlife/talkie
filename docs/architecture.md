# Talkie — Technical Architecture

## Overview

Talkie is a desktop chat application built with **Tauri v2** (Rust backend + Vue 3 frontend). It communicates with LLM APIs (OpenAI-compatible) and stores conversations locally via SQLite.

## Frontend Architecture

| Layer | Technology | Purpose |
|-------|-----------|---------|
| UI Components | **shadcn-vue** (Reka UI + Tailwind v4) | Pre-built accessible components |
| State | **Pinia** | Reactive stores for chat and settings |
| Bridge | **Tauri invoke** | Async calls to Rust backend |
| Icons | **lucide-vue-next** | All icons come from this library |
| Styling | **Tailwind v4** + CSS variables | Shadcn-vue color tokens via `@theme inline` |
| Testing | **vitest** + @vue/test-utils | Unit tests |

### Component Tree

```
App.vue
├── Toolstrip.vue — 左侧工具条（导航 + 夜间模式切换）
│   └── Button × 4（聊天/知识库/夜间模式/设置）
│
├── ChatView.vue — 聊天页面
│   ├── Header（标题 + 窗口控制按钮）
│   ├── Sidebar.vue — 对话列表（搜索/新建/重命名/右键菜单）
│   ├── MessageList.vue — 滚动容器 + 自动滚底
│   │   └── MessageItem.vue × N — 消息气泡
│   └── ChatInput.vue — Textarea + 发送/停止按钮
│
├── KnowledgeView.vue — 知识库页面（占位）
│
└── SettingsView.vue — 设置页面
    └── SettingsPanel.vue — 设置表单
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

1. **shadcn-vue over TDesign**: shadcn-vue chosen for its simpler CSS variable-based theming, smaller bundle size, and better compatibility with Tailwind v4.

2. **CSS Variable Theming**: Colors live in `src/styles/global.css` as CSS variables (`--background`, `--foreground`, `--muted`, `--accent`, etc.) and are exposed to Tailwind via `@theme inline`. Neutral color scheme with `--muted: hsl(0 0% 92%)` for outer areas and `--accent: hsl(0 0% 96%)` for hover states.

3. **Layout Pattern**: All pages use unified layout with `bg-muted` outer area (Toolstrip + Header) and `bg-background` content area with `rounded-lg border`. No layout dividers, only rounded border separation.

4. **MCP-first component addition**: New shadcn-vue components must be added via the shadcn-vue MCP CLI before they can be used in Vue files. Sub-agents must not create component files themselves.

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
| 14px | `text-sm` | All content, titles, inputs, sidebar |

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
