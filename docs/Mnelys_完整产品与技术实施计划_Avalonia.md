# Mnelys Minecraft Launcher 完整产品与技术实施计划（Avalonia 重制版）

> 文档类型：产品计划书 / 技术架构方案 / 交付实施基线
> 项目定位：面向玩家与模组开发者的 AI 原生 Minecraft Java Edition 启动器
> 中文名：忆涟
> 技术基线：C# 14、.NET 10 LTS、Avalonia 12.x
> 项目形态：全新独立项目，不继承 Prism Launcher 代码与 Git 历史
> 正式平台：Windows 10/11 x64、macOS 13+ Apple Silicon、主流 Linux x86_64
> 交付形态：Windows 单 EXE、macOS DMG、Linux AppImage
> 窗口材质：不使用 Windows Mica、Acrylic 或其他系统背景材质
> 文档状态：新项目立项基线
> 更新日期：2026-07-19

---

## 目录

1. 项目摘要
2. 已冻结的路线决策
3. 可行性结论
4. 产品定位与用户
5. 产品原则
6. 版本范围与支持矩阵
7. UI/UX 与无 Mica 规范
8. 单窗口与响应式架构
9. Avalonia 技术基线
10. 软件架构
11. 仓库与解决方案结构
12. 账户与凭据
13. Minecraft 版本、实例与 Java
14. 加载器架构
15. 下载、缓存与镜像
16. 内容平台与资源安装
17. 陶瓦联机
18. 开发者工作区与 Daemon
19. AI 与崩溃诊断
20. 任务、通知与可恢复操作
21. 数据模型与目录
22. 安全设计
23. 更新系统
24. Windows 单 EXE 交付
25. macOS DMG 交付
26. Linux AppImage 交付
27. CI/CD 与发布渠道
28. 测试计划
29. 可观测性、性能与无障碍
30. 许可证与合规
31. 里程碑与排期
32. 人员分工
33. 风险与控制
34. 验收标准
35. 立项后前八周任务
36. 待确认事项
37. 结论

---

# 1. 项目摘要

Mnelys 不再是 Prism Launcher 的换皮、分支或渐进式重构项目。新项目在独立目录、独立仓库、独立命名空间和独立发布链中从零建立 UI、应用层、领域模型与启动核心。

产品同时服务两类用户：

- 普通玩家：登录、安装、管理、启动、下载内容和联机；
- 模组开发者：构建、部署、调试、日志、崩溃分析和 AI 辅助。

两种使用方式共享同一套账户、实例、任务、缓存和进程模型，不拆成两个应用。

```text
普通玩家模式
  └─ 开始 / 实例 / 下载 / 联机 / 设置

开发者模式
  └─ 在相同实例工作区增加项目、构建、调试、日志和 AI 工具
```

新技术路线为：

```text
Avalonia UI
  ↓ ViewModel / Application Facade
.NET Application Layer
  ↓ Use Case / Port
Domain Layer
  ↓ Interface
Infrastructure / Provider / Platform Adapter
```

正式发布物必须满足：

| 平台 | 用户取得的主发布物 | 安装/启动体验 |
|---|---|---|
| Windows | `Mnelys.exe` | 无安装、无需预装 .NET，双击直接启动 |
| macOS | `Mnelys-<version>-<arch>.dmg` | 打开 DMG，拖入 Applications，签名并公证 |
| Linux | `Mnelys-<version>-x86_64.AppImage` | 添加执行权限后直接运行 |

“单文件”只约束应用分发物，不把账户、实例、缓存、Java、模组和日志写入可执行文件。用户数据始终存放在平台规范的数据目录，另提供显式便携模式。

---

# 2. 已冻结的路线决策

## 2.1 与 Prism 彻底切割

以下事项视为已经决定，不再作为普通技术选项反复讨论：

1. 在 `D:\Project\Mnelys` 建立新项目；
2. 创建新的 Git 仓库，不复制旧仓库 `.git`；
3. 不设置 Prism Launcher 为 upstream；
4. 不复制 Prism 源文件、资源、翻译、API Key、CI 或打包脚本；
5. 不沿用 Prism 的类名、命名空间和内部数据模型；
6. 只依据公开协议、官方 API、公开格式和独立测试夹具实现功能；
7. HMCL、PCL2、Prism、MultiMC 仅用于行为研究和兼容性测试；
8. 旧项目只读归档，必要时导出产品截图和不受限制的设计决策记录。

如未来决定复制或改编 GPL 代码，必须单独进行许可证评估，并重新决定整个发行物的开源义务；不得在日常开发中无意混入。

## 2.2 Avalonia 是唯一桌面 UI 主框架

- UI：Avalonia 12.x；
- 语言与运行时：C# 14、.NET 10 LTS；
- 标记语言：AXAML；
- 模式：MVVM + 单向状态更新；
- 不使用 Qt、Qt Widgets、Qt Quick 或 QML；
- 不以 WebView 作为主界面；
- 不以 WPF 作为跨平台抽象层。

依赖版本统一使用 Central Package Management 锁定。只跟随 Avalonia 12.x 的稳定补丁版本，重大升级必须通过 ADR 和完整回归矩阵。

## 2.3 拒绝 Windows Mica

Windows 版本不得启用：

- `Mica`；
- `AcrylicBlur`；
- DWM 系统背景材质；
- `SetWindowCompositionAttribute` 等未公开背景模糊方案；
- 依赖桌面内容采样的透明根窗口。

根窗口必须使用不透明背景，视觉层次通过纯色、渐变、描边、阴影、间距和局部动画表达。

## 2.4 三平台正式交付

Windows 优先完成 Alpha，但 macOS 与 Linux 不是“以后再说”的架构占位。1.0 发布门槛必须同时包含：

- Windows x64 单 EXE；
- macOS Apple Silicon DMG；
- Linux x86_64 AppImage。

Windows ARM64 和 Linux ARM64 可在 1.x 后续版本评估。macOS Intel/x64 不在当前发布范围内。

---

# 3. 可行性结论

## 3.1 总体判断

方案可行，且比继续改造 Prism 更符合 Mnelys 的长期目标。

Avalonia 能覆盖复杂桌面 UI、跨平台窗口、MVVM、无障碍和自定义样式；.NET 的自包含发布能让用户无需预装运行时。Windows 可以交付一个可直接运行的 EXE，macOS 和 Linux 分别通过标准 `.app`/DMG 与 AppImage 完成分发。

## 3.2 需要如实接受的工程事实

1. Windows 的“单 EXE”是分发层单文件。Avalonia/Skia 等原生库可能在启动时由 .NET 解包到临时目录；用户仍只下载和启动一个 EXE。
2. 严格意义上“不解包、无托管运行时的单一原生文件”需要 Native AOT，但 AOT 会限制动态加载、反射和部分第三方库，不适合作为未验证的首发硬前提。
3. macOS 的 DMG 只是容器，内部仍是标准 `.app`；公开分发必须完成 Developer ID 签名、Hardened Runtime 和 Apple 公证。
4. AppImage 需要选择兼容性足够老的 Linux 构建基线，并在 X11、XWayland、Wayland、不同桌面环境中验证。
5. 真正困难的部分不是 UI 框架，而是 Minecraft 元数据、加载器安装器、账户安全、三平台进程行为和大规模兼容性矩阵。

## 3.3 发布策略结论

默认采用两阶段策略：

```text
阶段 A：Self-contained + Single-file
  └─ 作为 Alpha、Beta 和 1.0 的可靠基线

阶段 B：Native AOT 技术验证
  └─ 达到兼容和体积目标后，按平台逐步启用
```

不允许为了追求文件体积而牺牲启动可靠性、崩溃可诊断性或 Provider 可维护性。

---

# 4. 产品定位与用户

## 4.1 一句话定位

> Mnelys 是一款兼顾玩家与模组开发者的 AI 原生 Minecraft Java Edition 启动器与开发工作区。

## 4.2 目标用户

### 普通玩家

- 希望快速安装并启动 Minecraft；
- 需要多实例、多账户和内容管理；
- 需要国内外网络线路自动选择；
- 不想理解复杂技术细节。

### 整合包玩家

- 需要导入、更新、复制和修复整合包；
- 需要隔离模组、配置、存档和 Java；
- 需要明确展示依赖冲突与版本兼容性。

### 模组开发者

- 需要把 Gradle 项目与测试实例绑定；
- 需要 Build、部署、Debug Run、JDWP 和统一日志；
- 需要本地工具、CLI/IDE 接入与 AI 诊断。

### 旧版本玩家

- 需要旧 Java、旧 LWJGL 和旧加载器适配；
- 接受部分组合被明确标为实验性，而不是被伪装成完全支持。

---

# 5. 产品原则

1. 单窗口优先，独立窗口只用于系统授权或平台强制流程；
2. 普通模式保持纯净，开发能力按需显现；
3. 长任务全部异步、可取消、可恢复，不阻塞 UI 线程；
4. UI 不直接处理 Token、下载、解压、安装器或启动命令；
5. 外部服务全部经过 Provider/Adapter 隔离；
6. 镜像只改变传输位置，不改变原始哈希和信任链；
7. 凭据不明文保存，日志默认脱敏；
8. 所有失败必须可定位阶段、可复制诊断信息、可重试；
9. 数据格式带 Schema 版本，迁移可回滚；
10. 默认设置对普通用户安全，高风险功能需显式开启；
11. 三平台功能语义一致，允许窗口装饰符合平台习惯；
12. 不使用 Mica 或系统背景模糊作为产品辨识度来源。

---

# 6. 版本范围与支持矩阵

## 6.1 1.0 必须支持

### 账户

- Microsoft 正版登录；
- 离线账户；
- 外置 Yggdrasil；
- 多账户、Token 刷新和实例绑定；
- 三平台安全凭据存储。

### 启动核心

- 版本元数据、安装、校验和修复；
- Java 发现、下载、选择和版本匹配；
- 实例创建、复制、导入、导出和备份；
- 启动参数、进程、实时日志和退出分析；
- 国内/国际源、测速、回退和缓存复用。

### 加载器

- Vanilla；
- Fabric；
- Quilt；
- Forge；
- NeoForge；
- Cleanroom；
- Legacy Fabric；
- LiteLoader/Rift 以验证结果决定正式或实验级别。

### 内容

- Modrinth；
- CurseForge（取得合规 API 授权后）；
- 本地文件和 URL 导入；
- 模组、整合包、资源包、光影、数据包和存档。

### 联机与开发者能力

- 陶瓦组件安装、修复、创建/加入房间、状态与诊断；
- 普通/开发者双模式；
- Gradle Wrapper Build、部署、Debug Run、JDWP；
- Developer Daemon 和最小 CLI/IDE 接入；
- 本地规则诊断与用户授权的 AI 崩溃分析。

## 6.2 1.0 不强制支持

- 移动端；
- 云端实例同步；
- 好友社交系统；
- 自建账户体系；
- 完整 IDE；
- 自动修改源码、自动提交 Git 或自动发布模组；
- 任意第三方二进制插件；
- Windows ARM64、Linux ARM64；
- macOS Intel/x64 与 Universal DMG；
- 所有远古版本与全部非主流加载器。

## 6.3 支持等级

| 等级 | 含义 |
|---|---|
| Tier A | CI 与人工矩阵完整覆盖，正式支持 |
| Tier B | 正式支持，但部分组合受版本/平台限制 |
| Tier C | 实验性，可能需要高级设置 |
| Tier D | 只保留接口或导入识别，不承诺运行 |

## 6.4 平台矩阵

| 平台 | 1.0 等级 | 发布物 |
|---|---:|---|
| Windows 10/11 x64 | Tier A | 单个自包含 `Mnelys.exe` |
| macOS 13+ arm64 | Tier A | 签名、公证 DMG |
| macOS 13+ x64 | Tier D | 不提供 1.0 发布物 |
| Linux x86_64 | Tier A | AppImage |
| Windows ARM64 | Tier D | 1.x 评估 |
| Linux ARM64 | Tier D | 1.x 评估 |

---

# 7. UI/UX 与无 Mica 规范

## 7.1 视觉方向

Mnelys 使用“稳定、不透明、内容优先”的桌面视觉：

- 根背景为不透明纯色或轻微渐变；
- 卡片通过明度差、细描边和低强度阴影分层；
- 深浅色均保持足够对比度；
- 动画表达状态变化，不承担装饰性炫技；
- 高密度开发界面优先可读性和键盘效率。

## 7.2 禁止项

- 禁止在 Windows 请求 `TransparencyLevelHint.Mica`；
- 禁止 `TransparencyLevelHint.AcrylicBlur`；
- 禁止透明根窗口采样桌面背景；
- 禁止调用非公开 DWM/Composition 模糊接口；
- 禁止把操作系统壁纸作为界面层次的一部分；
- 禁止为了“玻璃感”降低文本对比度。

Avalonia 窗口默认基线：

```xml
<Window
    WindowDecorations="None"
    ExtendClientAreaToDecorationsHint="True"
    TransparencyLevelHint="None"
    Background="{DynamicResource Color.Window.Background}">
</Window>
```

最终属性以锁定的 Avalonia 12.x API 为准。升级时必须检查窗口装饰破坏性变更。

## 7.3 窗口装饰策略

- Windows：自绘不透明标题栏，保留最小化、最大化、关闭和系统拖动/缩放语义；
- macOS：优先保留符合平台习惯的交通灯按钮与系统交互，内容区仍不透明；
- Linux：根据窗口管理器能力使用系统装饰或稳定的自绘装饰，不依赖合成器透明特性；
- 三平台都要支持系统菜单、Alt+F4/Command+Q、双击标题栏、最大化与键盘焦点。

## 7.4 设计 Token

设计系统只允许通过语义 Token 使用颜色和尺寸：

```text
Color.Window.Background
Color.Surface.Default
Color.Surface.Elevated
Color.Border.Subtle
Color.Text.Primary
Color.Text.Secondary
Color.Accent.Primary
Color.Status.Success / Warning / Error

Radius.Small / Medium / Large
Space.1 / 2 / 3 / 4 / 6 / 8
Motion.Fast / Normal / Emphasis
```

不允许在页面 AXAML 中散落品牌色、任意圆角和任意动画时长。

## 7.5 无障碍与输入

- 所有操作可使用键盘完成；
- 焦点环不可被主题隐藏；
- 控件提供 AutomationProperties 名称与帮助文本；
- 不只用颜色表达成功、警告或失败；
- 支持 100%～200% DPI，重点验证 125%、150%、200%；
- 支持系统减少动画设置；
- 首发至少完成简体中文和英文资源分离。

---

# 8. 单窗口与响应式架构

## 8.1 一级页面

```text
开始
实例
下载
联机
开发（仅开发者模式）
设置
```

账户中心、任务中心、资源详情、实例详情优先作为页面、抽屉或 Overlay 呈现，不创建无意义的独立窗口。

## 8.2 主窗口结构

```text
AppWindow
├─ TitleBar
├─ NavigationRail / CompactNavigation
├─ RouterHost
│  └─ CurrentPage
├─ OverlayHost
├─ DrawerHost
├─ DialogHost
├─ TaskCenter
└─ StatusBar
```

## 8.3 响应式断点

| 宽度 | 布局 |
|---:|---|
| `< 900` | 不支持作为常规工作区；显示最小尺寸约束 |
| `900–1199` | 紧凑导航、单列内容、抽屉详情 |
| `1200–1599` | 标准导航、双列详情 |
| `>= 1600` | 开发者面板可常驻，多栏日志与任务 |

推荐窗口初始尺寸 1280×800，最小 900×600。页面必须在缩放后保持主操作可见。

## 8.4 导航与状态

- 使用可序列化的 Route 描述页面，不让 ViewModel 持有 View；
- 页面恢复只保存无敏感状态；
- 模式切换不销毁当前实例和任务；
- 深链接仅映射到白名单 Route；
- 导航历史限制长度，避免持有大型页面对象。

---

# 9. Avalonia 技术基线

## 9.1 选型

| 领域 | 基线 |
|---|---|
| 语言 | C# 14，启用 Nullable 与隐式 using |
| 运行时 | .NET 10 LTS |
| UI | Avalonia 12.x 稳定版 |
| UI 模式 | MVVM、Compiled Bindings |
| 序列化 | `System.Text.Json` Source Generation |
| 数据库 | SQLite，经 Repository 隔离 |
| HTTP | `HttpClientFactory`、显式超时和策略 |
| 日志 | `Microsoft.Extensions.Logging` 抽象，结构化事件 |
| 测试 | xUnit + 领域测试 + UI 自动化 |
| 构建 | `dotnet` CLI + MSBuild |

MVVM 辅助库优先选择 CommunityToolkit.Mvvm；如其在 Avalonia 12/.NET 10 组合上出现 AOT 或绑定问题，可由内部最小基类替代。业务代码不得依赖具体 MVVM 包。

## 9.2 AOT 兼容约束

从第一天遵循 AOT 友好规则，但不强制首发 AOT：

- 使用 Compiled Bindings；
- JSON 使用 Source Generator；
- 禁止运行时扫描整个程序集自动注册服务；
- Provider 在编译期显式注册；
- 避免依赖 `Assembly.Load(byte[])` 的进程内插件；
- 对反射和动态代码保留清单与测试；
- 每个里程碑运行一次 `PublishAot` 试编译，记录阻塞项。

## 9.3 异步规则

- I/O API 必须接收 `CancellationToken`；
- 不允许 `.Result`、`.Wait()` 或 UI 线程同步网络请求；
- 进度通过只读事件流或任务快照上报；
- 后台异常必须被 TaskService 记录；
- UI 更新通过 Avalonia Dispatcher 的最小临界区完成。

---

# 10. 软件架构

## 10.1 分层

```text
Presentation
  Avalonia Views / ViewModels / Navigation / Design System
        ↓
Application
  Use Cases / Commands / Queries / Orchestration / DTO
        ↓
Domain
  Entity / Value Object / Policy / Port / Domain Event
        ↓
Infrastructure
  HTTP / SQLite / Filesystem / Archive / Process / Cache
        ↓
Providers & Platform
  Accounts / Loaders / Content / Multiplayer / OS adapters
```

依赖只能向内：Domain 不引用 Avalonia、SQLite、HTTP、平台 API 或具体 Provider。

## 10.2 进程边界

```text
Mnelys Desktop Process
├─ Avalonia UI
├─ Application + Domain
├─ Core background tasks
└─ Local IPC client

按需子进程
├─ Java / Minecraft
├─ Loader installer processor
├─ Terracotta / EasyTier component
├─ Developer Daemon
└─ Temporary update helper
```

下载、校验等普通任务首发可在主进程后台运行。高风险安装器、开发者外部接口和自更新替换必须有明确进程边界。

## 10.3 核心服务

```text
AccountService
InstanceService
MinecraftService
LoaderService
JavaService
DownloadService
ContentService
MultiplayerService
DeveloperService
AiService
TaskService
UpdateService
MigrationService
```

## 10.4 Provider 规则

Provider 是编译期注册的能力适配器，不等于任意 DLL 插件：

```csharp
public interface IContentProvider
{
    string Id { get; }
    Task<SearchPage> SearchAsync(SearchRequest request, CancellationToken ct);
    Task<ContentDetails> GetDetailsAsync(ContentId id, CancellationToken ct);
    Task<InstallPlan> ResolveInstallAsync(ContentInstallRequest request, CancellationToken ct);
}
```

- Provider 只实现外部协议差异；
- Use Case 决定业务流程；
- 所有 Provider 有能力声明和健康状态；
- 网络、文件和时间通过可替换抽象注入；
- 将来若开放第三方扩展，优先使用受限的进程外协议，而非加载不受信任 DLL。

## 10.5 UI 边界

View/ViewModel 不得：

- 读取或写入真实 Token；
- 拼接 Java 命令行字符串；
- 直接请求认证 API；
- 直接解压未知文件；
- 直接执行安装器；
- 直接修改实例关键文件。

---

# 11. 仓库与解决方案结构

```text
Mnelys/
├─ Mnelys.slnx
├─ Directory.Build.props
├─ Directory.Packages.props
├─ global.json
├─ README.md
├─ LICENSE
├─ docs/
│  ├─ Mnelys_完整产品与技术实施计划_Avalonia.md
│  ├─ ARCHITECTURE.md
│  ├─ DESIGN_SPEC.md
│  ├─ SECURITY.md
│  ├─ SUPPORT_MATRIX.md
│  └─ adr/
├─ src/
│  ├─ Mnelys.Desktop/
│  ├─ Mnelys.Presentation/
│  ├─ Mnelys.Application/
│  ├─ Mnelys.Domain/
│  ├─ Mnelys.Infrastructure/
│  ├─ Mnelys.Platform.Windows/
│  ├─ Mnelys.Platform.MacOS/
│  ├─ Mnelys.Platform.Linux/
│  ├─ Mnelys.Providers.Accounts/
│  ├─ Mnelys.Providers.Loaders/
│  ├─ Mnelys.Providers.Content/
│  ├─ Mnelys.Providers.Multiplayer/
│  ├─ Mnelys.DeveloperDaemon/
│  └─ Mnelys.UpdateHelper/
├─ tests/
│  ├─ Mnelys.Domain.Tests/
│  ├─ Mnelys.Application.Tests/
│  ├─ Mnelys.Infrastructure.Tests/
│  ├─ Mnelys.Providers.Tests/
│  ├─ Mnelys.Presentation.Tests/
│  ├─ Mnelys.Packaging.Tests/
│  └─ Mnelys.EndToEnd.Tests/
├─ fixtures/
├─ packaging/
│  ├─ windows/
│  ├─ macos/
│  └─ linux/
├─ tools/
└─ .github/workflows/
```

边界约束通过架构测试验证，例如 Domain 项目不得引用 `Avalonia.*`、平台项目不得被 Domain 引用。

---

# 12. 账户与凭据

## 12.1 账户类型

```csharp
public enum AccountType
{
    Microsoft,
    Offline,
    ExternalYggdrasil
}
```

## 12.2 登录流程

Microsoft：

```text
系统浏览器 OAuth + PKCE
→ state/redirect 校验
→ Microsoft Token
→ Xbox Live
→ XSTS
→ Minecraft Services
→ 所有权验证
→ 玩家资料
→ 安全保存 Refresh Token
```

外置 Yggdrasil：

- 首次添加展示域名、HTTPS 状态、元数据和信任提示；
- 支持角色选择和 authlib-injector；
- 密码只在认证调用生命周期内存在，不落盘；
- 服务器信息与凭据引用分离。

离线账户必须明确标记为离线，不伪装正版能力。

## 12.3 三平台凭据存储

| 平台 | 首选存储 |
|---|---|
| Windows | Windows Credential Manager / DPAPI 当前用户范围 |
| macOS | Keychain |
| Linux | Secret Service（libsecret） |

Linux 环境没有 Secret Service 时，不得静默降级为明文；应提示用户启用安全存储，或使用由用户设置主密码保护的加密 Vault。

配置文件只保存账户 ID、类型、显示名、UUID、Provider ID 和凭据引用。

---

# 13. Minecraft 版本、实例与 Java

## 13.1 实例原则

每个实例拥有独立：

- 游戏目录和配置；
- 模组、资源包、光影与存档；
- Java 选择与 JVM 参数；
- 账户绑定；
- 日志、崩溃报告和备份；
- 开发项目关联。

共享目录只保存可校验、可重新下载的 libraries、assets、Java 包和下载缓存。

## 13.2 创建流程

```text
选择 Minecraft 版本
→ 选择加载器与版本
→ 解析兼容 Java
→ 设置实例和账户
→ 生成不可变 InstallPlan
→ 用户确认
→ 后台下载/校验/安装
→ 原子提交实例
```

安装失败不得留下“看似可用”的半成品实例。

## 13.3 启动管线

```text
验证实例
→ 验证/刷新账户
→ 选择 Java
→ 校验游戏和加载器文件
→ 构建 Classpath
→ 构建 JVM 与游戏参数数组
→ 应用 authlib-injector / 调试 Agent
→ 启动 Java 进程
→ 捕获 stdout/stderr
→ 记录退出与崩溃分类
```

参数始终以数组传递给 `ProcessStartInfo.ArgumentList`，禁止 Shell 字符串拼接。

## 13.4 Java 管理

- 自动发现系统 Java；
- 读取发行版、架构和主版本；
- 根据 Minecraft/加载器规则推荐 Java；
- 可下载并校验托管 Java；
- 实例可覆盖全局 Java；
- macOS 只接受 arm64 Java，并在发现 x64 Java 时给出架构不兼容提示；
- Linux 检查可执行权限和动态依赖。

---

# 14. 加载器架构

## 14.1 分类

| 类型 | 示例 | 实施方式 |
|---|---|---|
| Metadata | Fabric、Quilt、Legacy Fabric | 获取元数据并合并版本模型 |
| Installer/Processor | Forge、NeoForge、Cleanroom | 隔离运行安装器/Processor，验证输出 |
| Legacy/Special | LiteLoader、Rift 等 | 独立规则包，明确版本边界 |

## 14.2 安装计划

所有加载器统一生成 `InstallPlan`：

```text
Resolve metadata
→ Resolve artifacts
→ Validate URLs and hashes
→ Download to cache
→ Run allowed processors
→ Verify outputs
→ Stage instance changes
→ Atomic commit or rollback
```

## 14.3 Processor 安全

- 在独立临时目录运行；
- 固定工作目录和环境变量；
- 只允许计划内输入输出路径；
- 设置超时、取消和进程树终止；
- 捕获标准输出、错误与退出码；
- 运行前后计算关键文件哈希；
- 不以管理员/root 权限运行；
- 失败时删除 staging，不覆盖有效实例。

## 14.4 兼容性数据

加载器规则与应用代码分离，但规则包必须签名、带 Schema、可回滚。远端规则不能直接引入可执行代码。

---

# 15. 下载、缓存与镜像

## 15.1 下载能力

- 并发与主机级限流；
- Range/断点续传；
- 临时文件与原子移动；
- SHA-1、SHA-256 和供应方声明哈希；
- 超时、指数退避和取消；
- HTTP 代理；
- 内容寻址缓存与去重；
- 国内/国际/自定义源；
- 单文件失败回退；
- 速度、阶段和失败原因可观测。

## 15.2 信任规则

镜像只能替换 URL，不得替换：

- 原始坐标；
- 预期大小；
- 官方或 Provider 提供的哈希；
- 签名验证结果。

哈希失败的节点立即隔离并记录健康度，不把错误文件交给安装阶段。

## 15.3 缓存

```text
下载请求
→ 查询内容哈希索引
→ 命中并校验：复用
→ 未命中：下载到 .partial
→ 校验
→ 原子进入对象缓存
→ 链接/复制到目标
```

缓存清理必须引用计数安全，不得删除仍被实例使用的文件。

---

# 16. 内容平台与资源安装

首发 Provider：

- Modrinth；
- CurseForge；
- 本地文件；
- URL 导入。

资源详情与安装面板必须展示：

- 项目来源、作者与许可证；
- Minecraft 版本与加载器兼容性；
- 必需/可选依赖；
- 客户端/服务端适用性；
- 不兼容项；
- 目标实例与变更预览；
- 下载源、哈希和安装结果。

依赖解析先生成计划，用户确认后执行。安装模组前创建可回滚快照，禁止直接覆盖未知同名文件。

---

# 17. 陶瓦联机

## 17.1 架构

```text
MultiplayerViewModel
  ↓
MultiplayerService
  ↓
IMultiplayerProvider
  ↓
TerracottaProvider
  ↓
Terracotta Core / EasyTier 子进程
```

## 17.2 能力

- 组件安装、版本锁定、哈希校验和修复；
- 创建、加入、离开房间；
- 房间码、成员、P2P/中继、延迟和日志；
- 与当前实例、账户和游戏进程关联；
- 导出脱敏诊断包。

## 17.3 安全

- 管理接口只监听本机；
- 使用随机短期 Token；
- 不自动开放公网端口；
- 子进程权限最小化；
- UI 提供完全关闭和清理组件选项；
- 网络状态和数据路径对用户透明。

---

# 18. 开发者工作区与 Daemon

## 18.1 两类后台能力

Core background tasks 始终可用，负责下载、安装、校验、游戏进程和日志。

Developer Daemon 只在开发者明确启用时运行，负责：

- CLI/IDE/Agent 接入；
- 构建与调试事件；
- 项目与实例关联；
- AI 工具桥接；
- 本地受限自动化。

## 18.2 IPC

| 平台 | 首选 IPC |
|---|---|
| Windows | Named Pipe + 当前用户 ACL |
| macOS/Linux | Unix Domain Socket + `0600` 权限 |

可选本机 WebSocket 必须绑定 loopback、校验 Origin、使用短期 Token 并按 Session 授权。

## 18.3 Build 与 Debug Run

```text
检查项目与 JDK
→ 调用项目内 Gradle Wrapper
→ 解析受允许任务
→ 构建
→ 定位产物
→ 部署到绑定实例
→ 可选启用 JDWP/HotSwapAgent
→ 启动游戏
→ 汇总构建与运行日志
```

自定义 Shell 命令默认禁用；高级模式开启前必须展示风险和具体命令。

---

# 19. AI 与崩溃诊断

## 19.1 本地确定性诊断优先

AI 之前先运行可测试规则：

- Java 版本不匹配；
- 内存不足；
- 模组重复或缺失依赖；
- Mixin/类加载典型错误；
- 网络、权限和磁盘错误；
- 加载器安装不完整；
- 退出码与崩溃文件关联。

## 19.2 AI 阶段

第一阶段：日志摘要、错误分类、可能原因、下一步建议和关键堆栈高亮。

第二阶段：模组冲突、启动参数、构建错误、Java 与内存建议。

第三阶段：在用户授权后读取项目上下文，生成修复草案和 IDE 跳转建议；不自动修改或提交源码。

## 19.3 隐私

发送前完成：

```text
收集候选内容
→ 删除 Token、密码、主目录、私有地址等敏感字段
→ 向用户展示预览
→ 用户确认模型、服务和发送范围
→ 请求
→ 本地保存可清除的诊断记录
```

默认不上传日志、源码、Token、服务器地址或账户信息。AI 不可成为启动、修复或卸载的单点依赖。

---

# 20. 任务、通知与可恢复操作

统一任务状态：

```text
Queued → Resolving → Running → Verifying → Finalizing → Completed
                                  └──────────────→ Failed
Queued/Running ─────────────────────────────────→ Cancelled
```

每个任务包含：

- ID、类型、标题和目标对象；
- 当前阶段、进度、速度和预计时间；
- `CancellationToken` 与是否可重试；
- 结构化错误码与用户可读建议；
- 诊断事件和日志引用；
- 应用重启后的恢复策略。

通知不得用全屏弹窗打断。需要确认的破坏性操作使用应用内 DialogHost，并说明影响范围与恢复方式。

---

# 21. 数据模型与目录

## 21.1 平台目录

| 平台 | 默认数据根目录 |
|---|---|
| Windows | `%LOCALAPPDATA%\Mnelys`，凭据另存系统保险库 |
| macOS | `~/Library/Application Support/Mnelys` |
| Linux | `$XDG_DATA_HOME/Mnelys`，默认 `~/.local/share/Mnelys` |

缓存和日志分别遵守平台 Cache/State 目录。不得假设应用目录可写。

## 21.2 逻辑结构

```text
MnelysData/
├─ config/
├─ instances/
├─ cache/
│  ├─ objects/
│  └─ index/
├─ libraries/
├─ assets/
├─ java/
├─ loaders/
├─ multiplayer/
├─ logs/
├─ tasks/
├─ updates/
└─ developer/
```

## 21.3 配置与数据库

- 小型、可导出的用户设置使用带 `schemaVersion` 的 JSON；
- 任务历史、索引和搜索缓存使用 SQLite；
- 写入采用临时文件 + fsync 能力评估 + 原子替换；
- 迁移前备份，迁移失败回退；
- 数据库迁移必须支持从最后两个稳定版升级；
- 用户实例不与应用卸载绑定。

## 21.4 便携模式

使用 `--portable` 或可执行文件同目录的明确标记启用。启用时：

- 数据写入 `MnelysData` 子目录；
- 启动时检查目录可写；
- 凭据仍优先使用系统安全存储；
- 不在 `Program Files` 等只读位置静默失败；
- UI 明确显示“便携模式”。

---

# 22. 安全设计

## 22.1 威胁范围

- 账户 Token 或密码泄漏；
- 恶意镜像、更新或规则包；
- ZIP Slip、路径穿越和符号链接逃逸；
- 加载器 Processor 任意执行；
- 模组与整合包携带恶意内容；
- Developer Daemon 被其他用户或网页访问；
- 命令注入；
- 日志和 AI 请求泄密；
- 更新替换到错误路径。

## 22.2 必须控制

- HTTPS、哈希、数字签名和证书有效性；
- 规范化路径后验证其仍在允许根目录；
- 解压前检查条目、大小上限和链接；
- 参数数组化，不经过 Shell；
- IPC 当前用户权限和短期 Token；
- 更新 Manifest 防降级、防重放；
- 凭据系统保险库；
- 日志结构化脱敏；
- SBOM、依赖许可证与漏洞扫描；
- 默认非管理员权限运行。

## 22.3 安全评审门

Microsoft 登录、外置登录、Processor、Daemon、自更新和 AI 上传在进入 Beta 前分别完成威胁建模与滥用用例测试。

---

# 23. 更新系统

## 23.1 更新清单

签名 Manifest 至少包含：

- 版本、渠道和发布时间；
- 平台、架构、RID；
- 完整包 URL、大小和 SHA-256；
- 最低数据 Schema；
- 是否强制更新及原因；
- 发布说明 URL；
- 签名 Key ID 和签名。

根公钥固定在应用中，密钥轮换通过双签名完成。

## 23.2 Windows 更新

运行中的 EXE 不能可靠覆盖自身。流程为：

```text
应用下载并验证新 EXE
→ 从已签名资源释放最小 UpdateHelper 到随机临时目录
→ 启动 Helper，传递当前 PID、目标路径和哈希
→ 主程序退出
→ Helper 再次验证并原子替换
→ 启动新版本
→ 健康检查失败时回滚旧 EXE
```

用户下载物仍然只有一个 EXE；UpdateHelper 只在更新时临时释放。

## 23.3 macOS 与 Linux 更新

- macOS：Stable 首发允许“下载已公证 DMG并引导替换”；自动替换必须维持代码签名和应用 Bundle 完整性；
- Linux：下载并验证新 AppImage，退出后替换当前用户可写位置的旧文件；不可写时提示保存到新位置；
- 所有平台均保留手动下载和跳过当前版本能力。

---

# 24. Windows 单 EXE 交付

## 24.1 目标定义

Windows 用户只需下载一个 `Mnelys.exe`：

- 无安装向导；
- 不需要管理员权限；
- 不需要预装 .NET；
- 不携带外部 DLL/资源目录；
- 可从任意用户可读位置启动；
- 首次启动创建用户数据目录。

## 24.2 发布基线

```xml
<PropertyGroup Condition="'$(RuntimeIdentifier)' == 'win-x64'">
  <SelfContained>true</SelfContained>
  <PublishSingleFile>true</PublishSingleFile>
  <IncludeNativeLibrariesForSelfExtract>true</IncludeNativeLibrariesForSelfExtract>
  <EnableCompressionInSingleFile>true</EnableCompressionInSingleFile>
  <DebugType>embedded</DebugType>
</PropertyGroup>
```

实际属性必须在锁定的 .NET 10 SDK 与 Avalonia 版本上通过发布测试确认。

推荐命令：

```powershell
dotnet publish src/Mnelys.Desktop/Mnelys.Desktop.csproj `
  -c Release -r win-x64 --self-contained true `
  -p:PublishSingleFile=true `
  -p:IncludeNativeLibrariesForSelfExtract=true
```

## 24.3 单文件验收

在干净 Windows Sandbox/VM 中验证：

1. 机器没有单独安装 .NET；
2. 发布目录只向用户交付一个 EXE；
3. 双击后可以进入主窗口；
4. 路径包含中文、空格和长路径时可运行；
5. 临时目录不可写时给出明确错误；
6. EXE 有 Authenticode 签名；
7. SmartScreen 信誉作为发布运营事项持续积累；
8. 更新和回滚不破坏用户数据。

## 24.4 Native AOT 试验门槛

仅当以下条件全部满足时，才将 `PublishAot=true` 提升为正式路径：

- Avalonia、Skia、SQLite、序列化、凭据和日志依赖全部通过；
- 没有未声明的反射/动态程序集加载；
- 登录、加载器、内容、Daemon 与更新完整回归；
- 启动时间、内存和体积有可量化收益；
- 崩溃符号和诊断能力不低于单文件 JIT 版本。

---

# 25. macOS DMG 交付

## 25.1 发布物

首发提供一个 Apple Silicon DMG：

```text
Mnelys-<version>-macos-arm64.dmg
```

每个 DMG 内包含标准 `Mnelys.app` 和 Applications 快捷方式，用户打开后拖拽安装。

## 25.2 构建流程

```text
dotnet publish -r osx-<arch> --self-contained
→ 组装 Mnelys.app/Contents
→ 写入 Info.plist、图标和权限说明
→ codesign 所有嵌套二进制
→ 启用 Hardened Runtime
→ 验证签名
→ 提交 Apple Notary Service
→ stapler 附加公证票据
→ hdiutil 创建 DMG
→ 再次校验 Gatekeeper
```

签名和公证只能在受保护的发布 Job 中执行，证书和 App Store Connect 凭据不进入仓库。

## 25.3 macOS 验收

- Apple Silicon 真机安装；
- 首次启动无未识别开发者警告；
- 从只读 DMG 误启动时提示拖入 Applications；
- Keychain 凭据工作；
- 文件选择、系统浏览器 OAuth 和 Java 子进程工作；
- App Translocation 情况不破坏路径发现；
- 升级不删除 `Application Support` 中的用户数据。

---

# 26. Linux AppImage 交付

## 26.1 发布物

```text
Mnelys-<version>-x86_64.AppImage
```

AppDir 至少包含：

```text
AppDir/
├─ AppRun
├─ Mnelys.desktop
├─ mnelys.png / hicolor icons
└─ usr/bin/Mnelys + runtime payload
```

## 26.2 构建原则

- 在受控、足够老的 glibc 基线上构建；
- 自包含 .NET，不要求用户安装运行时；
- 打包 Avalonia 需要的原生库并审计许可证；
- 不把系统 GPU/窗口系统库盲目全部内置；
- desktop entry 的 `Exec`、`Icon`、`StartupWMClass` 一致；
- AppImage 文件提供 SHA-256 和签名。

## 26.3 Linux 验收矩阵

- Ubuntu LTS、Fedora、Arch 系代表环境；
- GNOME 与 KDE；
- X11、XWayland；
- 原生 Wayland 仅在 Avalonia 稳定支持和回归通过后启用；
- 有/无 FUSE 环境；
- Secret Service 存在与缺失；
- NVIDIA、AMD、Intel 的基本渲染冒烟；
- `/tmp` 带 `noexec` 或 AppImage 需要解包回退时给出明确指引。

---

# 27. CI/CD 与发布渠道

## 27.1 提交级 CI

- `dotnet format`/格式检查；
- Restore 锁定检查；
- 编译并将警告视为错误；
- 单元、架构和 Provider 合约测试；
- AXAML 编译与 Compiled Binding 检查；
- JSON Source Generator/AOT 分析警告；
- 依赖漏洞、许可证与秘密扫描；
- Windows/macOS/Linux 冒烟构建。

## 27.2 发布级流水线

```text
版本冻结
→ 三平台矩阵构建
→ 单元/集成/E2E
→ 生成 SBOM 与 notices
→ 平台签名
→ macOS 公证
→ 包结构与干净机冒烟
→ 生成并签名 Update Manifest
→ 发布到渠道
→ 分阶段放量
```

## 27.3 渠道

| 渠道 | 用途 | 更新策略 |
|---|---|---|
| Nightly | 开发验证 | 不保证数据向后兼容，独立数据目录 |
| Alpha | 功能验证 | 小范围，允许缺少部分 Provider |
| Beta | 全平台兼容性 | 数据迁移稳定，支持回滚 |
| Stable | 正式使用 | 分批发布、可暂停、强签名 |

Release 构建只能来自受保护 Tag；本地构建不得上传到正式更新源。

---

# 28. 测试计划

## 28.1 单元测试

- 版本继承与参数规则；
- Java 选择；
- 实例 Schema 与迁移；
- 下载计划、哈希和缓存；
- 加载器 InstallPlan；
- 依赖解析；
- 路径规范化与解压安全；
- 日志脱敏；
- 更新 Manifest 验签。

## 28.2 集成测试

- 使用录制/伪造 HTTP 响应测试账户与元数据，不在 CI 保存真实凭据；
- 在临时目录完成 Vanilla 安装；
- 对 Fabric/Quilt/Forge/NeoForge 运行固定版本夹具；
- 使用本地测试服务器验证 Range、重试、回退和损坏文件；
- SQLite 迁移与崩溃恢复；
- 子进程取消、超时和进程树清理；
- 三平台凭据 Adapter 合约。

## 28.3 E2E

核心黄金路径：

```text
首次启动
→ 创建离线测试身份
→ 创建 Vanilla 实例
→ 下载与校验
→ 启动到主菜单
→ 正常退出
→ 查看日志
```

然后扩展到账户、加载器、内容、陶瓦、开发者 Build/Debug Run 和更新回滚。

## 28.4 兼容矩阵控制

不测试所有笛卡尔积。采用：

- 每个 Provider 的固定代表版本；
- 最新稳定版；
- 最低受支持版；
- 已知高风险版本；
- 每周扩展矩阵；
- Stable 前完整矩阵。

失败组合进入公开支持矩阵，禁止仅在代码注释中记录。

---

# 29. 可观测性、性能与无障碍

## 29.1 本地日志

- 结构化事件、UTC 时间、Correlation ID；
- 默认不记录 Token、密码、完整命令行敏感参数；
- 按大小与日期轮转；
- 用户可一键导出脱敏诊断包；
- 遥测默认关闭或首次明确选择，策略必须独立成文。

## 29.2 性能预算

以常规 SSD、4 核 CPU 的支持设备为基线：

- 冷启动到可交互窗口目标 `< 2.5s`；
- 首屏无网络时目标 `< 1.5s`；
- 空闲内存目标 `< 250MB`，以实际 Avalonia/.NET 基线复核；
- 10,000 个本地内容项采用虚拟化，不一次创建全部控件；
- UI 线程不得执行超过 50ms 的磁盘或计算任务；
- 下载任务不会导致滚动和窗口拖动明显卡顿。

## 29.3 无障碍验收

- Windows Narrator、macOS VoiceOver、Linux 主流 AT 基本冒烟；
- 键盘完整遍历、焦点顺序正确；
- 200% DPI 和高对比主题；
- 错误提示可被读屏读取；
- 动画可减少或关闭。

---

# 30. 许可证与合规

## 30.1 项目原则

- 不复制 Prism Launcher 代码与 API Key；
- 不声称是 Prism 官方衍生产品；
- Avalonia 本身按其 MIT 许可证合规使用；
- .NET 运行时、Skia、SQLite、authlib-injector、EasyTier、图标、字体等逐项记录；
- 分发前核查 Microsoft、Mojang、Modrinth、CurseForge 和第三方认证条款；
- 不暗示与 Mojang/Microsoft 存在官方关联。

## 30.2 必须生成

```text
THIRD_PARTY_NOTICES.md
DEPENDENCIES.json
SBOM.spdx.json
LICENSES/
```

CI 阻止许可证未知、禁止分发或缺少 Notice 的依赖进入 Release。

## 30.3 品牌与资产

- Mnelys 名称、Logo、应用 ID、签名主体在 M0 冻结；
- 不打包未授权 Minecraft 资产；
- 启动器下载内容遵循官方分发机制和用户授权；
- 截图、字体、图标和音效保留来源记录。

---

# 31. 里程碑与排期

跨平台与全加载器目标很大，排期采用可交付切片，不允许每个模块都开发 80% 后一起集成。

## M0：独立立项与技术验证（2～3 周）

- 新仓库、许可证、应用 ID 和文档；
- Avalonia 12/.NET 10 版本锁定；
- Windows 单 EXE、macOS DMG、Linux AppImage 空壳 PoC；
- 无 Mica 窗口 PoC；
- Native AOT 兼容性初测；
- CI 三平台构建。

退出条件：三个平台都能从目标发布物启动同一个不透明 Mnelys Shell。

## M1：Design System 与 Shell（4～6 周）

- Token、深浅色、控件状态和无障碍基线；
- 单窗口、导航、Overlay、Drawer、DialogHost；
- 普通/开发者模式状态；
- 设置、日志、任务中心；
- 响应式和 DPI 测试。

## M2：Vanilla 垂直切片（7～11 周）

- 元数据、下载、缓存、校验；
- Java 检测和选择；
- 实例模型、离线测试身份；
- 启动、进程和日志；
- 三平台启动到主菜单。

## M3：账户与安全存储（5～8 周）

- Microsoft、离线、外置 Yggdrasil；
- Windows Credential Manager/DPAPI、Keychain、Secret Service；
- Token 刷新、所有权验证、实例绑定；
- 认证威胁模型。

## M4：Metadata 加载器（4～7 周）

- Fabric、Quilt、Legacy Fabric；
- Provider 合约与兼容矩阵；
- 安装回滚。

## M5：Installer 加载器（8～14 周）

- Forge、NeoForge；
- Processor 隔离、超时、回滚和诊断；
- Cleanroom 初步支持。

## M6：内容平台与源（6～10 周）

- Modrinth、CurseForge 合规接入；
- 内容详情、依赖解析和安装；
- 国内/国际源、测速和回退；
- 导入/导出。

## M7：陶瓦联机（4～7 周）

- 组件生命周期、房间、状态、日志与修复；
- 三平台子进程和网络验证；
- 安全评审。

## M8：开发者工作区（8～12 周）

- 项目绑定、Gradle Build、部署；
- Debug Run、JDWP；
- Developer Daemon 与 IPC；
- CLI/IDE 最小接口。

## M9：AI 与恢复体验（5～9 周）

- 本地确定性崩溃规则；
- 脱敏、预览和 AI Provider；
- 模组冲突与构建错误摘要；
- 诊断包和修复建议。

## M10：特殊加载器与兼容收口（5～9 周）

- Cleanroom 完整矩阵；
- LiteLoader/Rift 可行性落地；
- 旧 Java/LWJGL；
- 支持等级固化。

## M11：全平台发布与更新（6～10 周）

- Windows 签名单 EXE与更新 Helper；
- macOS arm64 DMG、签名和公证；
- Linux AppImage 和桌面集成；
- Manifest、分批发布和回滚。

## M12：Beta 稳定化（8～12 周）

- 全矩阵测试、性能、无障碍和安全复核；
- 数据迁移演练；
- 文档、许可证、SBOM；
- 公测、崩溃修复和 1.0 候选版。

## 31.1 总周期估算

里程碑可并行，不能简单相加。包含三平台正式交付后的现实区间：

| 团队 | 可用 Alpha | 跨平台 Beta | 1.0 |
|---|---:|---:|---:|
| 单人全职 | 7～10 个月 | 18～26 个月 | 24～36 个月 |
| 单人业余 | 12～20 个月 | 30～48 个月 | 40～60 个月 |
| 2～3 人全职 | 4～7 个月 | 10～15 个月 | 14～20 个月 |
| 4～5 人全职 | 3～5 个月 | 8～12 个月 | 11～16 个月 |

如果必须缩短 1.0 周期，应延后 AI 第三阶段、特殊加载器和高级开发者工具，不应削减账户安全、校验、回滚或平台签名。

---

# 32. 人员分工

## 32.1 最小 2～3 人

### 核心/平台工程

- Minecraft 元数据、Java、启动与进程；
- 下载缓存、加载器 Processor；
- Windows/macOS/Linux Adapter；
- 打包、签名、更新和安全。

### UI/产品工程

- Avalonia Shell、Design System 和响应式；
- 账户、实例、下载、内容、联机页面；
- ViewModel、无障碍和 UI 自动化。

### 服务/质量（可由前两人兼任，但风险较高）

- 内容 Provider、陶瓦、Daemon、AI；
- 测试夹具、兼容矩阵、发布运营和文档。

## 32.2 推荐 4～5 人

1. Minecraft Core/Loader；
2. Avalonia UI/Product；
3. Platform/Packaging/Security；
4. Content/Multiplayer/Developer；
5. QA/Automation，可与产品或平台角色部分合并。

---

# 33. 风险与控制

| 风险 | 影响 | 控制 |
|---|---|---|
| 1.0 范围膨胀 | 周期失控 | 垂直切片、Tier、每阶段退出条件 |
| Avalonia 12 API 变化 | 窗口/绑定回归 | 锁定小版本、升级 ADR、三平台 UI 冒烟 |
| 单 EXE 原生库解包 | 安全软件或临时目录问题 | 干净机测试、签名、明确诊断、AOT 试验 |
| Native AOT 不兼容 | 发布延期 | AOT 不是 1.0 必需，持续试编译 |
| macOS 签名/公证失败 | 无法分发 | M0 即做空壳 PoC，专用发布 Job |
| Linux 环境碎片化 | AppImage 启动失败 | 老基线、代表发行版矩阵、X11/XWayland 回退 |
| 加载器频繁变化 | 实例安装失败 | Provider 隔离、远端签名规则、固定夹具 |
| Microsoft 登录变化 | 正版登录中断 | 分阶段错误、监控官方变更、Provider 合约测试 |
| 外置登录泄密 | 高安全风险 | 系统保险库、首次信任、日志脱敏 |
| 镜像内容污染 | 供应链风险 | 原始哈希、节点隔离、签名 Manifest |
| Daemon 被滥用 | 本机攻击面 | 默认关闭、用户 ACL、短期 Token、最小权限 |
| AI 幻觉或泄密 | 错误修复/隐私风险 | 本地规则优先、预览确认、不自动执行 |
| 与 Prism 代码混入 | 许可证与独立性风险 | 新仓库、来源记录、代码评审清洁室规则 |

---

# 34. 验收标准

## 34.1 产品

- 三种账户完成登录、刷新、退出和实例绑定；
- 能创建、安装、修复并启动支持矩阵中的实例；
- 能安装内容、处理依赖并回滚失败变更；
- 能切换线路并理解下载失败；
- 能使用陶瓦创建/加入房间；
- 普通模式全流程不出现不必要的独立窗口。

## 34.2 开发者

- 能绑定 Gradle 项目与实例；
- 能 Build、部署、Debug Run 和使用 JDWP；
- 能查看统一构建/运行日志和 AI 诊断；
- 关闭开发者模式后 Daemon 不再接受外部连接。

## 34.3 UI 与无 Mica

- 900×600 可用，1280×800 体验完整；
- 200% DPI 清晰，键盘与读屏基本可用；
- 深浅色切换不丢状态；
- Windows 根窗口不透明，未请求 Mica/Acrylic；
- 代码中不存在 DWM 非公开背景模糊调用；
- 长任务不阻塞窗口拖动、滚动和导航。

## 34.4 发布物

- Windows：干净机只下载一个签名 EXE，无 .NET 前置安装即可启动；
- macOS：arm64 DMG 已签名、公证并通过 Gatekeeper；
- Linux：AppImage 在目标矩阵启动并能找到数据目录；
- 三平台更新包验签、替换与回滚通过；
- 卸载或替换应用不删除实例和账户数据。

## 34.5 安全与数据

- Token/密码不明文；
- 更新、规则包和下载文件可验证；
- 解压和 Processor 无路径逃逸；
- 配置与数据库迁移失败可回退；
- 诊断包不包含预定义敏感信息；
- Release 附带 SBOM 和第三方许可证。

---

# 35. 立项后前八周任务

## 第 1 周：新项目边界

- 在 `D:\Project\Mnelys` 初始化新 Git 仓库；
- 冻结产品名、应用 ID、命名空间和许可证；
- 建立 ADR、Security 和 Support Matrix；
- 明确旧 Prism 仓库只读规则；
- 锁定 .NET 10 SDK 与 Avalonia 12.x 稳定版。

## 第 2 周：三平台发布 PoC

- 创建最小 Avalonia Desktop；
- Windows 发布单 EXE并在无 .NET VM 启动；
- macOS 组装 `.app` 与测试 DMG；
- Linux 组装 AppImage；
- 记录原生库、体积、启动时间和解包行为；
- 试编译 Native AOT，建立阻塞清单。

## 第 3～4 周：无 Mica Shell

- 设计 Token 和深浅主题；
- Windows 不透明自绘标题栏；
- macOS/Linux 窗口装饰适配；
- Navigation、RouterHost、Overlay、Drawer、DialogHost；
- 响应式断点、键盘焦点和基本无障碍；
- 静态开始、实例、下载、联机和设置页面。

## 第 5～6 周：核心骨架

- Domain/Application/Infrastructure 分层；
- TaskService、日志、设置、SQLite Migration；
- HTTP、文件、进程、压缩和凭据 Port；
- Provider 显式注册与合约测试；
- JSON Source Generation 与架构测试。

## 第 7～8 周：首个 Vanilla 垂直切片

- 获取版本元数据；
- 下载、哈希、缓存与进度；
- Java 发现；
- 实例 Schema 与原子创建；
- 离线测试身份；
- 构建启动参数并捕获日志。

首个技术验收目标：

> 在完全独立的 Avalonia Mnelys 中，从三个目标发布物启动应用，创建 Vanilla 实例并至少在 Windows 上进入游戏主菜单；macOS/Linux 完成相同管线的持续集成和人工冒烟，不存在 Prism 运行时或代码依赖，也不启用 Mica。

---

# 36. 待确认事项

以下事项不会阻塞 M0，但应在对应里程碑前冻结：

1. “Lift”具体指 LiteLoader、Rift 还是其他加载器；
2. LiteLoader/Rift 是否允许在 1.0 标记为 Tier C；
3. 是否导入 Prism/MultiMC、HMCL、PCL2 实例，以及仅导入哪些字段；
4. CurseForge API 的申请主体和分发条款；
5. 陶瓦组件的正式名称、许可证、签名和更新来源；
6. AI 采用云端、本地模型还是两者并存；
7. 是否提供官方国内镜像，或只允许用户配置第三方镜像；
8. 项目整体开源范围与服务端组件边界；
9. Windows EXE 是否额外提供可选 ZIP 校验包；
10. Linux 原生 Wayland 何时从实验路径升级为默认；
11. 开发者模式 1.0 是否必须包含 DCEVM/HotSwapAgent。

---

# 37. 结论

Mnelys 的新方向可执行，推荐路线已经收敛为：

```text
建立完全独立的新仓库
→ 用 Avalonia 12 + .NET 10 完成不透明、无 Mica 的跨平台 Shell
→ 先打通 Windows 单 EXE / macOS DMG / Linux AppImage 空壳发布链
→ 完成 Vanilla 垂直切片
→ 逐步加入账户、加载器、内容与陶瓦
→ 建立开发者 Daemon 与 AI 诊断
→ 全平台兼容、安全、签名和更新收口
```

Avalonia 是此项目的产品最优解：它比 Qt 静态单文件路线减少了授权和构建复杂度，比 WebView 路线更适合稳定的桌面工作区，也比更轻量的新框架更能承载实例管理、日志、任务中心和开发者面板。

首发应以 .NET 自包含单文件作为 Windows 可靠基线，把 Native AOT 作为持续验证的优化项，而不是项目成败条件。macOS 与 Linux 从 M0 就进入构建和测试链，避免在 Windows 功能完成后才发现跨平台架构问题。

最重要的工程纪律有三条：

1. 不从旧 Prism 仓库复制实现；
2. 不启用 Mica/Acrylic 或系统背景材质；
3. 每个里程碑都交付可运行、可验证、可回滚的垂直切片。

---

## 参考基线

- Avalonia 官方文档：<https://docs.avaloniaui.net/>
- Avalonia macOS 部署：<https://docs.avaloniaui.net/docs/deployment/macos>
- Avalonia Linux 部署：<https://docs.avaloniaui.net/docs/deployment/linux>
- Avalonia Windows 平台指南：<https://docs.avaloniaui.net/docs/platform-specific-guides/windows>
- .NET 单文件部署：<https://learn.microsoft.com/dotnet/core/deploying/single-file/overview>
- .NET Native AOT：<https://learn.microsoft.com/dotnet/core/deploying/native-aot/>

> 版本说明：本计划按 2026-07-19 可用的 .NET 10 与 Avalonia 12 文档编写。创建仓库时应再次锁定具体 SDK/包补丁版本，并把版本号写入 `global.json` 与 `Directory.Packages.props`。
