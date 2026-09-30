<p align="center">
  <img src="assets/branding/mnelys-icon-source.jpg" width="220" alt="忆涟图标">
</p>

<h1 align="center">忆涟 · Mnelys</h1>

<p align="center">
  面向玩家与模组开发者的 AI 原生 Minecraft Java Edition 启动器与开发工作区
</p>

<p align="center">
  简体中文 · <a href="README.en.md">English</a> · <a href="README.ja.md">日本語</a>
</p>

> [!IMPORTANT]
> Mnelys 正在把技术基线从 .NET/Avalonia 切换到 **Tauri v2**（见 [ADR 0005](docs/adr/0005-runtime-and-ui-baseline-tauri.md)）。当前 `main` 上只有项目文档、品牌素材与决策记录，**还没有可运行的实现**，也尚不具备 Minecraft 安装与启动能力。

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

当前处于 M0“独立立项与技术验证”，并因技术路线切换而重新开始：

- [x] 独立仓库、许可证、ADR 与安全基线
- [x] 确立 Tauri v2 运行时与 UI 基线（[ADR 0005](docs/adr/0005-runtime-and-ui-baseline-tauri.md)）
- [x] 移除 .NET/Avalonia 存量实现（实现本身仍保留在本仓库 Git 历史中）
- [ ] 确定前端框架（ADR 0006，待定）
- [ ] Rust 核心骨架与 Tauri 命令边界
- [ ] Windows / macOS / Linux 三平台打包 PoC（Tauri bundler）
- [ ] 首个 Vanilla 垂直切片

上一轮 Avalonia 实现取得的打包证据见 [M0 打包 PoC 报告](docs/M0_PACKAGING_POC.md)。那是历史记录，**不构成 Tauri 基线的证据**。

## 平台范围

| 平台 | 1.0 目标 | 交付物 |
|---|---:|---|
| Windows 10/11 x64 | Tier A | NSIS 安装包与便携版可执行文件 |
| macOS 13+ Apple Silicon | Tier A | 签名并公证的 DMG |
| Linux x86_64 | Tier A | AppImage（附 `.deb`） |
| macOS Intel/x64 | 不支持 | 不生成发布物 |
| Windows ARM64 / Linux ARM64 | 暂缓 | 1.x 后评估 |

## 技术栈

- **Tauri 2.x** 作为桌面壳与运行时（[ADR 0005](docs/adr/0005-runtime-and-ui-baseline-tauri.md)）
- **Rust** 实现核心：账户、实例、Java 管理、下载、缓存、启动管线与 IPC
- 平台 WebView 渲染：Windows 用 WebView2，macOS 用 WKWebView，Linux 用 WebKitGTK
- 前端框架**尚未确定**，需先通过 ADR 0006；在那之前不提交生产 UI 代码
- Rust 工具链由 `rust-toolchain.toml` 固定；Node.js 与包管理器版本在仓库内固定
- 根窗口保持不透明：禁用 Mica、Acrylic、macOS vibrancy 与桌面采样
- 前端只能通过显式声明的 Tauri 命令接触原生能力；能力集按窗口声明并逐项评审

架构依赖始终向内：`presentation → application → domain`，Infrastructure、Providers 与 Platform Adapters 实现内层定义的 Port；Tauri 类型只允许出现在命令与适配器边界。详见[架构基线](docs/ARCHITECTURE.md)。

## 本地运行

前置条件（自 2026-09-30 起）：

- Rust stable 工具链，版本由 `rust-toolchain.toml` 固定
- Node.js 与仓库内固定的包管理器
- 平台构建依赖：Windows 需要 WebView2 与 MSVC 生成工具；macOS 需要 Xcode command line tools；Linux 需要 WebKitGTK 与 `libayatana-appindicator` 等 Tauri 系统依赖
- Git 2.40+

```powershell
git clone https://github.com/sdf123098/Mnelys.git
cd Mnelys
```

当前 `main` 上还没有可运行的 Tauri 工程，脚手架会在 ADR 0006 确定前端框架后落地。届时本地启动与构建为 `npm run tauri dev` 与 `npm run tauri build`，具体脚本名以前端工程落地后的 `package.json` 为准。

## 打包

三平台发布物由 Tauri bundler 统一产出，不再使用自定义脚本：

| 平台 | bundle 目标 | 产物 |
|---|---|---|
| Windows x64 | `nsis` | 安装包与便携版可执行文件 |
| macOS arm64 | `dmg` | 签名并公证的 DMG |
| Linux x86_64 | `appimage`、`deb` | AppImage（主）与 `.deb` |

```bash
npm run tauri build -- --bundles nsis
npm run tauri build -- --bundles dmg
npm run tauri build -- --bundles appimage,deb
```

目标平台工具、签名要求和限制见[打包说明](packaging/README.md)。

## 仓库结构

```text
Mnelys/
├─ assets/                 品牌源文件
├─ docs/                   计划、架构、安全、支持矩阵与 ADR
├─ packaging/              三平台打包说明（产物由 Tauri bundler 生成）
└─ （待落地）              `src-tauri/` 与前端工程，于 ADR 0006 之后创建
```

## 贡献与独立实现边界

提交代码前请阅读 [CONTRIBUTING.md](CONTRIBUTING.md)。不得复制或改编 Prism Launcher、MultiMC、HMCL、PCL2 或其他启动器的源码、资源、翻译、密钥、CI、打包脚本和内部模型。

发现安全问题时请遵循 [SECURITY.md](docs/SECURITY.md)，不要在公开 Issue 中提交凭据或未脱敏数据。

## 许可证与声明

代码以 [MIT License](LICENSE) 发布。

Mnelys 不是 Mojang Studios 或 Microsoft 的官方产品，也未获得其认可或关联。Minecraft 是其各自权利人的商标。品牌图原作者与公开再分发条款需在首次公开发行前完成记录。
