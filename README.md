<p align="center">
  <img src="assets/branding/mnelys-icon-source.jpg" width="220" alt="忆涟图标">
</p>

<h1 align="center">忆涟</h1>

<p align="center">
  面向玩家与模组开发者的 AI 原生 Minecraft Java Edition 启动器与开发工作区
</p>

<p align="center">
  简体中文 · <a href="README.en.md">English</a> · <a href="README.ja.md">日本語</a>
</p>

> [!IMPORTANT]
> Mnelys 的技术基线为 **Tauri v2 + React 19**（见 [ADR 0005](docs/adr/0005-runtime-and-ui-baseline-tauri.md) 与 [ADR 0006](docs/adr/0006-frontend-framework-and-application-stack.md)）。`main` 已实现 M1–M6 的核心能力：账户、实例、Java、元数据与加载器安装、内容平台与启动管线。真实桌面启动冒烟、三平台打包证据、签名与公证仍未完成；CI 产物是**未签名的测试构建，不是正式版本**。

## 项目简介

忆涟（Mnelys）是一款从零开发的跨平台 Minecraft Java Edition 启动器。它希望用同一套账户、实例、任务、缓存和进程模型，同时满足普通玩家与模组开发者的需求。

项目不继承 Prism Launcher 的代码、资源、Git 历史或内部数据模型。兼容能力只依据公开协议、官方 API、公开文件格式和独立测试夹具实现。

## 目标能力

- Microsoft、离线及外置 Yggdrasil 多账户管理
- Vanilla、Fabric、Quilt、Forge、NeoForge 等实例安装与启动
- Java 自动发现、下载与版本匹配
- Modrinth、CurseForge及本地内容安装
- 下载缓存、哈希校验、镜像回退和失败恢复
- 陶瓦联机组件集成
- Gradle 构建、部署、Debug Run、JDWP 与开发者 Daemon
- 本地规则优先、用户明确授权的 AI 崩溃诊断

这些是路线图目标，并不代表当前版本已经实现。

## 当前进度

M1–M6 的核心能力已实现，尚缺真实桌面环境中的运行证据与发布产物：

- [x] 独立仓库、许可证、ADR 与安全基线
- [x] Tauri v2 运行时与 UI 基线（[ADR 0005](docs/adr/0005-runtime-and-ui-baseline-tauri.md)）
- [x] 前端框架与技术栈（[ADR 0006](docs/adr/0006-frontend-framework-and-application-stack.md)）
- [x] 设计系统、浅色 / 深色 / 系统主题、默认品牌粉与自定义强调色
- [x] 自绘标题栏；macOS 保留原生交通灯；产品名单一本地化
- [x] 单窗口 Shell、响应式导航、任务中心、日志面板、设置与开发者模式
- [x] 10 套语言目录：简中、繁中、英、日、韩、法、德、西、葡（巴西）、俄
- [x] 界面图标统一为 Lucide（ISC），见 [ICONS.md](docs/ICONS.md)
- [x] Rust Cargo workspace 分层与 v3 命令契约（39 条命令）
- [x] Vanilla 安装与启动：元数据、哈希校验、镜像回退、Java 发现与安装、参数渲染、进程与日志
- [x] 离线、Yggdrasil 与 Microsoft 设备码账户；密钥库存储，令牌不跨越 IPC
- [x] Fabric / Quilt / Legacy Fabric 元数据加载器
- [x] Forge / NeoForge 安装器管线
- [x] Modrinth / CurseForge 搜索、安装、移除与实例清单导入导出
- [x] GitHub Actions：前端检查、三平台 Rust 矩阵与三平台桌面打包
- [ ] 真实桌面启动到主菜单、DPI 与三平台视觉回归
- [ ] 三平台打包体积、签名与公证证据

上一轮 Avalonia 实现取得的打包证据见 [M0 打包 PoC 报告](docs/M0_PACKAGING_POC.md)。那是历史记录，**不构成 Tauri 基线的证据**。

## 平台范围

| 平台                        | 1.0 目标 | 交付物                        |
| --------------------------- | -------: | ----------------------------- |
| Windows 10/11 x64           |   Tier A | NSIS 安装包与便携版可执行文件 |
| macOS 13+ Apple Silicon     |   Tier A | 签名并公证的 DMG              |
| Linux x86_64                |   Tier A | AppImage（附 `.deb`）         |
| macOS Intel/x64             |   不支持 | 不生成发布物                  |
| Windows ARM64 / Linux ARM64 |     暂缓 | 1.x 后评估                    |

## 技术栈

- **Tauri 2.x** 作为桌面壳与运行时（[ADR 0005](docs/adr/0005-runtime-and-ui-baseline-tauri.md)）
- **Rust** 实现核心：账户、实例、Java 管理、下载、缓存、启动管线与 IPC。核心是 Cargo workspace（domain / application / infrastructure / presentation），`src-tauri` 只做组合根
- 平台 WebView 渲染：Windows 用 WebView2，macOS 用 WKWebView，Linux 用 WebKitGTK
- 前端为 **React 19 + TypeScript**（[ADR 0006](docs/adr/0006-frontend-framework-and-application-stack.md)），Vite 构建，`react-router` 采用 hash 路由，服务端状态用 TanStack Query，瞬时状态用 Zustand，界面文本用 i18next，界面图标用 Lucide（ISC，见 [ICONS.md](docs/ICONS.md)）
- 本地化目录由 `import.meta.glob` 自动发现：新增语言只需增加一个 JSON 文件；`src/i18n/catalogs.test.ts` 要求每套目录键集与 `en` 完全一致。强调色与命名策略见 [ADR 0007](docs/adr/0007-accent-color-and-localization-policy.md)
- Rust 工具链由 `rust-toolchain.toml` 固定；Node.js 与 pnpm 版本在 `package.json` 中固定
- 根窗口保持不透明：禁用 Mica、Acrylic、macOS vibrancy 与桌面采样；全局样式表以 `backdrop-filter: none !important` 兜底
- 前端只能通过显式声明的 Tauri 命令接触原生能力，且 `invoke` 只允许出现在 `src/ipc/`；能力集按窗口声明并逐项评审

架构依赖始终向内：`presentation → application → domain`，Infrastructure、Providers 与 Platform Adapters 实现内层定义的 Port；Tauri 类型只允许出现在命令与适配器边界。详见[架构基线](docs/ARCHITECTURE.md)。

## 本地运行

前置条件（自 2026-09-30 起）：

- Rust stable 工具链，版本由 `rust-toolchain.toml` 固定
- Node.js 24+ 与 pnpm 11.7.0（`packageManager` 字段已固定，推荐用 corepack 启用）
- 平台构建依赖：Windows 需要 WebView2 运行时与 MSVC 生成工具（Visual Studio Build Tools 的「使用 C++ 的桌面开发」工作负载，提供 `link.exe`）；macOS 需要 Xcode command line tools；Linux 需要 WebKitGTK 与 `libayatana-appindicator` 等 Tauri 系统依赖
- Git 2.40+

> Windows 上不能用 MSYS2/MinGW 的 GNU 工具链代替 MSVC。`x86_64-pc-windows-gnu` 能通过编译、`cargo clippy` 与 domain / application / infrastructure 的测试，但 `mnelys-presentation` 与 `src-tauri` 的测试二进制依赖 WinRT API set，在本机 GNU 环境无法加载。完整测试以 CI 的 MSVC / Linux / macOS 运行为准。

```powershell
git clone https://github.com/sdf123098/Mnelys.git
cd Mnelys

pnpm install                                       # 前端依赖
cargo fetch --manifest-path src-tauri/Cargo.toml   # Rust 依赖

pnpm tauri:dev                                     # 启动桌面应用（会先拉起 Vite dev server）
```

`package.json` 中的脚本：

| 脚本                                  | 用途                                 |
| ------------------------------------- | ------------------------------------ |
| `pnpm dev`                            | 只启动 Vite 前端（浏览器中调试界面） |
| `pnpm build`                          | 类型检查并构建前端到 `dist/`         |
| `pnpm typecheck`                      | `tsc --noEmit`                       |
| `pnpm lint`                           | ESLint                               |
| `pnpm format:check`                   | Prettier 检查                        |
| `pnpm test`                           | Vitest + Testing Library             |
| `pnpm tauri:dev` / `pnpm tauri:build` | Tauri 开发与打包                     |

## 持续集成与打包

- [`CI`](.github/workflows/ci.yml)：前端格式、Lint、类型检查、测试与构建；Windows / macOS / Linux 三平台执行 `cargo fmt`、`cargo clippy -D warnings` 与 `cargo test`。
- [`Desktop packages`](.github/workflows/desktop-package.yml)：手动触发或 `v*` 标签运行，上传 NSIS、macOS arm64 DMG 与 AppImage。产物未签名，保留 14 天，仅用于构建验证；Apple 签名与公证需要仓库外凭据。
- **下载测试产物**：打开 GitHub 仓库的 Actions → `Desktop packages` → `Run workflow`；运行完成后在该次运行的 Artifacts 区域下载 `mnelys-windows-x64-unsigned`、`mnelys-macos-arm64-unsigned` 或 `mnelys-linux-x64-unsigned`。Windows 产物包含 NSIS 安装包与便携版 `mnelys.exe`。
- 常规验证以 CI 为准，本地桌面构建仅用于必须取得产物或交互调试的例外场景。

> `src-tauri/` 的编译期会校验前端产物存在。若要单独跑 `cargo check`、`cargo test` 或 `cargo clippy`，必须先执行一次 `pnpm build`，否则 `tauri::generate_context!` 会因找不到 `dist/` 而失败。

## 打包

三平台发布物由 Tauri bundler 统一产出，不再使用自定义脚本：

| 平台         | bundle 目标       | 产物                     |
| ------------ | ----------------- | ------------------------ |
| Windows x64  | `nsis`            | 安装包与便携版可执行文件 |
| macOS arm64  | `dmg`             | 签名并公证的 DMG         |
| Linux x86_64 | `appimage`、`deb` | AppImage（主）与 `.deb`  |

```bash
pnpm tauri:build -- --bundles nsis
pnpm tauri:build -- --bundles dmg
pnpm tauri:build -- --bundles appimage,deb
```

Windows 目标显式指定为 `x86_64-pc-windows-msvc`；macOS 为 `aarch64-apple-darwin`。

目标平台工具、签名要求和限制见[打包说明](packaging/README.md)。

## 仓库结构

```text
Mnelys/
├─ assets/                 品牌源文件
├─ crates/                 Rust 核心分层：domain / application / infrastructure / presentation
├─ docs/                   计划、架构、安全、支持矩阵与 ADR
├─ packaging/              三平台打包说明（产物由 Tauri bundler 生成）
├─ src/                    React 19 前端
│  ├─ app/                 应用装配：providers、router
│  ├─ components/          可复用组件
│  ├─ ipc/                 唯一的 Tauri 命令与错误边界
│  ├─ i18n/                10 套语言目录（自动发现）
│  ├─ routes/              页面级组件
│  ├─ state/               跨页面瞬时状态
│  ├─ styles/              Token 与全局样式
│  └─ test/                测试初始化
├─ src-tauri/              Tauri v2 壳、窗口配置与组合根
│  ├─ capabilities/        按窗口声明的能力集
│  ├─ icons/               应用图标
│  ├─ src/                 组合根：装配状态、挂载命令、启动运行时
│  └─ tests/               前后端命令契约测试
├─ Cargo.toml / Cargo.lock  工作区清单与依赖锁
├─ package.json / pnpm-lock.yaml
├─ rust-toolchain.toml
├─ tsconfig.json / vite.config.ts / eslint.config.js
└─ .prettierrc.json
```

Rust 采用 Cargo workspace 分层：`mnelys-domain`（失败词汇）、`mnelys-application`（用例与端口）、`mnelys-infrastructure`（适配器）、`mnelys-presentation`（Tauri 命令与 handler）；`src-tauri` 仅负责组合。Providers 与 platform 适配器等对应能力出现后再拆分为独立 crate。

## 贡献与独立实现边界

提交代码前请阅读 [CONTRIBUTING.md](CONTRIBUTING.md)。不得复制或改编 Prism Launcher、MultiMC、HMCL、PCL2 或其他启动器的源码、资源、翻译、密钥、CI、打包脚本和内部模型。

发现安全问题时请遵循 [SECURITY.md](docs/SECURITY.md)，不要在公开 Issue 中提交凭据或未脱敏数据。

## 许可证与声明

代码以 [MIT License](LICENSE) 发布。

Mnelys 不是 Mojang Studios 或 Microsoft 的官方产品，也未获得其认可或关联。Minecraft 是其各自权利人的商标。品牌图原作者与公开再分发条款需在首次公开发行前完成记录。
