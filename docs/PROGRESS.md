# Mnelys 项目进度

- 最后更新：2026-07-19
- 当前里程碑：M0 — 独立立项与技术验证
- 状态：进行中

## 当前快照

| 项目 | 当前值 |
|---|---|
| 产品名 | Mnelys |
| 中文名 | 忆涟 |
| 应用 ID | `io.mnelys.launcher` |
| 许可证 | MIT |
| 根命名空间 | `Mnelys` |
| 默认分支 | `main` |
| 本报告前的远端基线提交 | `24f80163d7dd9e91e45a89433207b755344359a1` |
| .NET SDK | `10.0.302` |
| 目标框架 | `net10.0` |
| Avalonia | `12.1.0` |
| CommunityToolkit.Mvvm | `8.4.2` |

远端仓库：<https://github.com/sdf123098/Mnelys>

## 已完成

### 第 1 周：新项目边界

- [x] 初始化全新 Git 仓库，不继承 Prism Launcher Git 历史。
- [x] 冻结产品名、中文名、应用 ID、命名空间和 MIT 许可证。
- [x] 建立独立实现与第三方代码来源边界。
- [x] 建立架构、安全、设计、支持矩阵和 ADR 文档。
- [x] 使用 `global.json` 锁定 .NET SDK 10.0.302。
- [x] 使用 Central Package Management 锁定 Avalonia 12.1.0。
- [x] 将新根提交强制同步到 GitHub `main`。
- [x] 建立简体中文、英文和日文三份独立 README。

### 第 2 周：三平台发布 PoC

- [x] 创建 `Mnelys.slnx` 与最小 Avalonia Desktop 项目。
- [x] 启用 Nullable、C# 14、Compiled Bindings、警告即错误和 AOT 兼容分析。
- [x] 移除反射型 ViewLocator，保留显式 View/ViewModel 绑定。
- [x] 创建不透明、无 Mica/Acrylic 的 1280×800 Shell，最小尺寸 900×600。
- [x] 将“忆涟”品牌图派生为 PNG 和多尺寸 ICO。
- [x] 完成 Debug/Release 构建与本机窗口启动冒烟。
- [x] 完成 Windows x64 自包含单 EXE 发布。
- [x] 完成 macOS arm64 自包含 payload 与 `.app` 目录结构。
- [x] 完成 Linux x64 自包含 payload 与 AppDir 目录结构。
- [x] 添加 Windows、macOS arm64 和 Linux x64 可复现打包脚本。
- [x] 执行 Native AOT 试编译并记录工具链阻塞项。
- [x] 将 macOS Intel/x64 从 1.0 范围移除。

## 验证证据

| 验证项 | 结果 |
|---|---|
| `dotnet build Mnelys.slnx -c Release` | 通过，0 警告、0 错误 |
| `dotnet format --verify-no-changes` | 通过 |
| README 本地链接检查 | 通过 |
| Windows 开发版启动 | 通过，窗口标题为“忆涟 · Mnelys” |
| Windows 单文件启动 | 通过 |
| Windows 单 EXE 大小 | 50,602,905 字节 |
| Windows 本机启动采样 | 约 1,755 ms 到主窗口句柄，不作为正式性能基准 |
| macOS arm64 cross-publish | 通过 |
| macOS `.app` 结构检查 | 通过，目标系统运行验证待完成 |
| Linux x64 cross-publish | 通过 |
| Linux AppDir 结构检查 | 通过，目标系统运行验证待完成 |

更完整的数据与哈希见 [M0 打包 PoC 报告](M0_PACKAGING_POC.md)。

## 已冻结决策

- Mnelys 是完全独立实现，不复制 Prism Launcher、MultiMC、HMCL 或 PCL2 的实现和资产。
- 中文产品名为“忆涟”。
- 当前代码使用 MIT License；旧 GPL 版本不会被追溯性重新许可。
- Windows 根窗口不请求 Mica、Acrylic、桌面背景采样或未公开 DWM 模糊。
- Windows 1.0 交付单个自包含 EXE。
- macOS 1.0 仅支持 Apple Silicon，不提供 Intel/x64 或 Universal DMG。
- Linux 1.0 目标为 x86_64 AppImage。
- Native AOT 是持续验证项，不是 1.0 的发布前提。

相关记录见 [ADR 索引](adr/README.md)。

## 尚未完成与阻塞项

### 目标平台验证

- [ ] 在无预装 .NET 的干净 Windows VM 验证单 EXE。
- [ ] 验证中文、空格、长路径和临时目录不可写等 Windows 场景。
- [ ] 在 Apple Silicon Mac 上生成 DMG、签名、公证、Staple 并通过 Gatekeeper。
- [ ] 在选定的旧 glibc Linux 构建环境生成 AppImage。
- [ ] 完成 Linux FUSE/no-FUSE、X11/XWayland、GNOME/KDE 和代表 GPU 冒烟。

### 工具链与合规

- [ ] 安装 Visual Studio“使用 C++ 的桌面开发”组件后重新执行 Windows Native AOT 试编译。
- [ ] 记录品牌图原作者和公开再分发条款。
- [ ] 在发布阶段建立签名证书、公证凭据和秘密管理流程。
- [ ] 建立 CI 三平台构建、格式、架构和打包检查。

## 下一步

本地开发可进入第 3～4 周“无 Mica Shell”：

1. 将语义颜色、间距、圆角和动画 Token 拆分为正式资源字典。
2. 建立浅色/深色主题和系统主题跟随。
3. 实现 Windows 不透明自绘标题栏，并保留系统拖动、缩放和窗口按钮语义。
4. 建立 Navigation、RouterHost、OverlayHost、DrawerHost 和 DialogHost。
5. 添加开始、实例、下载、联机和设置静态页面。
6. 完成响应式断点、键盘焦点、自动化名称和 200% DPI 基线测试。

目标系统打包验证可与 Shell 开发并行进行，但在 M0 退出前必须完成。

## 更新规则

每完成一个可验证切片，应更新：

1. 本文档的完成项、证据和下一步；
2. [支持矩阵](SUPPORT_MATRIX.md)中的平台状态；
3. 涉及架构或产品范围的 ADR；
4. 对应构建、测试或打包命令的实际结果。
