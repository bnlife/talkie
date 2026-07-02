# UI 规范

## 颜色变量

| 变量 | 亮色值 | 用途 |
|------|--------|------|
| `--background` | `hsl(0 0% 100%)` | 内容区背景 |
| `--foreground` | `hsl(0 0% 9%)` | 正文文字 |
| `--muted` | `hsl(0 0% 92%)` | 外层区域（Toolstrip、Header） |
| `--accent` | `hsl(0 0% 96%)` | Hover 背景 |
| `--border` | `hsl(0 0% 90%)` | 边框 |

## Hover 模式

| 场景 | Class | 效果 |
|------|-------|------|
| `bg-muted` 上 | `hover:bg-background` | 浮起效果 |
| `bg-background` 上 | `hover:bg-foreground/5` | 微弱高亮 |
| 危险操作 | `hover:bg-destructive hover:text-destructive-foreground` | 红色警告 |

## 字号

| 尺寸 | Class | 用途 |
|------|-------|------|
| 12px | `text-xs` | 标签、徽章、时间戳、提示文字 |
| 14px | `text-sm` | 所有正文、标题、输入框、侧栏、按钮 |

## 侧栏一致性（Chat / Prompt / Settings）

- 宽度：`w-60`（240px）
- 文字：`text-sm text-muted-foreground`
- 操作按钮：Edit2（重命名）+ Trash2（删除），`size-5` 按钮，`size-3` 图标
- Hover 态：`hover:bg-foreground/5`
- 选中态：`bg-accent text-accent-foreground`
- 右键上下文菜单

## Tauri 权限

`src-tauri/capabilities/default.json`：
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
