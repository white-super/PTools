# 贡献指南

感谢参与 PTools 开发。提交代码前，请确保改动目标明确且已完成必要验证。

## 开发准备

项目需要 macOS、Node.js 20+、pnpm 和 Rust 1.85.0。Rust 版本由 `rust-toolchain.toml` 固定。

```bash
pnpm install
pnpm tauri dev
```

## 提交代码

1. 从最新代码创建功能或修复分支。
2. 完成功能并补充必要的测试或文档。
3. 运行提交前检查，确认没有引入无关改动。
4. 提交 Pull Request，清楚说明改动内容和验证方式。

## 代码约定

- 保持变更聚焦，不混入无关格式化、依赖升级或重构。
- 前端使用 Vue 3 Composition API 和 TypeScript，并保持组件数据流清晰。
- Rust 代码应返回可读错误，避免对外部输入或窗口资源使用 `unwrap()`。
- 不要提交剪贴板历史、应用数据库、系统配置、密钥、`.env` 文件或开发会话记录。
- 修改用户行为、数据或权限时，同步更新相关测试和文档。

## 提交前检查

在项目根目录运行：

```bash
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo check --manifest-path src-tauri/Cargo.toml
```

涉及剪贴板、快捷键、面板、自动粘贴或系统权限的改动，还需在 macOS 上手动验证对应交互。
