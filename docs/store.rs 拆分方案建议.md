# store.rs 拆分方案建议

`store.rs` 目前已增长至 1200+ 行，集成了数据库初始化、迁移逻辑、业务 CRUD 以及大量的内联单元测试。为了提升代码的可维护性和清晰度，建议将其拆分为一个模块包（Module Package）。

## 1. 目标目录结构

建议将 `src-tauri/src/store.rs` 替换为 `src-tauri/src/store/` 目录，结构如下：

```text
src-tauri/src/store/
├── mod.rs          # 模块入口，保留 init 和公共接口
├── migrations.rs   # 数据库建表、迁移及 Seed 逻辑
├── chat.rs         # 对话 (Conversations) 与 消息 (Messages) 的 CRUD
├── prompt.rs       # 提示词模板 (Prompts) 的 CRUD
└── mcp.rs          # MCP 市场与实例的 CRUD
```

---

## 2. 拆分逻辑说明

### 2.1 `migrations.rs` (逻辑剥离)
*   **内容**：将 `init` 函数中的 `CREATE TABLE` 语句、`ALTER TABLE` 迁移语句，以及整个 `seed_mcp_registry` 函数移至此处。
*   **理由**：初始化和迁移逻辑只在应用启动时运行一次，放在主逻辑文件中会干扰日常开发。剥离后，主文件会变得非常清爽。

### 2.2 `chat.rs` (核心业务)
*   **内容**：包含 `list_conversations`, `create_conversation`, `get_conversation`, `update_conversation`, `delete_conversation`, `pin_conversation`, `unpin_conversation` 以及所有 `messages` 相关的函数（`list_messages_by_conversation`, `create_message`, `delete_message` 等）。
*   **理由**：这是应用最频繁变动的核心业务逻辑。

### 2.3 `prompt.rs` (独立模块)
*   **内容**：包含 `list_prompts`, `create_prompt`, `update_prompt`, `delete_prompt`, `set_default_prompt`, `get_default_prompt` 等。
*   **理由**：提示词管理逻辑相对独立，适合作为单独的子模块。

### 2.4 `mcp.rs` (协议扩展)
*   **内容**：包含 `list_mcp_categories`, `list_mcp_servers`, `list_mcp_instances`, `create_mcp_instance` 等。
*   **理由**：MCP 属于插件/协议层逻辑，未来可能会有更复杂的同步或发现机制，独立出来便于扩展。

### 2.5 `mod.rs` (粘合剂)
*   **内容**：
    *   使用 `pub use` 重新导出子模块的公共函数，确保外部调用（如 `commands/*.rs`）不需要修改引用路径。
    *   保留 `init` 函数，但内部调用 `migrations::run_migrations(conn)`。

---

## 3. 专业化建议：测试代码的处理

目前 `store.rs` 中有约 500 行是内联测试（`mod tests`）。在拆分时，建议采取以下专业做法：

1.  **单元测试随文件走**：将对应的 `mod tests` 移动到对应的子模块文件末尾（例如 `create_prompt` 的测试移动到 `store/prompt.rs`）。这是 Rust 的标准做法。
2.  **提取共享测试工具**：将 `setup_db` 这种通用的测试辅助函数移动到 `store/mod.rs` 中，并标记为 `#[cfg(test)]`，供所有子模块测试共享。
3.  **考虑集成测试**：如果某些测试涉及跨模块协作（例如“删除对话应级联删除消息”），可以将其移动到 `src-tauri/tests/` 目录下的独立文件中。

---

## 4. 进阶建议：引入迁移框架

目前项目中通过 `ALTER TABLE ...` 手写迁移逻辑。随着版本增加，这种方式会变得难以维护。
*   **专业做法**：考虑引入 `rusqlite_migration` 库。它允许你按版本号定义迁移脚本，框架会自动处理“当前版本是多少”、“需要执行哪些脚本”的逻辑，避免在 `init` 函数中堆砌 `if` 或 `let _ = ...`。

## 5. 拆分后的引用示例

在 `src-tauri/src/lib.rs` 中，你依然可以像以前一样使用：
```rust
use crate::store;
// ...
let db = store::init(&db_path)?;
```
因为 `store/mod.rs` 已经通过 `pub use` 保持了 API 的兼容性。这样拆分对现有代码的侵入性最小。
