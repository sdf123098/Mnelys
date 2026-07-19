<p align="center">
  <img src="assets/branding/mnelys-icon-source.jpg" width="220" alt="Mnelys icon">
</p>

<h1 align="center">Mnelys · 忆涟</h1>

<p align="center">
  An AI-native Minecraft: Java Edition launcher and development workspace for players and mod developers
</p>

<p align="center">
  <a href="README.md">简体中文</a> · English · <a href="README.ja.md">日本語</a>
</p>

> [!IMPORTANT]
> Mnelys is still in the M0 technical-validation phase. The repository currently contains a runnable Avalonia shell and cross-platform packaging proofs of concept, not a complete Minecraft installation and launch workflow.

## About

Mnelys is an independently developed, cross-platform Minecraft: Java Edition launcher. It is designed to serve regular players and mod developers through one shared account, instance, task, cache, and process model.

The project does not inherit Prism Launcher source code, assets, Git history, or internal data models. Compatibility is implemented only from public protocols, official APIs, public file formats, and independently authored test fixtures.

## Planned capabilities

- Microsoft, offline, and external Yggdrasil accounts
- Vanilla, Fabric, Quilt, Forge, NeoForge, and other instance types
- Automatic Java discovery, download, and version matching
- Modrinth, CurseForge, and local content installation
- Download caching, hash verification, mirror fallback, and recovery
- Terracotta multiplayer component integration
- Gradle build, deployment, Debug Run, JDWP, and a developer daemon
- Local-rule-first crash diagnostics with explicitly authorized AI analysis

These items describe the roadmap and are not all implemented today.

## Current status

Mnelys is currently at M0, independent setup and technical validation:

- [x] Independent repository, license, ADRs, and security baseline
- [x] Pinned .NET 10, C# 14, and Avalonia 12 toolchain
- [x] Opaque Avalonia Desktop shell without Mica or Acrylic
- [x] Windows x64 self-contained single-file proof of concept
- [x] macOS arm64 self-contained payload and `.app` layout proof of concept
- [x] Linux x64 self-contained payload and AppDir layout proof of concept
- [ ] macOS hardware signing, notarization, and DMG validation
- [ ] Linux AppImage builder and distribution-matrix validation
- [ ] First end-to-end Vanilla slice

See the [M0 packaging PoC report](docs/M0_PACKAGING_POC.md) for evidence and remaining gates.

## Platform scope

| Platform | 1.0 target | Artifact |
|---|---:|---|
| Windows 10/11 x64 | Tier A | Self-contained single `Mnelys.exe` |
| macOS 13+ Apple Silicon | Tier A | Signed and notarized DMG |
| Linux x86_64 | Tier A | AppImage |
| macOS Intel/x64 | Unsupported | No artifact |
| Windows ARM64 / Linux ARM64 | Deferred | To be evaluated after 1.0 |

## Technology

- C# 14 and .NET 10 LTS
- Avalonia 12, AXAML, and compiled bindings
- MVVM with one-way state updates
- Central Package Management
- `System.Text.Json` source generation for upcoming domain slices
- SQLite, `HttpClientFactory`, and structured logging in the upcoming core skeleton

Dependencies point inward: Presentation → Application → Domain. Infrastructure, providers, and platform adapters implement ports defined by the inner layers. See the [architecture baseline](docs/ARCHITECTURE.md).

## Run locally

Prerequisites:

- .NET SDK `10.0.302`, or a later `10.0.3xx` patch accepted by `global.json`
- Git 2.40+

```powershell
git clone https://github.com/sdf123098/Mnelys.git
cd Mnelys
dotnet restore Mnelys.slnx
dotnet run --project src/Mnelys.Desktop/Mnelys.Desktop.csproj
```

Verify the Release build and formatting:

```powershell
dotnet build Mnelys.slnx -c Release
dotnet format Mnelys.slnx --verify-no-changes --no-restore
```

## Packaging proofs of concept

Windows x64:

```powershell
.\packaging\windows\Publish-SingleFile.ps1
```

macOS arm64:

```bash
./packaging/macos/build-dmg.sh
```

Linux x64:

```bash
./packaging/linux/build-appimage.sh
```

See the [packaging guide](packaging/README.md) for target-host tools, signing requirements, and current limitations.

## Repository layout

```text
Mnelys/
├─ assets/                 Canonical branding assets
├─ docs/                   Plans, architecture, security, support matrix, and ADRs
├─ packaging/              Windows, macOS, and Linux packaging scripts
├─ src/Mnelys.Desktop/     Current Avalonia desktop shell
├─ Directory.Build.props   Repository-wide build rules
├─ Directory.Packages.props Central package versions
├─ global.json             .NET SDK pin
└─ Mnelys.slnx             Solution entry point
```

## Contributing and clean-room boundary

Read [CONTRIBUTING.md](CONTRIBUTING.md) before submitting changes. Do not copy or adapt source code, assets, translations, credentials, CI files, packaging scripts, or internal models from Prism Launcher, MultiMC, HMCL, PCL2, or other launchers.

Follow [SECURITY.md](docs/SECURITY.md) when reporting a vulnerability. Never place credentials or unredacted personal data in a public issue.

## License and disclaimer

Source code is released under the [MIT License](LICENSE).

Mnelys is not an official Mojang Studios or Microsoft product and is not endorsed by or affiliated with either company. Minecraft is a trademark of its respective owners. Original authorship and public redistribution terms for the branding artwork must be recorded before the first public release.
