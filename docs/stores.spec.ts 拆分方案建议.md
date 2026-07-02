# stores.spec.ts 拆分方案建议

`src/__tests__/stores.spec.ts` 目前集成了三个不同 Store 的测试逻辑，且包含大量的 Mock 和 Helper 函数。为了符合“测试与代码一一对应”的原则，建议将其拆分为独立的测试文件。

## 1. 目标目录结构

建议将测试文件按 Store 名称拆分，并统一存放在 `src/__tests__/stores/` 目录下：

```text
src/__tests__/
├── stores/
│   ├── chatStore.spec.ts      # 对应 chatStore.ts
│   ├── settingsStore.spec.ts  # 对应 settingsStore.ts
│   └── promptStore.spec.ts    # 对应 promptStore.ts
└── helpers.ts                 # 提取公共测试工具
```

---

## 2. 拆分逻辑说明

### 2.1 `helpers.ts` (工具共享)
*   **内容**：将原文件中的 `createConv`, `createMsg`, `createPrompt`, `createProvider` 等工厂函数提取到此文件中。
*   **理由**：这些函数在多个 Store 测试中都会用到。提取出来可以避免代码重复，且方便后续统一扩展数据模型。

### 2.2 `chatStore.spec.ts` (核心测试)
*   **内容**：保留 `describe('chatStore', ...)` 块。
*   **重点测试项**：
    *   对话的加载、创建、删除、重命名、置顶。
    *   消息发送、流式响应处理、自动生成标题逻辑。
    *   跨 Store 联动（如依赖 `settingsStore` 的当前 Provider）。

### 2.3 `settingsStore.spec.ts` (配置测试)
*   **内容**：保留 `describe('settingsStore', ...)` 块。
*   **重点测试项**：
    *   配置的加载与保存。
    *   Provider 的 CRUD 操作。
    *   连接测试与模型列表获取。

### 2.4 `promptStore.spec.ts` (模板测试)
*   **内容**：保留 `describe('promptStore', ...)` 块。
*   **重点测试项**：
    *   提示词模板的增删改查。
    *   默认模板的切换逻辑。

---

## 3. 专业化建议：Mock 管理

目前每个测试文件头部都有大量的 `vi.mock(...)`。
*   **优化方案**：考虑在 `src/__tests__/setup.ts` 中统一配置全局 Mock，或者在 `helpers.ts` 中导出统一的 Mock 清理函数。
*   **优势**：减少每个测试文件的模板代码，让测试用例本身更加突出。

## 4. 拆分后的引用示例

拆分后，每个测试文件的头部会变得非常整洁：

```typescript
// src/__tests__/stores/chatStore.spec.ts
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useChatStore } from '../../stores/chatStore'
import { createConv, createMsg } from '../helpers'
import * as chatBridge from '../../bridge/chat'

// 仅保留本模块需要的 Mock
vi.mock('../../bridge/chat')
// ...
```

## 5. 为什么要拆分？

1.  **并行测试**：Vitest 可以并行运行不同的测试文件。拆分后，三个 Store 的测试可以同时运行，缩短整体测试时间。
2.  **定位更准**：当 `chatStore` 报错时，你只需要打开 `chatStore.spec.ts`，不需要在近千行的文件中上下翻找。
3.  **降低认知负担**：每个文件只关注一个业务领域，符合“高内聚低耦合”的原则。
