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
> Mnelys is moving its technical baseline from .NET/Avalonia to **Tauri v2** (see [ADR 0005](docs/adr/0005-runtime-and-ui-baseline-tauri.md)). `main` currently holds only project documentation, brand assets, and decision records — there is **no runnable implementation** yet, and no Minecraft installation or launch workflow.

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

Mnelys is at M0, independent setup and technical validation, restarted after the stack change:

- [x] Independent repository, license, ADRs, and security baseline
- [x] Tauri v2 runtime and UI baseline established ([ADR 0005](docs/adr/0005-runtime-and-ui-baseline-tauri.md))
- [x] .NET/Avalonia implementation removed (it remains in this repository's Git history)
- [ ] Frontend framework decision (ADR 0006, pending)
- [ ] Rust core skeleton and Tauri command boundary
- [ ] Windows, macOS, and Linux packaging proof of concept through the Tauri bundler
- [ ] First end-to-end Vanilla slice

The packaging evidence from the earlier Avalonia implementation is recorded in the [M0 packaging PoC report](docs/M0_PACKAGING_POC.md). It is a historical record and is **not** evidence for the Tauri baseline.

## Platform scope

| Platform | 1.0 target | Artifact |
|---|---:|---|
| Windows 10/11 x64 | Tier A | NSIS installer and portable executable |
| macOS 13+ Apple Silicon | Tier A | Signed and notarized DMG |
| Linux x86_64 | Tier A | AppImage, with `.deb` alongside |
| macOS Intel/x64 | Unsupported | No artifact |
| Windows ARM64 / Linux ARM64 | Deferred | To be evaluated after 1.0 |

## Technology

- **Tauri 2.x** as the desktop shell and runtime ([ADR 0005](docs/adr/0005-runtime-and-ui-baseline-tauri.md))
- **Rust** for the core: accounts, instances, Java management, downloads, cache, launch pipeline, and IPC
- Platform webviews: WebView2 on Windows, WKWebView on macOS, WebKitGTK on Linux
- The frontend framework is **not yet decided** (ADR 0006); no production UI code is committed before then
- The Rust toolchain is pinned in `rust-toolchain.toml`; Node.js and the package manager are pinned in the repository
- The root window stays opaque: Mica, Acrylic, macOS vibrancy, and desktop sampling are prohibited
- The frontend reaches native capability only through explicitly declared Tauri commands; capabilities are declared per window and reviewed

Dependencies point inward: `presentation → application → domain`. Infrastructure, provider, and platform crates implement ports defined by the inner layers. Tauri types stay inside the command and adapter boundary. See the [architecture baseline](docs/ARCHITECTURE.md).

## Run locally

Prerequisites (since 2026-09-30):

- Rust stable toolchain, pinned by `rust-toolchain.toml`
- Node.js and the package manager pinned in the repository
- Platform build dependencies: WebView2 and MSVC build tools on Windows; Xcode command line tools on macOS; WebKitGTK, `libayatana-appindicator`, and similar Tauri system packages on Linux
- Git 2.40+

```powershell
git clone https://github.com/sdf123098/Mnelys.git
cd Mnelys
```

There is no runnable Tauri project on `main` yet; the scaffold lands once ADR 0006 fixes the frontend framework. At that point local development and builds are `npm run tauri dev` and `npm run tauri build`, with exact script names taken from the `package.json` that lands with the frontend project.

## Packaging

The three platform artifacts come from the Tauri bundler rather than custom scripts:

| Platform | Bundle target | Artifact |
|---|---|---|
| Windows x64 | `nsis` | Installer and portable executable |
| macOS arm64 | `dmg` | Signed and notarized DMG |
| Linux x86_64 | `appimage`, `deb` | AppImage (primary) and `.deb` |

```bash
npm run tauri build -- --bundles nsis
npm run tauri build -- --bundles dmg
npm run tauri build -- --bundles appimage,deb
```

See the [packaging guide](packaging/README.md) for target-host tools, signing requirements, and current limitations.

## Repository layout

```text
Mnelys/
├─ assets/                 Canonical branding assets
├─ docs/                   Plans, architecture, security, support matrix, and ADRs
├─ packaging/              Three-platform packaging notes (artifacts come from the Tauri bundler)
└─ (pending)               `src-tauri/` and the frontend project, created after ADR 0006
```

## Contributing and clean-room boundary

Read [CONTRIBUTING.md](CONTRIBUTING.md) before submitting changes. Do not copy or adapt source code, assets, translations, credentials, CI files, packaging scripts, or internal models from Prism Launcher, MultiMC, HMCL, PCL2, or other launchers.

Follow [SECURITY.md](docs/SECURITY.md) when reporting a vulnerability. Never place credentials or unredacted personal data in a public issue.

## License and disclaimer

Source code is released under the [MIT License](LICENSE).

Mnelys is not an official Mojang Studios or Microsoft product and is not endorsed by or affiliated with either company. Minecraft is a trademark of its respective owners. Original authorship and public redistribution terms for the branding artwork must be recorded before the first public release.
