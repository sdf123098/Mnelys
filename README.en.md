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
> Mnelys has moved its technical baseline to **Tauri v2 + React 19** (see [ADR 0005](docs/adr/0005-runtime-and-ui-baseline-tauri.md) and [ADR 0006](docs/adr/0006-frontend-framework-and-application-stack.md)). `main` now contains both the Rust core skeleton and the React frontend project and **can be run locally**, but there is still no Minecraft installation or launch workflow and no three-platform artifact yet.

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
- [x] Frontend framework and stack decided ([ADR 0006](docs/adr/0006-frontend-framework-and-application-stack.md))
- [x] Rust core skeleton, command boundary, and cross-boundary contract tests
- [x] React frontend project, three-language i18n, and component tests
- [ ] Windows, macOS, and Linux packaging proof of concept through the Tauri bundler
- [ ] First end-to-end Vanilla slice

The packaging evidence from the earlier Avalonia implementation is recorded in the [M0 packaging PoC report](docs/M0_PACKAGING_POC.md). It is a historical record and is **not** evidence for the Tauri baseline.

## Platform scope

| Platform                    |  1.0 target | Artifact                               |
| --------------------------- | ----------: | -------------------------------------- |
| Windows 10/11 x64           |      Tier A | NSIS installer and portable executable |
| macOS 13+ Apple Silicon     |      Tier A | Signed and notarized DMG               |
| Linux x86_64                |      Tier A | AppImage, with `.deb` alongside        |
| macOS Intel/x64             | Unsupported | No artifact                            |
| Windows ARM64 / Linux ARM64 |    Deferred | To be evaluated after 1.0              |

## Technology

- **Tauri 2.x** as the desktop shell and runtime ([ADR 0005](docs/adr/0005-runtime-and-ui-baseline-tauri.md))
- **Rust** for the core: accounts, instances, Java management, downloads, cache, launch pipeline, and IPC
- Platform webviews: WebView2 on Windows, WKWebView on macOS, WebKitGTK on Linux
- The frontend is **React 19 with TypeScript** ([ADR 0006](docs/adr/0006-frontend-framework-and-application-stack.md)), built by Vite, with hash routing through `react-router`, TanStack Query for server state, Zustand for transient state, and i18next for UI text
- The Rust toolchain is pinned in `rust-toolchain.toml`; Node.js and pnpm are pinned in `package.json`
- The root window stays opaque: Mica, Acrylic, macOS vibrancy, and desktop sampling are prohibited; a global `backdrop-filter: none !important` rule is the fallback
- The frontend reaches native capability only through explicitly declared Tauri commands, and `invoke` may only appear inside `src/ipc/`; capabilities are declared per window and reviewed

Dependencies point inward: `presentation → application → domain`. Infrastructure, provider, and platform crates implement ports defined by the inner layers. Tauri types stay inside the command and adapter boundary. See the [architecture baseline](docs/ARCHITECTURE.md).

## Run locally

Prerequisites (since 2026-09-30):

- Rust stable toolchain, pinned by `rust-toolchain.toml`
- Node.js 24+ and pnpm 11.7.0 (pinned by the `packageManager` field; corepack is the recommended way to enable it)
- Platform build dependencies: the WebView2 runtime and MSVC build tools on Windows (the "Desktop development with C++" workload of Visual Studio Build Tools, which provides `link.exe`); Xcode command line tools on macOS; WebKitGTK, `libayatana-appindicator`, and similar Tauri system packages on Linux
- Git 2.40+

> The MSYS2/MinGW GNU toolchain is not a substitute for MSVC on Windows. `x86_64-pc-windows-gnu` compiles and passes `cargo clippy`, but test binaries fail to load with `0xC0000139` (`STATUS_ENTRYPOINT_NOT_FOUND`): Rust's GNU target is msvcrt-based while MSYS2's UCRT64 toolchain is UCRT-based.

```powershell
git clone https://github.com/sdf123098/Mnelys.git
cd Mnelys

pnpm install                                       # frontend dependencies
cargo fetch --manifest-path src-tauri/Cargo.toml   # Rust dependencies

pnpm tauri:dev                                     # launch the desktop app (starts the Vite dev server too)
```

Scripts in `package.json`:

| Script                                | Purpose                                        |
| ------------------------------------- | ---------------------------------------------- |
| `pnpm dev`                            | Vite only, for working on the UI in a browser  |
| `pnpm build`                          | Type-check and build the frontend into `dist/` |
| `pnpm typecheck`                      | `tsc --noEmit`                                 |
| `pnpm lint`                           | ESLint                                         |
| `pnpm format:check`                   | Prettier check                                 |
| `pnpm test`                           | Vitest with Testing Library                    |
| `pnpm tauri:dev` / `pnpm tauri:build` | Tauri development and packaging                |

> `src-tauri/` validates the frontend output at compile time. To run `cargo check`, `cargo test`, or `cargo clippy` on their own, you must run `pnpm build` first, or `tauri::generate_context!` fails because `dist/` is missing.

## Packaging

The three platform artifacts come from the Tauri bundler rather than custom scripts:

| Platform     | Bundle target     | Artifact                          |
| ------------ | ----------------- | --------------------------------- |
| Windows x64  | `nsis`            | Installer and portable executable |
| macOS arm64  | `dmg`             | Signed and notarized DMG          |
| Linux x86_64 | `appimage`, `deb` | AppImage (primary) and `.deb`     |

```bash
pnpm tauri:build -- --bundles nsis
pnpm tauri:build -- --bundles dmg
pnpm tauri:build -- --bundles appimage,deb
```

Windows targets `x86_64-pc-windows-msvc` explicitly; macOS targets `aarch64-apple-darwin`.

See the [packaging guide](packaging/README.md) for target-host tools, signing requirements, and current limitations.

## Repository layout

```text
Mnelys/
├─ assets/                 Canonical branding assets
├─ docs/                   Plans, architecture, security, support matrix, and ADRs
├─ packaging/              Three-platform packaging notes (artifacts come from the Tauri bundler)
├─ src/                    React 19 frontend
│  ├─ app/                 Application wiring: providers, router
│  ├─ components/          Reusable components
│  ├─ ipc/                 The only Tauri command and error boundary
│  ├─ i18n/                zh-Hans / en / ja catalogs
│  ├─ routes/              Page-level components
│  ├─ state/               Cross-page transient state
│  ├─ styles/              Tokens and global styles
│  └─ test/                Test setup
├─ src-tauri/              Tauri v2 shell and Rust core
│  ├─ capabilities/        Per-window capability sets
│  ├─ icons/               Application icons
│  ├─ src/                 commands, error, and the `run()` entry point
│  └─ tests/               Cross-boundary command contract tests
├─ package.json / pnpm-lock.yaml
├─ rust-toolchain.toml
├─ tsconfig.json / vite.config.ts / eslint.config.js
└─ .prettierrc.json
```

The `crates/` split (domain / application / presentation / infrastructure / providers / platform) is planned for M1; during M0 the Rust code stays inside `src-tauri/src/`.

## Contributing and clean-room boundary

Read [CONTRIBUTING.md](CONTRIBUTING.md) before submitting changes. Do not copy or adapt source code, assets, translations, credentials, CI files, packaging scripts, or internal models from Prism Launcher, MultiMC, HMCL, PCL2, or other launchers.

Follow [SECURITY.md](docs/SECURITY.md) when reporting a vulnerability. Never place credentials or unredacted personal data in a public issue.

## License and disclaimer

Source code is released under the [MIT License](LICENSE).

Mnelys is not an official Mojang Studios or Microsoft product and is not endorsed by or affiliated with either company. Minecraft is a trademark of its respective owners. Original authorship and public redistribution terms for the branding artwork must be recorded before the first public release.
