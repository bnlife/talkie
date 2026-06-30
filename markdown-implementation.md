# Markdown 渲染实现方案

## 一句话

用 `marked` 解析 + `DOMPurify` 安全过滤 + CSS 变量预留样式调整接口，流式输出时全量重解析。

---

## 技术选型

| 组件 | 选择 | 体积 | 说明 |
|------|------|------|------|
| 解析器 | `marked` | ~26KB | 最成熟的 markdown 解析库，API 简单 |
| 安全过滤 | `DOMPurify` | ~16KB | 防止 XSS，过滤 HTML 标签/属性 |
| 代码高亮 | **不需要** | 0 | 代码块用等宽字体纯文本显示 |
| 流式策略 | 全量重解析 | - | 每次 chunk 重新 `marked.parse(全部文本)` |

**不选的方案：**
- `markdown-streaming`（~4KB）：流式增量解析，但社区小，维护不确定
- `shiki`（~1MB）：代码高亮，太大且不需要
- 自研解析器：重复造轮子

---

## 开发步骤

### Step 1：安装依赖

```bash
npm install marked dompurify
npm install -D @types/dompurify
```

### Step 2：创建 markdown 工具模块

**文件：** `src/lib/markdown.ts`

```typescript
import { marked } from 'marked'
import DOMPurify from 'dompurify'

// 配置 marked：不使用 highlight.js
marked.setOptions({
  breaks: true,       // 支持换行符转 <br>
  gfm: true,          // GitHub Flavored Markdown
})

// 解析 markdown 为安全的 HTML
export function renderMarkdown(text: string): string {
  const rawHtml = marked.parse(text) as string
  return DOMPurify.sanitize(rawHtml, {
    ALLOWED_TAGS: [
      'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
      'p', 'br', 'hr',
      'strong', 'em', 'del', 'code', 'pre',
      'ul', 'ol', 'li',
      'blockquote',
      'table', 'thead', 'tbody', 'tr', 'th', 'td',
      'a',
    ],
    ALLOWED_ATTR: ['href', 'title', 'target'],
  })
}
```

### Step 3：定义 CSS 变量

**文件：** `src/styles/global.css`（追加）

```css
/* === Markdown 样式变量（Phase 2 样式调整页面可修改这些值） === */
:root {
  --md-h1-size: 1.5rem;
  --md-h2-size: 1.25rem;
  --md-h3-size: 1.125rem;
  --md-line-height: 1.7;
  --md-paragraph-spacing: 0.75rem;
  --md-blockquote-border: hsl(220 60% 60%);
  --md-blockquote-bg: hsl(220 60% 97%);
  --md-code-bg: hsl(0 0% 95%);
  --md-code-radius: 4px;
  --md-code-font-size: 0.875rem;
  --md-table-border: hsl(0 0% 85%);
  --md-link-color: hsl(220 60% 55%);
}
```

### Step 4：编写 markdown HTML 样式

**文件：** `src/styles/markdown.css`（新建）

```css
/* === Markdown 渲染样式 === */
/* 所有值引用 CSS 变量，Phase 2 样式调整页面直接修改变量即可 */

.markdown-body h1 {
  font-size: var(--md-h1-size);
  font-weight: 600;
  margin: 1.25rem 0 0.75rem;
  line-height: 1.3;
}

.markdown-body h2 {
  font-size: var(--md-h2-size);
  font-weight: 600;
  margin: 1rem 0 0.5rem;
  line-height: 1.3;
}

.markdown-body h3 {
  font-size: var(--md-h3-size);
  font-weight: 600;
  margin: 0.75rem 0 0.5rem;
  line-height: 1.4;
}

.markdown-body p {
  margin: 0 0 var(--md-paragraph-spacing);
  line-height: var(--md-line-height);
}

.markdown-body p:last-child {
  margin-bottom: 0;
}

.markdown-body strong {
  font-weight: 600;
}

.markdown-body em {
  font-style: italic;
}

.markdown-body del {
  text-decoration: line-through;
  opacity: 0.6;
}

.markdown-body a {
  color: var(--md-link-color);
  text-decoration: underline;
  text-underline-offset: 2px;
}

.markdown-body a:hover {
  opacity: 0.8;
}

/* 列表 */
.markdown-body ul,
.markdown-body ol {
  margin: 0.5rem 0;
  padding-left: 1.5rem;
}

.markdown-body li {
  margin: 0.25rem 0;
  line-height: var(--md-line-height);
}

.markdown-body ul li {
  list-style-type: disc;
}

.markdown-body ol li {
  list-style-type: decimal;
}

/* 引用块 */
.markdown-body blockquote {
  margin: 0.75rem 0;
  padding: 0.5rem 0.75rem;
  border-left: 3px solid var(--md-blockquote-border);
  background: var(--md-blockquote-bg);
  border-radius: 0 var(--md-code-radius) var(--md-code-radius) 0;
}

.markdown-body blockquote p {
  margin: 0;
}

/* 行内代码 */
.markdown-body code {
  background: var(--md-code-bg);
  padding: 0.15em 0.35em;
  border-radius: var(--md-code-radius);
  font-size: var(--md-code-font-size);
  font-family: ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, monospace;
}

/* 代码块 */
.markdown-body pre {
  margin: 0.75rem 0;
  padding: 0.75rem;
  background: var(--md-code-bg);
  border-radius: var(--md-code-radius);
  overflow-x: auto;
}

.markdown-body pre code {
  background: none;
  padding: 0;
  font-size: var(--md-code-font-size);
  line-height: 1.5;
}

/* 表格 */
.markdown-body table {
  width: 100%;
  margin: 0.75rem 0;
  border-collapse: collapse;
}

.markdown-body th,
.markdown-body td {
  border: 1px solid var(--md-table-border);
  padding: 0.4rem 0.6rem;
  text-align: left;
}

.markdown-body th {
  background: var(--md-code-bg);
  font-weight: 600;
}

.markdown-body tr:nth-child(even) {
  background: hsl(0 0% 98%);
}

/* 分割线 */
.markdown-body hr {
  margin: 1rem 0;
  border: none;
  border-top: 1px solid var(--md-table-border);
}
```

### Step 5：在 `src/styles/global.css` 引入

```css
@import './markdown.css';
```

### Step 6：改造 MessageItem.vue

**改动点：**

1. 用户消息：保持纯文本（不变）
2. 助手消息：用 `v-html` 渲染 markdown
3. 流式过程中也渲染 markdown（每次 chunk 重新解析）

```vue
<script setup lang="ts">
import { computed } from 'vue'
import { renderMarkdown } from '@/lib/markdown'

const props = defineProps<{
  message: Message
  // ... 其他 props
}>()

const isAssistant = computed(() => props.message.role === 'assistant')
const renderedContent = computed(() => {
  if (isAssistant.value) {
    return renderMarkdown(props.message.content)
  }
  return ''
})
</script>

<template>
  <!-- 用户消息：纯文本（不变） -->
  <p v-if="!isAssistant" class="text-sm leading-relaxed whitespace-pre-wrap break-words">
    {{ message.content }}
  </p>

  <!-- 助手消息：markdown 渲染 -->
  <div
    v-else
    class="markdown-body text-sm"
    v-html="renderedContent"
  />
</template>
```

### Step 7：处理流式渲染

**流式过程中也需要渲染 markdown。** 每次 `streamingContent` 变化时重新解析：

```vue
<script setup lang="ts">
// 在 MessageList.vue 中，流式消息也需要 markdown 渲染
const streamingHtml = computed(() => {
  if (chatStore.streamingContent) {
    return renderMarkdown(chatStore.streamingContent)
  }
  return ''
})
</script>

<template>
  <!-- 流式中的助手消息 -->
  <div
    v-if="chatStore.streamingId"
    class="markdown-body text-sm"
    v-html="streamingHtml"
  />
</template>
```

**性能说明：** `marked.parse()` 对几千字的文本解析耗时约 1-2ms，每次 chunk（通常几个字到几十字）触发一次完全可接受。如果消息特别长（>10000 字），可以加防抖（debounce 50ms），但一般不需要。

### Step 8：链接处理（可选）

默认 `marked` 输出的链接是 `<a href="...">`，在 Tauri WebView 中点击会跳转。如果需要调系统浏览器打开：

```typescript
// src/lib/markdown.ts 中添加
const renderer = new marked.Renderer()
renderer.link = function({ href, text }) {
  return `<a href="${href}" target="_blank" rel="noopener">${text}</a>`
}
marked.setOptions({ renderer })
```

配合 Tauri 的 `shell:allow-open` 权限，或者在 App.vue 中监听链接点击事件调用 `window.__TAURI__.shell.open(url)`。

**这一步可以后加，不影响核心功能。**

---

## 文件变更清单

| 文件 | 改动类型 | 说明 |
|------|----------|------|
| `package.json` | 修改 | +marked, +dompurify, +@types/dompurify |
| `src/lib/markdown.ts` | **新建** | renderMarkdown 函数 |
| `src/styles/global.css` | 修改 | +import markdown.css, +CSS 变量 |
| `src/styles/markdown.css` | **新建** | markdown HTML 样式（引用 CSS 变量） |
| `src/pages/chat/MessageItem.vue` | 修改 | 助手消息 v-html 渲染 |
| `src/pages/chat/MessageList.vue` | 修改 | 流式消息 markdown 渲染 |
| `src/__tests__/components/MessageItem.spec.ts` | 修改 | 更新助手消息测试 |
| `src/__tests__/lib/markdown.spec.ts` | **新建** | renderMarkdown 单元测试 |

---

## 测试计划

### markdown.spec.ts

| 测试 | 说明 |
|------|------|
| `renderMarkdown 基础格式` | 标题/加粗/斜体/列表正确转 HTML |
| `renderMarkdown 代码块` | `<pre><code>` 正确生成，无高亮 |
| `renderMarkdown 表格` | table/thead/tbody 正确生成 |
| `renderMarkdown 引用` | blockquote 正确生成 |
| `renderMarkdown XSS 过滤` | `<script>` 等恶意标签被移除 |
| `renderMarkdown 链接` | `<a href>` 正确生成 |
| `renderMarkdown 空文本` | 返回空字符串不报错 |
| `renderMarkdown 流式中间态` | 未闭合的代码块不会崩溃 |

### MessageItem.spec.ts 更新

| 测试 | 说明 |
|------|------|
| `用户消息显示纯文本` | 不变 |
| `助手消息渲染 markdown` | 输出包含 `<strong>` `<p>` 等 HTML |

---

## 实现顺序

1. `npm install marked dompurify`
2. 创建 `src/lib/markdown.ts`
3. 创建 `src/styles/markdown.css` + CSS 变量
4. 创建 `src/__tests__/lib/markdown.spec.ts`（先写测试）
5. 改造 `MessageItem.vue`（用户纯文本 / 助手 markdown）
6. 改造 `MessageList.vue`（流式 markdown）
7. 更新 `MessageItem.spec.ts`
8. `npm run test` + `npm run typecheck` 验证

---

## 未来扩展：样式调整页面（Phase 2）

Phase 1 的 CSS 变量设计已经为 Phase 2 预留了接口：

- 所有 markdown 样式都引用 `--md-*` 变量
- Phase 2 只需要在设置页面加控件（Slider / ColorPicker）修改这些变量值
- 变量值存入 Settings（新增 `markdownStyle` 字段）
- 应用启动时加载 Settings 并注入 CSS 变量

**Phase 2 的调整项：**

| 分组 | 变量 | 控件 |
|------|------|------|
| 标题 | `--md-h1-size`, `--md-h2-size`, `--md-h3-size` | Slider (0.8rem ~ 2rem) |
| 段落 | `--md-line-height`, `--md-paragraph-spacing` | Slider |
| 引用 | `--md-blockquote-border`, `--md-blockquote-bg` | ColorPicker |
| 代码 | `--md-code-bg`, `--md-code-radius`, `--md-code-font-size` | Slider + ColorPicker |
| 表格 | `--md-table-border` | ColorPicker |
| 链接 | `--md-link-color` | ColorPicker |
