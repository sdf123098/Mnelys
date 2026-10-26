# Mnelys 项目进度

- 最后更新：2026-10-26
- 当前阶段：M1–M6 核心能力已落地（账户、实例、下载安装、加载器、内容平台与启动管线）；桌面运行时、三平台打包证据与签名仍未关闭
- 状态：可构建、可测试；桌面交互验证与发布产物以 GitHub Actions 为权威路径

## 当前快照

| 项目           | 当前值                                                                                                                |
| -------------- | --------------------------------------------------------------------------------------------------------------------- |
| 产品名         | Mnelys                                                                                                                |
| 中文名         | 忆涟                                                                                                                  |
| 应用 ID        | `io.mnelys.launcher`                                                                                                  |
| 许可证         | MIT                                                                                                                   |
| 默认分支       | `main`                                                                                                                |
| 桌面壳与运行时 | Tauri 2.x                                                                                                             |
| 核心语言       | Rust（stable，`rust-toolchain.toml` 固定）                                                                            |
| 前端框架       | React 19 + TypeScript（[ADR 0006](adr/0006-frontend-framework-and-application-stack.md)）                             |
| 前端构建与路由 | Vite + `createHashRouter`（hash 路由，三平台 WebView 一致）                                                           |
| 前端状态与样式 | TanStack Query + Zustand；CSS Modules 与语义 Token                                                                    |
| Rust 分层      | Cargo workspace：domain / application / infrastructure / presentation；`src-tauri` 为组合根                           |
| 命令契约       | v3，39 条命令；`src-tauri/tests/contract.rs` 校验前后端命令集、错误码与版本                                           |
| 持续集成       | GitHub Actions：前端检查、三平台 Rust 矩阵、三平台桌面打包（手动或标签触发）                                          |
| 本地化         | 10 套目录：`en` / `zh-Hans` / `zh-Hant` / `ja` / `ko` / `fr` / `de` / `es` / `pt-BR` / `ru`；键集必须与 `en` 完全一致 |
| 主题           | 浅色 / 深色 / 跟随系统；默认强调色为品牌粉 `#C2185B`，支持预设与自定义十六进制色                                      |
| 标题栏         | 自绘标题栏；macOS 保留原生交通灯并以 overlay 方式留位                                                                 |
| 一级页面       | 开始 / 实例 / 下载 / 资源站 / 联机 / 设置；账户与任务中心为顶栏入口，开发分区受开发者模式控制                         |
| 界面图标       | Lucide（ISC），经单一注册表封装（[ICONS.md](ICONS.md)）                                                               |
| 平台 WebView   | WebView2 / WKWebView / WebKitGTK                                                                                      |
| 窗口材质       | 不透明，无 Mica、Acrylic、vibrancy                                                                                    |

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

## 2026-10-26 M1 收尾：品牌粉、自定义强调色与标题栏

- 强调色引擎：默认品牌粉 `#C2185B`；设置页提供 9 个预设色与自定义十六进制色。自定义色由一份纯函数展开为强调色、悬停色、柔和背景、焦点环与前景色五组 token；若对比度不足 WCAG AA，仅按最小幅度调整明度，色相与饱和度保持不变，并在界面说明"已调整"。选择在首屏渲染前应用并持久化。
- 自绘标题栏替换原生顶栏：Windows / Linux 绘制最小化、最大化 / 还原与关闭按钮，macOS 保留原生交通灯并以 overlay 方式为其留位；支持拖拽区域与双击最大化，窗口标题随界面语言同步。
- 产品名改为单一语言：简中"忆涟"、繁中"憶漣"、其余（含日语）"Mnelys"，不再出现"忆涟 · Mnelys"这类混排。
- 语言目录从 3 套扩展到 10 套：在原有 `zh-Hans` / `en` / `ja` 基础上新增 `zh-Hant`、`ko`、`fr`、`de`、`es`、`pt-BR`、`ru`。目录由 `import.meta.glob` 自动发现，新增语言只需放入一个 JSON 文件；`src/i18n/catalogs.test.ts` 校验每套目录与 `en` 的键集完全一致，并禁止产品名包含第二个名称。
- 语言选择器按核心语言优先、其余按代码排序；每项显示本地名称（endonym），并预留 RTL 基础语言集合。决策记录见 [ADR 0007](adr/0007-accent-color-and-localization-policy.md)。

## 2026-10-26 M2：Vanilla 垂直切片

- 设置与数据根：`MNELYS_HOME` 可覆盖数据根，否则使用平台数据目录；镜像偏好 `direct` / `bmclapi` / `auto`，`probe_mirrors` 返回可达性与延迟；内存上限持久化。
- 元数据与下载：从 Mojang 版本清单与版本 JSON 解析客户端、库、资源索引、日志配置与参数模板；BMCLAPI 仅改写传输地址，SHA-1 始终按上游值校验，镜像失败后最后一次直连源站；支持旧版虚拟资源解包与 natives 提取。
- Java：发现 `JAVA_HOME`、托管目录、各平台常见安装根与 `PATH`，逐个探测主版本与厂商；缺失时可从 Adoptium API 安装 Temurin，校验哈希并以任务形式报告进度。
- 实例：创建、读取、更新、删除；可覆盖 Java 路径；记录最后启动时间。导出与导入为 JSON 清单（上限 2 MiB），不是完整实例归档。
- 启动：按规则与特性渲染 JVM / 游戏参数，组装 classpath、`java.library.path`、资源索引与日志配置；离线账户 UUID 采用与 Java 一致的 MD5 v3 算法；进程输出按会话保留并可通过 `launch_logs` 读取，`stop_instance` 请求停止。
- 任务中心：安装、启动、Java、内容、导入导出等长操作统一为可取消任务，前端轮询不可变快照，不阻塞 UI 线程。

## 2026-10-26 M3：账户与安全存储

- 三类账户：离线、外置 Yggdrasil（认证 / 刷新 / 校验）、Microsoft 设备码 OAuth。
- 凭据只写入平台密钥库（Windows Credential Manager、macOS Keychain、Linux Secret Service），账户文件不含 token；密钥库不可用时明确报错，不回退明文。
- 刷新令牌只存在于核心内存与密钥库；访问令牌仅在构造启动计划时使用。`launch_instance` 只返回会话描述（PID、会话 ID），启动计划与访问令牌不跨越 IPC。
- Microsoft 客户端 ID 由 `MNELYS_MS_CLIENT_ID` 提供；未配置时登录按钮返回可读错误，不内置任何凭据。

## 2026-10-26 M4：Metadata 加载器

- Fabric、Quilt 与 Legacy Fabric 通过各自的 meta API 查询加载器版本，`list_loaders` 返回版本、游戏版本、加载器与稳定标记。
- 版本解析合并加载器 profile 与 Vanilla 文档，保留参数、库与主类；不运行安装器。

## 2026-10-26 M5：Installer 加载器

- Forge 与 NeoForge 走安装器路径：先安装 Vanilla 基础文件，再下载安装器 JAR、读取 Maven 元数据与 install profile，用选定 Java 运行 processors，最后合并生成的版本文档。
- 安装器与解压产物限制在实例与库目录内；解压使用 `enclosed_name` 与仅允许常规路径组件的校验，跳过越界条目。

## 2026-10-26 M6：内容平台与源

- 内容提供方：Modrinth v2 与 CurseForge v1。Modrinth 无需密钥；CurseForge 从 `MNELYS_CURSEFORGE_API_KEY` 读取密钥，未配置时明确提示而不是静默失败。
- 能力：搜索、列出项目版本、安装到实例、列出已安装内容、移除；支持 mod、资源包、光影、世界与数据包五类，按类型放入实例对应目录。
- 实例导出 / 导入清单包含版本、加载器与已安装内容引用，供另一台机器重建；不打包世界与用户文件。

## 2026-10-26 本地验证结果

- 前端：`pnpm format:check`、`pnpm lint`、`pnpm typecheck`、`pnpm test`、`pnpm build` 全部通过；8 个测试文件、38 个用例。
- 前端构建产物：JS 730.57 kB（gzip 222.24 kB），CSS 39.69 kB（gzip 6.51 kB）；仅有 >500 kB 分块提示，未做代码分割。
- Rust：`cargo fmt --all --check`、`cargo clippy --workspace --all-targets -- -D warnings` 通过，零告警。
- 本机可执行的 Rust 测试 107 个（domain 34、application 20、infrastructure 53）全部通过。`mnelys-presentation`（4 个）与 `src-tauri` 契约测试（3 个）的测试二进制在本机 Windows GNU 环境无法加载——它们依赖 WinRT API set DLL，属环境限制而非代码失败；完整测试以 CI 为准。
- GitHub Actions 首次远端验证通过（[CI run 40261698848](https://github.com/sdf123098/Mnelys/actions/runs/40261698848)）：前端检查与 Windows / macOS / Linux 三平台 Rust 作业全部成功；Linux 侧共执行 114 个 Rust 测试（domain 34、application 20、infrastructure 53、presentation 4、契约 3），零失败。
- 浏览器预览验证过强调色预设与自定义色、10 种语言切换、自绘标题栏布局与窄屏导航；真实 Tauri 窗口、DPI 与三平台打包仍待 CI 与桌面环境。
- `Desktop packages` 首次远端运行（[run 40262368596](https://github.com/sdf123098/Mnelys/actions/runs/40262368596)）三个作业全部成功，产物 zip 体积：`mnelys-windows-x64-unsigned` 6 782 945 B、`mnelys-macos-arm64-unsigned` 6 696 204 B、`mnelys-linux-x64-unsigned` 81 688 402 B。这些是未签名测试产物，保留 14 天，不是正式版本。

## 2026-10-26 资源站与界面接线收口

- 新增一级页面 **资源站**（`/content`，`src/routes/ContentRoute.tsx`）：平台切换、五类资源分段控件、游戏版本与加载器筛选、目标实例选择与安装，以及按类型分组的已安装内容。内容浏览不再只存在于实例详情里。
- 已安装内容改为按 `mod / 资源包 / 光影 / 世界 / 数据包` 分组（`src/components/InstalledContentGroups.tsx`），资源站与实例详情共用；搜索结果行带类型徽标。
- 侧栏品牌标记不再用产品名首字占位，改为应用图标本体 `public/app-icon.png`（与文档 favicon 同一 128×128 派生，`public/favicon.png` 已改名）。
- 顶栏账户入口接入 `list_accounts`：显示当前选中账户名，无账户时才回落到"未登录"。
- 下载量按界面语言用 `Intl.NumberFormat` 分组显示，语言目录里 `{{count}}` 改为 `{{total}}` 以免与 i18next 复数解析冲突。
- 10 套语言目录各新增 31 个键（`nav.content` 与 `content.*`），`src/i18n/catalogs.test.ts` 的键集奇偶校验通过；简体中文为"资源站"、日文为"コンテンツ"，切到日文时窗口标题仍是 `Mnelys`。
- 浏览器接线验证：注入 mock 核心（`@tauri-apps/api/mocks` 的 `mockIPC`）后走完开始、实例、实例详情、资源站、下载、账户、设置七个界面，确认 39 个命令的返回结构都能被界面消费；任务中心显示三种任务状态，资源站搜索 → 安装 → 分组列表链路可用。
- 该次改动自身的远端验证：[CI run 40265413181](https://github.com/sdf123098/Mnelys/actions/runs/40265413181)（提交 `3b169fe`）与 [CI run 40269430485](https://github.com/sdf123098/Mnelys/actions/runs/40269430485)（提交 `771a62e`）前端检查加三平台 Rust 作业全部成功。
- 产物已在 HEAD 上重打：[Desktop packages run 40269539520](https://github.com/sdf123098/Mnelys/actions/runs/40269539520)（提交 `771a62e`）三平台作业全绿，Windows 工件 zip 6 796 357 B，其中 `Mnelys_0.1.0_x64-setup.exe` 3 140 080 B（SHA-256 `84DB3980…CE0BE9`）、便携 `mnelys.exe` 6 827 008 B。两者只下载并确认是 PE，仍未安装、未启动，冷启动与真机运行时行为依旧空缺。

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
- [x] 由 GitHub Actions 三平台矩阵执行 `cargo test`，跑通 `src-tauri/tests/contract.rs` 与全部 Rust 单元测试。
- [ ] 三平台 WebView 兼容基线与启动冒烟。
- [ ] 在开发者本机安装并运行 NSIS / DMG / AppImage 产物，记录体积与冷启动数据。
- [ ] 在 Apple Silicon Mac 上完成签名、公证与 Gatekeeper 验证。
- [x] 建立 CI 三平台构建、格式与打包检查（工作流已落地并通过首次远端运行）。

### Design System 与 Shell（M1）

- [x] 语义 token、浅色 / 深色主题与系统跟随。
- [x] 单窗口 Shell：导航、顶栏、状态栏、跳转链接与内容滚动复位。
- [x] 基础控件与原生语义（radio 组、checkbox switch、可见焦点）。
- [x] 任务中心抽屉：模态语义、焦点陷阱、Esc 关闭、焦点归还。
- [x] 七个一级页面骨架与诚实的未实现状态。
- [x] 普通 / 开发者模式持久化与导航显隐。
- [x] 10 种语言界面与语言切换持久化。
- [x] 界面图标统一为 Lucide（ISC）并经注册表封装。
- [x] 前端格式、Lint、类型检查、构建与 38 个单元测试全部通过。
- [x] 日志面板（任务中心“日志”分区）与基于原生 `<dialog>` 的通用 `Dialog`。
- [x] `crates/` 分层骨架：domain / application / infrastructure / presentation；`src-tauri` 负责组合。
- [x] 默认品牌粉、强调色预设与自定义色；对比度不足时按最小幅度调整明度。
- [x] 自绘标题栏；macOS 保留原生交通灯并留位。
- [x] 产品名单一本地化，窗口标题随语言同步。
- [ ] Tauri 桌面运行时的三平台冒烟与 125% / 150% / 200% DPI 检查。
- [ ] 三平台桌面视觉回归（浏览器预览不能替代）。
- [ ] 基于 tauri-driver 的 E2E 冒烟。

### Minecraft 核心能力（M2–M6）

- [x] M2 Vanilla 垂直切片：元数据、哈希校验下载、Java 发现与安装、实例 CRUD、启动与日志、停止。
- [x] M2 镜像策略与探测；`MNELYS_HOME` 数据根覆盖。
- [x] M3 离线 / Yggdrasil / Microsoft 设备码账户；密钥库存储；令牌不跨越 IPC。
- [x] M4 Fabric / Quilt / Legacy Fabric 元数据加载器。
- [x] M5 Forge / NeoForge 安装器管线与 processors。
- [x] M6 Modrinth / CurseForge 搜索、版本、安装、移除与已安装列表。
- [x] 实例 JSON 清单导出 / 导入。
- [ ] 真实游戏启动到主菜单的三平台冒烟证据。
- [ ] CurseForge 与 Microsoft 登录的端到端验证（需要仓库外 API key 与 Azure 客户端）。
- [ ] 世界、光影与资源包在真实实例中的回归验证。

### 上一轮 Avalonia 基线（已弃用，仅作存档）

- [x] .NET 10 / C# 14 / Avalonia 12 版本锁定与不透明无 Mica Shell。
- [x] Windows x64、macOS arm64、Linux x64 三种打包 PoC 及证据记录。

## 尚未完成与阻塞项

### Tauri 基线的 M0 收尾

- [x] 由 GitHub Actions 三平台矩阵执行 `cargo test`（含 `src-tauri/tests/contract.rs`）；[CI run 40261698848](https://github.com/sdf123098/Mnelys/actions/runs/40261698848) 全绿。
- [x] 由 `Desktop packages` 工作流产出 NSIS、macOS arm64 DMG 与 AppImage 并记录体积（首跑 [run 40262368596](https://github.com/sdf123098/Mnelys/actions/runs/40262368596)，HEAD 重跑 [run 40269539520](https://github.com/sdf123098/Mnelys/actions/runs/40269539520)）；冷启动数据仍需真实桌面运行。
- [ ] 建立三平台 WebView 兼容基线（WebView2 / WKWebView / WebKitGTK）与启动冒烟。
- [ ] 在 Apple Silicon Mac 上完成 Developer ID 签名、公证、staple 与 Gatekeeper 验证；需要仓库外提供的 Apple 凭据。
- [x] 建立 CI 三平台前端检查、Rust 格式 / Clippy / 测试与桌面打包工作流（`.github/workflows/`），并通过首次远端运行。
- [x] 首次远端运行的产物体积与运行链接已记入本文档；安装与启动结果待真实桌面环境。

### M1 收尾

- [x] 日志面板（任务中心“日志”分区）与基于原生 `<dialog>` 的通用 `Dialog`。
- [x] 引入 `crates/` 分层骨架（domain / application / infrastructure / presentation；providers / platform 等能力出现后再拆）。
- [x] 默认品牌粉、自定义强调色、10 套语言目录与单一本地化产品名。
- [x] 自绘标题栏与 macOS 原生交通灯留位。
- [ ] 在真实 Tauri 窗口中验证 100%～200% DPI、缩放与窗口装饰。
- [ ] 三平台桌面运行的视觉回归，浏览器预览不能替代。
- [ ] 基于 tauri-driver 的 E2E 冒烟。

### M2–M6 的运行证据

- [x] M2–M6 代码、单元测试与前后端契约已落地。
- [ ] 在真实桌面环境完成一次 Vanilla 与至少一种加载器实例的安装、启动到主菜单。
- [ ] 验证离线、Yggdrasil 与 Microsoft 三类账户的实际登录与刷新。
- [ ] 验证 Modrinth 与 CurseForge 的实际搜索与安装（CurseForge 需要 `MNELYS_CURSEFORGE_API_KEY`）。
- [ ] 记录下载缓存命中、镜像回退与失败恢复的实测数据。

## 下一步

1. 在开发者机器上安装 CI 产出的三平台未签名包，记录冷启动与首屏数据；后续平台差异仍由 `CI` 与 `Desktop packages` 兜底。
2. 在具备图形桌面的 Windows / macOS / Linux 环境补做真实窗口的 100%～200% DPI、缩放、窗口装饰与视觉回归；CI 的无头打包不能替代这一步。
3. 用真实账户与网络完成一次 Vanilla 和一种加载器实例的端到端启动，补齐 M2–M6 的运行证据。
4. 根据实测结果决定首个 0.2 预发布范围；在此之前不发布中间版本。

目标平台打包由 CI 产出，但启动冒烟、DPI 与 Apple 签名/公证仍需对应的真实桌面环境和仓库外凭据。CI 产物是未签名的测试构建，不是正式版本。

## 更新规则

每完成一个可验证切片，应更新：

1. 本文档的完成项、证据和下一步；
2. [支持矩阵](SUPPORT_MATRIX.md)中的平台状态；
3. 涉及架构或产品范围的 ADR；
4. 对应构建、测试或打包命令的实际结果。
