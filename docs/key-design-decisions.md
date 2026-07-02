# 关键设计决策

1. **shadcn-vue 而非 TDesign**：shadcn-vue 的 CSS 变量主题方案更简洁，打包体积更小，与 Tailwind v4 兼容性更好。

2. **CSS 变量主题**：颜色定义在 `src/styles/global.css`（`--background`、`--foreground`、`--muted`、`--accent` 等），通过 `@theme inline` 暴露给 Tailwind。`--muted` 用于外层区域，`--accent` 用于 hover 态。

3. **统一布局模式**：所有页面共用 `bg-muted` 外层区域（Toolstrip + Header）+ `bg-background` 内容区（`rounded-lg border` 分隔）。三个侧栏（Chat / Prompt / Settings）统一宽度 `w-60`。

4. **MCP-first 组件安装**：新增 shadcn-vue 组件必须先通过 shadcn-vue MCP CLI 安装，再在 Vue 文件中使用。

5. **对话级 Provider 绑定**：每个对话独立存储 `provider_id` + `model`，不同对话可使用不同 Provider。Settings 中的 `active_provider_id` 仅作为新建对话的默认值。

6. **失焦自动保存**：提示词模板和 Provider 配置在输入框失焦时自动保存，无需显式保存按钮。

7. **系统提示词注入**：`send_message` 和 `regenerate_message` 会从对话的 `system_prompt` 字段或默认提示词模板注入系统提示词（优先级：对话 > 默认模板 > 无）。

8. **Tauri 事件错误上报**：Rust 端通过 `chat:error` 事件上报错误，前端通过事件监听 + vue-sonner toast 展示。
