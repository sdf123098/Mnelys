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
> Mnelys 仍处于 M0 技术验证阶段。目前仓库包含可运行的 Avalonia Shell 和三平台打包 PoC，尚不具备完整的 Minecraft 安装与启动能力。

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

当前处于 M0“独立立项与技术验证”：

- [x] 独立仓库、许可证、ADR 与安全基线
- [x] .NET 10 / C# 14 / Avalonia 12 版本锁定
- [x] 不透明、无 Mica/Acrylic 的 Avalonia Desktop Shell
- [x] Windows x64 自包含单 EXE PoC
- [x] macOS arm64 自包含 payload 与 `.app` 结构 PoC
- [x] Linux x64 自包含 payload 与 AppDir 结构 PoC
- [ ] macOS 真机签名、公证与 DMG 验证
- [ ] Linux AppImage 构建机与发行版矩阵验证
- [ ] 首个 Vanilla 垂直切片

详细结果见 [M0 打包 PoC 报告](docs/M0_PACKAGING_POC.md)。

## 平台范围

| 平台 | 1.0 目标 | 交付物 |
|---|---:|---|
| Windows 10/11 x64 | Tier A | 单个自包含 `Mnelys.exe` |
| macOS 13+ Apple Silicon | Tier A | 签名并公证的 DMG |
| Linux x86_64 | Tier A | AppImage |
| macOS Intel/x64 | 不支持 | 不生成发布物 |
| Windows ARM64 / Linux ARM64 | 暂缓 | 1.x 后评估 |

## 技术栈

- C# 14 与 .NET 10 LTS
- Avalonia 12，AXAML 与 Compiled Bindings
- MVVM 与单向状态更新
- Central Package Management
- `System.Text.Json` Source Generation（后续领域切片）
- SQLite、`HttpClientFactory` 与结构化日志（后续核心骨架）

架构依赖始终向内：Presentation → Application → Domain，Infrastructure、Providers 与 Platform Adapters 实现内层定义的 Port。详见[架构基线](docs/ARCHITECTURE.md)。

## 本地运行

前置条件：

- .NET SDK `10.0.302`，或 `global.json` 接受的更高 `10.0.3xx` 补丁
- Git 2.40+

```powershell
git clone https://github.com/sdf123098/Mnelys.git
cd Mnelys
dotnet restore Mnelys.slnx
dotnet run --project src/Mnelys.Desktop/Mnelys.Desktop.csproj
```

验证 Release 构建与格式：

```powershell
dotnet build Mnelys.slnx -c Release
dotnet format Mnelys.slnx --verify-no-changes --no-restore
```

## 打包 PoC

Windows x64：

```powershell
.\packaging\windows\Publish-SingleFile.ps1
```

macOS arm64：

```bash
./packaging/macos/build-dmg.sh
```

Linux x64：

```bash
./packaging/linux/build-appimage.sh
```

目标平台工具、签名要求和限制见[打包说明](packaging/README.md)。

## 仓库结构

```text
Mnelys/
├─ assets/                 品牌源文件
├─ docs/                   计划、架构、安全、支持矩阵与 ADR
├─ packaging/              Windows、macOS、Linux 打包脚本
├─ src/Mnelys.Desktop/     当前 Avalonia 桌面 Shell
├─ Directory.Build.props   全仓库编译规则
├─ Directory.Packages.props 中央依赖版本
├─ global.json             .NET SDK 锁定
└─ Mnelys.slnx             解决方案入口
```

## 贡献与独立实现边界

提交代码前请阅读 [CONTRIBUTING.md](CONTRIBUTING.md)。不得复制或改编 Prism Launcher、MultiMC、HMCL、PCL2 或其他启动器的源码、资源、翻译、密钥、CI、打包脚本和内部模型。

发现安全问题时请遵循 [SECURITY.md](docs/SECURITY.md)，不要在公开 Issue 中提交凭据或未脱敏数据。

## 许可证与声明

代码以 [MIT License](LICENSE) 发布。

Mnelys 不是 Mojang Studios 或 Microsoft 的官方产品，也未获得其认可或关联。Minecraft 是其各自权利人的商标。品牌图原作者与公开再分发条款需在首次公开发行前完成记录。
