# Mnelys 项目进度

- 最后更新：2026-09-30
- 当前里程碑：M0 — 独立立项与技术验证（因技术路线切换而重启）
- 状态：进行中

## 当前快照

| 项目 | 当前值 |
|---|---|
| 产品名 | Mnelys |
| 中文名 | 忆涟 |
| 应用 ID | `io.mnelys.launcher` |
| 许可证 | MIT |
| 默认分支 | `main` |
| 桌面壳与运行时 | Tauri 2.x |
| 核心语言 | Rust（stable，`rust-toolchain.toml` 固定） |
| 前端框架 | 未决定（待 ADR 0006） |
| 平台 WebView | WebView2 / WKWebView / WebKitGTK |
| 窗口材质 | 不透明，无 Mica、Acrylic、vibrancy |

远端仓库：<https://github.com/sdf123098/Mnelys>

## 2026-09-30 技术路线切换

原 .NET 10 + Avalonia 12 基线（ADR 0002）已被 [ADR 0005](adr/0005-runtime-and-ui-baseline-tauri.md) 取代，改为 Tauri v2。

- 项目身份、名称、仓库、许可证与清洁室边界不变，ADR 0001、0003、0004 继续有效。
- 上一轮 Avalonia 实现已从 `main` 移除，但**完整保留在本仓库的 Git 历史中**（由 `24f8016` 引入，`5a6cb8d` 是切换前的最后一个提交）。
- 项目负责人另行在本地维护的两套实现副本（C++/Prism 派生版与 .NET/Avalonia 版）已被删除；其中未推送的 2 个提交与 94 个未提交文件无法恢复。这批代码与 UI 技术栈深度绑定，本就不参与迁移，但其中的"下载缓存 / SHA-1 内容寻址 / 哈希校验"属于与技术栈无关的领域设计，重做时需要重新设计一份。
- 上一轮 M0 的三平台打包证据（见 [M0 打包 PoC 报告](M0_PACKAGING_POC.md)）属于 Avalonia 基线的历史记录，**不构成 Tauri 基线的证据**。

## 已完成

### 项目边界（仍然有效）

- [x] 初始化全新 Git 仓库，不继承 Prism Launcher Git 历史。
- [x] 冻结产品名、中文名、应用 ID、命名空间和 MIT 许可证。
- [x] 建立独立实现与第三方代码来源边界。
- [x] 建立架构、安全、设计、支持矩阵和 ADR 文档。
- [x] 建立简体中文、英文和日文三份独立 README。
- [x] 确立 macOS 1.0 仅支持 Apple Silicon（ADR 0004）。

### 技术基线（已切换至 Tauri v2）

- [x] 确立 Tauri v2 运行时与 UI 基线（[ADR 0005](adr/0005-runtime-and-ui-baseline-tauri.md)），ADR 0002 标记为 Superseded。
- [x] 移除 .NET/Avalonia 专属文件与 `src/Mnelys.Desktop`。
- [x] 改写架构基线、支持矩阵、贡献指南与三语 README。

### 上一轮 Avalonia 基线（已弃用，仅作存档）

- [x] .NET 10 / C# 14 / Avalonia 12 版本锁定与不透明无 Mica Shell。
- [x] Windows x64、macOS arm64、Linux x64 三种打包 PoC 及证据记录。

## 尚未完成与阻塞项

### 前置决策

- [ ] **确定前端框架**（ADR 0006）：框架、语言、状态管理、路由与设计 Token 管线。在此之前不提交生产 UI 代码。

### Tauri 基线的 M0 工作

- [ ] 建立 `rust-toolchain.toml` 与前端工程，固定 Rust、Node.js 与包管理器版本。
- [ ] 搭建 Rust 分层骨架：domain / application / presentation / infrastructure / providers / platform。
- [ ] 定义 Tauri 命令边界，以及按窗口声明的 capability 集。
- [ ] 建立三平台 WebView 兼容基线（WebView2 / WKWebView / WebKitGTK）与启动冒烟。
- [ ] 用 Tauri bundler 产出 NSIS、DMG、AppImage 三平台 PoC，并记录体积与冷启动数据。
- [ ] 在 Apple Silicon Mac 上完成签名、公证与 Gatekeeper 验证。
- [ ] 建立 CI 三平台构建、格式与打包检查。

### 产品

- [ ] 首个 Vanilla 垂直切片。

## 下一步

1. 通过 ADR 0006 确定前端框架。
2. 落地工程骨架并固定工具链版本。
3. 复现三平台打包 PoC，取得 Tauri 基线自己的数据。

目标平台打包验证可与骨架开发并行进行，但在 M0 退出前必须完成。

## 更新规则

每完成一个可验证切片，应更新：

1. 本文档的完成项、证据和下一步；
2. [支持矩阵](SUPPORT_MATRIX.md)中的平台状态；
3. 涉及架构或产品范围的 ADR；
4. 对应构建、测试或打包命令的实际结果。
