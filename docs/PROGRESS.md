# Mnelys 项目进度

- 最后更新：2026-10-26
- 当前里程碑：M1 — Design System 与 Shell（M0 验证项仍未全部关闭）
- 状态：进行中（核心日志、分层骨架与 CI 工作流已落地，等待首次远端运行证据）

## 当前快照

| 项目           | 当前值                                                                                      |
| -------------- | ------------------------------------------------------------------------------------------- |
| 产品名         | Mnelys                                                                                      |
| 中文名         | 忆涟                                                                                        |
| 应用 ID        | `io.mnelys.launcher`                                                                        |
| 许可证         | MIT                                                                                         |
| 默认分支       | `main`                                                                                      |
| 桌面壳与运行时 | Tauri 2.x                                                                                   |
| 核心语言       | Rust（stable，`rust-toolchain.toml` 固定）                                                  |
| 前端框架       | React 19 + TypeScript（[ADR 0006](adr/0006-frontend-framework-and-application-stack.md)）   |
| 前端构建与路由 | Vite + `createHashRouter`（hash 路由，三平台 WebView 一致）                                 |
| 前端状态与样式 | TanStack Query + Zustand；CSS Modules 与语义 Token                                          |
| Rust 分层      | Cargo workspace：domain / application / infrastructure / presentation；`src-tauri` 为组合根 |
| 持续集成       | GitHub Actions：前端检查、三平台 Rust 矩阵、三平台桌面打包（手动或标签触发）                |
| 本地化         | i18next，`zh-Hans` / `en` / `ja` 目录键集必须完全一致                                       |
| 界面图标       | Lucide（ISC），经单一注册表封装（[ICONS.md](ICONS.md)）                                     |
| 平台 WebView   | WebView2 / WKWebView / WebKitGTK                                                            |
| 窗口材质       | 不透明，无 Mica、Acrylic、vibrancy                                                          |

远端仓库：<https://github.com/sdf123098/Mnelys>

## 2026-09-30 技术路线切换

原 .NET 10 + Avalonia 12 基线（ADR 0002）已被 [ADR 0005](adr/0005-runtime-and-ui-baseline-tauri.md) 取代，改为 Tauri v2。

- 项目身份、名称、仓库、许可证与清洁室边界不变，ADR 0001、0003、0004 继续有效。
- 上一轮 Avalonia 实现已从 `main` 移除，但**完整保留在本仓库的 Git 历史中**（由 `24f8016` 引入，`5a6cb8d` 是切换前的最后一个提交）。
- 项目负责人另行在本地维护的两套实现副本（C++/Prism 派生版与 .NET/Avalonia 版）已被删除；其中未推送的 2 个提交与 94 个未提交文件无法恢复。这批代码与 UI 技术栈深度绑定，本就不参与迁移，但其中的"下载缓存 / SHA-1 内容寻址 / 哈希校验"属于与技术栈无关的领域设计，重做时需要重新设计一份。
- 上一轮 M0 的三平台打包证据（见 [M0 打包 PoC 报告](M0_PACKAGING_POC.md)）属于 Avalonia 基线的历史记录，**不构成 Tauri 基线的证据**。

## 2026-09-30 骨架验证

在本地对 `main` 上的前端工程与 Tauri 骨架做了第一轮完整校验：

- 前端 `pnpm format:check`、`pnpm lint`、`pnpm build`、`pnpm test` 全部通过（3 个测试文件、9 个用例；构建产物 408.33 kB，gzip 129.68 kB）。
- Rust 侧 `cargo fmt --check` 与 `cargo clippy --all-targets --all-features -- -D warnings` 通过，零告警。
- `cargo test` 在本机**无法执行**，属环境阻塞而非代码问题：本机既没有 MSVC 链接器（缺 Visual Studio Build Tools 的「使用 C++ 的桌面开发」工作负载），也没有 msvcrt 版 MinGW。改用 `x86_64-pc-windows-gnu` 可以编译，但测试二进制以 `0xC0000139`（`STATUS_ENTRYPOINT_NOT_FOUND`）退出——Rust 的 GNU 目标基于 msvcrt，而本机唯一的 MinGW 是 MSYS2 UCRT64。该结论已写入三语 README 与贡献指南。
- 本轮修掉两个真实缺陷：
  - `src-tauri/src/commands/mod.rs` 未重导出 `#[tauri::command]` 生成的隐藏宏（`__cmd__*` 与 `__tauri_command_name_*`）。命令放在子模块时，`generate_handler!` 按与命令相同的路径解析这些宏，缺失即报 `error[E0433]`。
  - `CoreError.details` 在 Rust 侧是 `Option<String>`，TypeScript 侧曾写成 `Readonly<Record<string, string>>`；`src-tauri/tests/contract.rs` 只校验 wire code，抓不到这类形状漂移。

## 2026-10-26 M1 外壳切片

本轮交付了可在浏览器预览的前端外壳。它仍然**不具备安装或启动 Minecraft 的能力**，所有未实现动作都禁用并标注里程碑，没有伪造实例、账户或下载记录。

- 设计系统：补齐字号、间距、层级、阴影、状态色与动效 token；落地 `system` / `light` / `dark` 三种外观，选择持久化，并在首屏渲染前应用以避免闪烁。
- 单窗口 Shell：两级宽度导航（220 px 常规 / 64 px 紧凑）、顶栏、状态栏、跳转主内容链接、可见焦点环、`prefers-reduced-motion` 处理。
- 任务中心：右侧模态抽屉，含焦点陷阱、Esc 关闭和焦点归还；空状态明确说明任务流将在 M2 接入。
- 基础控件：`Button` / `ButtonLink`、`IconButton`、`Badge`、`Card`、`PageHeader`、`EmptyState`、`SegmentedControl`（原生 radio 组）、`Switch`（原生 checkbox）。
- 页面：开始、实例、下载、联机、账户、开发、设置，均为诚实的空状态或规划说明。
- 模式：普通 / 开发者模式持久化；开发者分区按模式显示，设置页开关与开发页开关保持一致。
- 本地化：简中、英、日三套目录键集一致；设置页可切换并持久化，同时同步 `document.lang`。
- 图标：全部界面图标改用 Lucide（ISC），经 `src/components/ui/Icon.tsx` 单一注册表封装；应用图标继续使用品牌源图。见 [ICONS.md](ICONS.md)。
- 其它修正：补 `favicon` 消除浏览器 404；Tauri 窗口初始与最小尺寸与 [DESIGN_SPEC](DESIGN_SPEC.md) 对齐为 1280×800 / 900×600。

验证结果：

- `pnpm format:check`、`pnpm lint`、`pnpm typecheck`、`pnpm build`、`pnpm test` 全部通过；8 个测试文件、18 个用例。
- 前端构建产物：JS 466.65 kB（gzip 149.04 kB），CSS 26.76 kB（gzip 4.82 kB）。
- 在 Vite 浏览器预览中检查了 1440×900 与 1000×800、浅色与深色、主题循环、任务抽屉（打开 / Esc / 焦点归还）、跳过链接、开发者模式以及简中 / 英 / 日切换。
- 浏览器状态栏显示"核心不可用"属预期：预览环境没有 Rust 核心进程。本机仍缺 MSVC 生成工具与 WebView2，Tauri 桌面运行时未验证。

## 2026-10-26 M1 收尾：核心分层与日志面板

- Rust 拆为 Cargo workspace：`mnelys-domain`（失败词汇）、`mnelys-application`（`LogService` + `LogStore` 端口）、`mnelys-infrastructure`（`MemoryLogStore`）、`mnelys-presentation`（Tauri 命令与 handler）；`src-tauri` 只保留组合根职责。
- 命令契约升至 v2，新增 `app_logs`（最新在前，可按 `limit` 截取）与 `clear_app_logs`（清空并返回条数）。日志上限 500 条，仅在内存中保留。
- 任务中心新增“任务 / 日志”两分区；日志面板只在可见时每 3 秒轮询，提供加载、空态、失败重试与清空操作。
- 新增基于原生 `<dialog>` 的通用 `Dialog`。清空日志确认使用它，顶层、焦点约束、Esc 与焦点归还都由平台提供；当前只有一个模态叠加层，因此不另建 OverlayHost。
- 三语文案补齐任务中心分区、日志级别与清空确认；契约测试继续校验命令名、错误码与契约版本。
- 新增 `.github/workflows/ci.yml`（前端检查 + Windows / macOS / Linux Rust 矩阵）与 `.github/workflows/desktop-package.yml`（NSIS、macOS arm64 DMG、AppImage；手动触发或 `v*` 标签）。
- 本地验证：`cargo fmt --all --check`、前端 `format:check`、`lint`、`typecheck`、`test`（8 个测试文件、20 个用例）与 `build` 通过。Rust 编译、三平台测试与打包证据等待 CI 首次运行；本机 crates.io 索引不可达，无法在本地补跑 `cargo check`。

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

### React 前端与 Tauri 骨架（M0）

- [x] 确定前端框架与技术栈（[ADR 0006](adr/0006-frontend-framework-and-application-stack.md)）：React 19 + TypeScript、Vite、TanStack Query、Zustand、react-router、i18next。
- [x] 建立 `rust-toolchain.toml`、`src-tauri/` 骨架与 `src/` 前端工程；`pnpm-lock.yaml` 与 `src-tauri/Cargo.lock` 均已生成。
- [x] 定义命令边界与 `CoreError` 契约（`src/ipc/commands.ts`、`src/ipc/errors.ts` 与 Rust 侧一一对应）。
- [x] 建立前后端命令契约测试 `src-tauri/tests/contract.rs`。
- [x] 声明 `src-tauri/capabilities/default.json` capability 集（仅 `core:default`）。
- [x] 落地 i18n 三语目录与键集一致性测试、IPC 客户端与错误类型测试、基础组件测试。
- [x] 前端在本地通过 `format:check`、`lint`、`build`、`test`；Rust 侧通过 `fmt` 与 `clippy`（含全部测试目标）。
- [ ] 在装有 MSVC 生成工具的机器上执行 `cargo test`，跑通 `src-tauri/tests/contract.rs` 与 Rust 单元测试。
- [ ] 三平台 WebView 兼容基线与启动冒烟。
- [ ] 用 Tauri bundler 产出 NSIS、DMG、AppImage 三平台 PoC，并记录体积与冷启动数据。
- [ ] 在 Apple Silicon Mac 上完成签名、公证与 Gatekeeper 验证。
- [x] 建立 CI 三平台构建、格式与打包检查（工作流已落地；首次远端证据待运行）。

### Design System 与 Shell（M1，进行中）

- [x] 语义 token、浅色 / 深色主题与系统跟随。
- [x] 单窗口 Shell：导航、顶栏、状态栏、跳转链接与内容滚动复位。
- [x] 基础控件与原生语义（radio 组、checkbox switch、可见焦点）。
- [x] 任务中心抽屉：模态语义、焦点陷阱、Esc 关闭、焦点归还。
- [x] 七个一级页面骨架与诚实的未实现状态。
- [x] 普通 / 开发者模式持久化与导航显隐。
- [x] 三语界面与语言切换持久化。
- [x] 界面图标统一为 Lucide（ISC）并经注册表封装。
- [x] 前端格式、Lint、类型检查、构建与 20 个单元测试全部通过。
- [x] 日志面板（任务中心“日志”分区）与基于原生 `<dialog>` 的通用 `Dialog`。
- [x] `crates/` 分层骨架：domain / application / infrastructure / presentation；`src-tauri` 负责组合。
- [ ] Tauri 桌面运行时的三平台冒烟与 125% / 150% / 200% DPI 检查。
- [ ] 三平台桌面视觉回归（浏览器预览不能替代）。
- [ ] 基于 tauri-driver 的 E2E 冒烟。

### 上一轮 Avalonia 基线（已弃用，仅作存档）

- [x] .NET 10 / C# 14 / Avalonia 12 版本锁定与不透明无 Mica Shell。
- [x] Windows x64、macOS arm64、Linux x64 三种打包 PoC 及证据记录。

## 尚未完成与阻塞项

### Tauri 基线的 M0 收尾

- [ ] 由 GitHub Actions 三平台矩阵执行 `cargo test`；工作流已就绪，等待首次远端运行并记录结果。
- [ ] 由 `Desktop packages` 工作流产出 NSIS、macOS arm64 DMG 与 AppImage，记录体积；冷启动数据仍需真实桌面运行。
- [ ] 建立三平台 WebView 兼容基线（WebView2 / WKWebView / WebKitGTK）与启动冒烟。
- [ ] 在 Apple Silicon Mac 上完成 Developer ID 签名、公证、staple 与 Gatekeeper 验证；需要仓库外提供的 Apple 凭据。
- [x] 建立 CI 三平台前端检查、Rust 格式 / Clippy / 测试与桌面打包工作流（`.github/workflows/`）。
- [ ] 在首次远端运行通过后，把打包体积、启动结果与运行链接补入本文档。

### M1 收尾

- [x] 日志面板（任务中心“日志”分区）与基于原生 `<dialog>` 的通用 `Dialog`。
- [x] 引入 `crates/` 分层骨架（domain / application / infrastructure / presentation；providers / platform 等能力出现后再拆）。
- [ ] 在真实 Tauri 窗口中验证 100%～200% DPI、缩放与窗口装饰。
- [ ] 三平台桌面运行的视觉回归，浏览器预览不能替代。
- [ ] 基于 tauri-driver 的 E2E 冒烟。

### 产品

- [ ] 首个 Vanilla 垂直切片。

## 下一步

1. 推送后运行 `CI` 与 `Desktop packages`，修复首次运行的平台差异，并把三平台测试结果、产物体积与运行链接记入本文档。
2. 在具备图形桌面的 Windows / macOS / Linux 环境补做真实窗口的 100%～200% DPI、缩放、窗口装饰与视觉回归；CI 的无头打包不能替代这一步。
3. 进入 M2 Vanilla 垂直切片：元数据、下载与校验、Java 发现、实例创建、启动到主菜单。

目标平台打包由 CI 产出，但启动冒烟、DPI 与 Apple 签名/公证仍需对应的真实桌面环境和仓库外凭据。

## 更新规则

每完成一个可验证切片，应更新：

1. 本文档的完成项、证据和下一步；
2. [支持矩阵](SUPPORT_MATRIX.md)中的平台状态；
3. 涉及架构或产品范围的 ADR；
4. 对应构建、测试或打包命令的实际结果。
