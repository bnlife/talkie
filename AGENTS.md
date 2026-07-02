# Talkie 项目规范

这是一个基于 Tauri + Vue 3 + shadcn-vue 的桌面应用项目。

## 技术栈

- **前端**: Vue 3 + TypeScript + Vite
- **UI 组件**: shadcn-vue (基于 Reka UI)
- **样式**: Tailwind CSS v4
- **桌面端**: Tauri v2
- **状态管理**: Pinia
- **路由**: Vue Router

## 项目结构

```
src/
├── components/     # 组件
│   ├── ui/         # shadcn-vue 组件源码（不要在使用处覆盖样式）
│   ├── app/        # 应用级组件
│   ├── chat/       # 聊天相关组件
│   └── settings/   # 设置相关组件
├── pages/          # 页面
├── stores/         # Pinia 状态
├── styles/         # 全局样式
│   ├── global.css  # CSS 变量 + @theme + 语义类
│   └── markdown.css
├── composables/    # 组合式函数
├── lib/            # 工具函数
└── types/          # 类型定义
```

## 构建与测试命令

```bash
npm run dev          # 启动开发服务器
npm run build        # 构建（会先运行 prebuild lint）
npm run test         # 运行测试
npm run typecheck    # 类型检查
npm run prebuild     # 运行 lint 检查
```

## 前端开发规则（强制）

当处理以下任务时，**必须先加载对应的 skill**：

| 任务类型 | 触发关键词 | 加载的 Skill |
|---------|-----------|-------------|
| 开发页面、使用组件 | UI、组件、样式、页面、Button、Input | `skill({ name: "shadcn-usage" })` |
| 新增 variant/size | variant、size、cva、扩展组件 | `skill({ name: "shadcn-customize" })` |
| 修改颜色/全局CSS | 颜色、主题、CSS变量、语义类 | `skill({ name: "shadcn-theme" })` |
| 代码检查/CI | lint、test、prebuild、检查 | `skill({ name: "shadcn-lint" })` |

### 核心禁令

1. **禁止在使用处覆盖组件样式** — 改组件源码一次，到处生效
2. **禁止使用原生 `<button>` 和 `<input>`** — 必须用 `<Button>` 和 `<Input>` 组件（`<input type="file">` 除外）
3. **禁止在 class 中覆盖 variant** — 颜色、hover、shadow 等必须通过 variant prop 控制
4. **禁止手写 UI 组件** — 必须使用 shadcn 提供的组件，不能用基础组件手动拼装（如用 DropdownMenu+Button 实现选择器，应用 Select）

### 正确流程

```
需要新样式？
  │
  ├─ 颜色/hover/shadow 变化 → shadcn-customize skill
  │   → 改组件 cva 定义，页面用 variant="xxx"
  │
  ├─ 高度/padding/border 变化 → shadcn-customize skill
  │   → 改组件 cva 定义，页面用 size="xxx"
  │
  ├─ 可复用布局模式 → shadcn-theme skill
  │   → 在 global.css 定义语义类，页面用 class="xxx"
  │
  └─ 纯布局（间距、flex、宽度）
      → 直接用布局工具类（ml-2, gap-2, w-full）
```
