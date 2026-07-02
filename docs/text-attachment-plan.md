# 文本附件功能实现方案

## 一句话

在 ChatInput 输入框支持附加文本/代码文件（txt、md、json、js、py、rs 等），文件内容拼入 prompt 发送给 LLM，后端零改动。

---

## 为什么只做文本

| 类型 | 处理方式 | 是否需要多模态 API | 复杂度 |
|------|----------|---------------------|--------|
| 图片/音视频 | base64 编码，走 vision 接口 | 是，且各厂 API 格式不一 | 高 |
| 文本/代码 | readAsText，拼入 content | 否，所有模型兼容 | 低 |
| PDF/epub | 需要解析库提取文本 | 否 | 中 |

文本附件覆盖了最常见的使用场景（粘贴代码、附文档让 AI 分析），且改动量最小。

---

## 数据流

```
用户选择文件 → 前端 FileReader 读取内容 → 附件列表展示
                                              ↓
用户点发送 → 拼接：用户文字 + 附件内容（markdown 格式）→ sendMessage(content)
                                              ↓
                                    后端收到纯文本，正常调 LLM（无感知）
```

关键：**后端 sendMessage 接口不改**，附件内容在前端就拼进了 content 字符串。

---

## 拼接格式

```
用户输入的文字

---
### 📎 附件: main.rs (2.3 KB)
```rust
fn main() {
    println!("hello");
}
```

---
### 📎 附件: config.json (0.5 KB)
```json
{ "key": "value" }
```
```

- 每个附件用 `---` 分隔
- 标题行包含文件名和大小
- 内容用代码块包裹，标注语言（根据扩展名推断）
- 无语言标识的文件用纯代码块

---

## 改动清单

### 1. 新建 `src/lib/attachment.ts`

核心纯函数，包含：

| 函数 | 职责 |
|------|------|
| `isTextFile(file: File)` | 判断是否为文本文件（MIME + 扩展名双重检查） |
| `validateFileSize(file: File)` | 校验文件大小（上限 500KB） |
| `readFileAsText(file: File)` | Promise 封装 FileReader.readAsText |
| `formatAttachments(text, attachments)` | 拼接用户文字 + 附件内容 |
| `getLanguageHint(filename)` | 根据扩展名推断代码语言标识 |

文本文件判断逻辑（参考 LobeHub）：

```
已知文本 MIME: text/*, application/json, application/javascript,
application/typescript, application/xml, application/x-sh, application/sql
兜底扩展名: .ts, .tsx, .cs, .jsonl, .rs, .py, .go, .java, .c, .cpp, .h
```

### 2. 修改 `src/pages/chat/ChatInput.vue`

改动点：

- **附件按钮**：Textarea 下方工具栏左侧加一个 📎 按钮
- **隐藏的 file input**：`<input type="file" multiple accept=".txt,.md,.json,...">`，点击按钮触发
- **附件列表**：按钮右侧显示已选文件（文件名 + 大小 + 删除 × 按钮）
- **拖拽支持**：Textarea 上加 `@dragover.prevent` `@drop` 事件
- **发送逻辑改造**：`handleSend` 中先读取所有文件内容，调 `formatAttachments` 拼接，再 emit
- **状态管理**：`attachments` ref 存 `{ file: File, name: string, size: number }[]`
- **对话切换清空**：watch activeConversationId 变化时清空附件列表

accept 属性值（允许的文件扩展名）：
```
.txt,.md,.json,.js,.ts,.jsx,.tsx,.py,.rs,.go,.java,.c,.cpp,.h,.hpp,
.css,.html,.xml,.yaml,.yml,.toml,.sql,.sh,.bat,.ps1,.rb,.php,.swift,
.kt,.scala,.lua,.r,.m,.vue,.svelte,.astro,.config,.env,.gitignore,
.dockerfile,.makefile,.cmake
```

### 3. 修改 `src/pages/chat/MessageItem.vue`

改动点：

- 用户消息气泡顶部，文字之前，显示附件标签
- 标签样式：小药丸形状，显示 📎 + 文件名
- 仅当 `message.attachments` 存在且非空时显示

### 4. 修改 `src/types/index.ts`

```ts
export interface AttachmentMeta {
  name: string
  size: number
}
```

Message 接口加可选字段：
```ts
export interface Message {
  // ... 现有字段
  attachments?: AttachmentMeta[]
}
```

### 5. 修改 `src/stores/chatStore.ts`

`sendMessage` 方法中，给 tempMsg 附加 attachments 元数据：

```ts
const tempMsg: Message = {
  // ... 现有字段
  attachments: attachmentMetas,  // 仅存 { name, size }，不存内容
}
```

---

## 测试计划

### 单元测试：`src/__tests__/lib/attachment.spec.ts`

| 用例 | 说明 |
|------|------|
| 空附件列表 | `formatAttachments("hello", [])` → 返回原文字 |
| 单个附件 | 拼接结果包含用户文字 + 文件名 + 文件内容 |
| 多个附件 | 两个文件都出现，顺序正确 |
| 文件内容含特殊字符 | 反引号、管道符、# 号不破坏格式 |
| 空文件 | 仍然显示附件标题行 |
| isTextFile 识别 | .js .py .rs .json → true；.png .jpg → false |
| 文件大小校验 | 500KB 以内通过，超过拒绝 |
| getLanguageHint | .rs → rust，.py → python，.txt → 空 |

### 组件测试：补充 `src/__tests__/components/ChatInput.spec.ts`

| 用例 | 说明 |
|------|------|
| 附件按钮存在 | 渲染后能找到附件按钮 |
| 有附件 + 空文字可发送 | 点发送后 emit 的内容包含文件内容 |
| 有附件 + 有文字 | emit 内容是拼接后的完整文本 |
| 删除附件 | 点 × 后附件从列表消失 |
| disabled 时附件按钮不可用 | 按钮有 disabled 属性 |
| 切换对话清空附件 | activeConversationId 变化后附件列表为空 |

### 组件测试：补充 `src/__tests__/components/MessageItem.spec.ts`

| 用例 | 说明 |
|------|------|
| 有附件的用户消息 | 显示文件名标签 |
| 无附件的消息 | 不显示附件区域 |
| 多个附件 | 所有文件名都显示 |

---

## 不做的事（明确排除）

- **后端改动**：零改动，llm.rs / commands/chat.rs / models.rs / 数据库均不动
- **图片/音视频附件**：后续版本
- **PDF/epub 解析**：后续版本，需要引入解析库
- **Token 消耗估算**：后续版本，在附件旁显示 "~2.5K tokens"
- **附件草稿持久化**：后续版本，切换对话恢复附件列表
- **附件内容存数据库**：不存，已拼入 message.content

---

## 文件变更汇总

| 文件 | 操作 | 改动量 |
|------|------|--------|
| `src/lib/attachment.ts` | 新建 | ~80 行 |
| `src/types/index.ts` | 改 | +7 行 |
| `src/pages/chat/ChatInput.vue` | 改 | ~100 行 |
| `src/pages/chat/MessageItem.vue` | 改 | ~15 行 |
| `src/stores/chatStore.ts` | 改 | ~5 行 |
| `src/__tests__/lib/attachment.spec.ts` | 新建 | ~100 行 |
| `src/__tests__/components/ChatInput.spec.ts` | 改 | ~50 行 |
| `src/__tests__/components/MessageItem.spec.ts` | 改 | ~20 行 |

总计约 **370 行**代码变更，其中测试占 ~170 行。
