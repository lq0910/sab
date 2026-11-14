# 轻页 SAB

Small And Beautiful，本地优先的轻量桌面浏览器。

基于 Vue、Tauri、系统 WebView 与 SQLite。支持多标签、地址搜索与历史补全、收藏管理/导入、外观设置、原生网页下载、原生菜单快捷键、会话恢复、离线/等待超时提示及后台标签手动休眠。

开发运行：`pnpm install --frozen-lockfile`，然后 `pnpm tauri:dev`。

完整检查：`pnpm verify`。隔离原生下载验收：`cargo run --manifest-path src-tauri/Cargo.toml --example download_smoke --offline`。

隔离原生工作区验收：先执行 `pnpm build`，再执行 `cargo run --manifest-path src-tauri/Cargo.toml --example workspace_smoke --offline`。使用临时数据库和测试窗口，要求本机 1421 端口空闲。

产品现状、下载能力边界和后续优先级见 [产品能力评估与下载增强](docs/产品能力评估与下载增强-20261114.md)。

首个 0.1.0 Apple Silicon macOS 预览构建及发布限制见 [发布记录](docs/首个预览版发布记录-0.1.0.md)，本机候选产物位于 `output/releases/0.1.0-macos-arm64-preview/`。该包仅有 ad-hoc 本地签名，未经 Developer ID 签名和公证，不可作为公开下载发行版。可运行 `pnpm release:mac --unsigned` 重建预览包。

**图标修复与默认浏览器更新（构建 2026111402）**：请使用 `output/releases/0.1.0-macos-arm64-build2026111402/` 中的新包。macOS与Windows Tag自动构建发布、签名Secrets和默认浏览器说明见 [构建发布文档](docs/GitHub构建发布与默认浏览器-20261114.md)。两份工作流分别覆盖macOS arm64 / Intel / Universal与Windows x64 / x86 / ARM64；推送与工程版本一致的 `v*` Tag后自动发布各架构安装包。
