<p align="center">
  <img src="assets/branding/mnelys-icon-source.jpg" width="220" alt="Mnelys icon">
</p>

<h1 align="center">Mnelys</h1>

<p align="center">
  An AI-native Minecraft: Java Edition launcher and development workspace for players and mod developers
</p>

<p align="center">
  <a href="README.md">简体中文</a> · English · <a href="README.ja.md">日本語</a>
</p>

> [!IMPORTANT]
> Mnelys uses **Tauri v2 + React 19** (see [ADR 0005](docs/adr/0005-runtime-and-ui-baseline-tauri.md) and [ADR 0006](docs/adr/0006-frontend-framework-and-application-stack.md)). `main` now implements the M1–M6 core: accounts, instances, Java, metadata and loader installation, content platforms, and the launch pipeline. Real-desktop launch smoke, three-platform packaging evidence, and signing/notarization are still open. CI artifacts are **unsigned test builds, not official releases**.

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

The M1–M6 core is implemented; real-desktop evidence and release artifacts are still open:

- [x] Independent repository, license, ADRs, and security baseline
- [x] Tauri v2 runtime and UI baseline ([ADR 0005](docs/adr/0005-runtime-and-ui-baseline-tauri.md))
- [x] Frontend framework and stack ([ADR 0006](docs/adr/0006-frontend-framework-and-application-stack.md))
- [x] Design system, light/dark/system themes, default pink, and custom accent colours
- [x] Drawn title bar with native macOS traffic lights; one localized product name
- [x] Single-window shell, responsive navigation, task center, log panel, settings, and developer mode
- [x] Ten catalogs: Simplified Chinese, Traditional Chinese, English, Japanese, Korean, French, German, Spanish, Brazilian Portuguese, Russian
- [x] Interface icons consolidated on Lucide (ISC), see [ICONS.md](docs/ICONS.md)
- [x] Rust Cargo workspace layering and a v3 command contract (39 commands)
- [x] Vanilla install and launch: metadata, hash verification, mirror fallback, Java discovery and install, argument rendering, process and logs
- [x] Offline, Yggdrasil, and Microsoft device-code accounts; keystore storage with tokens kept off IPC
- [x] Fabric / Quilt / Legacy Fabric metadata loaders
- [x] Forge / NeoForge installer pipeline
- [x] Content hub: Modrinth / CurseForge search, installs grouped by resource type, removal, and instance-manifest export/import
- [x] GitHub Actions: frontend checks, a three-platform Rust matrix, and desktop packaging
- [ ] Real-desktop launch to the main menu, DPI, and three-platform visual regression
- [ ] Installing the artifacts on real desktops plus signing and notarization evidence (unsigned sizes are recorded in [PROGRESS](docs/PROGRESS.md))

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
- **Rust** for the core: accounts, instances, Java management, downloads, cache, launch pipeline, and IPC. The core is a Cargo workspace (domain / application / infrastructure / presentation); `src-tauri` is only the composition root
- Platform webviews: WebView2 on Windows, WKWebView on macOS, WebKitGTK on Linux
- The frontend is **React 19 with TypeScript** ([ADR 0006](docs/adr/0006-frontend-framework-and-application-stack.md)), built by Vite, with hash routing through `react-router`, TanStack Query for server state, Zustand for transient state, i18next for UI text, and Lucide for interface icons (ISC, see [ICONS.md](docs/ICONS.md))
- Catalogs are discovered with `import.meta.glob`: adding a language is adding one JSON file, and `src/i18n/catalogs.test.ts` requires every catalog to match the `en` key set exactly. Accent and naming rules are recorded in [ADR 0007](docs/adr/0007-accent-color-and-localization-policy.md)
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

> The MSYS2/MinGW GNU toolchain is not a substitute for MSVC on Windows. `x86_64-pc-windows-gnu` compiles, passes `cargo clippy`, and runs the domain, application, and infrastructure tests, but the `mnelys-presentation` and `src-tauri` test binaries depend on WinRT API sets and cannot load in the local GNU environment. CI on MSVC, Linux, and macOS is the authoritative full test run.

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

## Continuous integration and packaging

- [`CI`](.github/workflows/ci.yml) runs frontend format, lint, type-check, tests, and build, plus `cargo fmt`, `cargo clippy -D warnings`, and `cargo test` on Windows, macOS, and Linux.
- [`Desktop packages`](.github/workflows/desktop-package.yml) runs on demand or on `v*` tags and uploads an NSIS installer, a macOS arm64 DMG, and an AppImage. The artifacts are unsigned, retained for 14 days, and exist as build evidence; signing and notarization need credentials supplied outside the repository.
- **Downloading test artifacts**: open the repository's Actions → `Desktop packages` → `Run workflow`, then download `mnelys-windows-x64-unsigned`, `mnelys-macos-arm64-unsigned`, or `mnelys-linux-x64-unsigned` from the run's Artifacts section. The Windows artifact contains the NSIS installer and the portable `mnelys.exe`.
- CI is the normal verification path. Local desktop builds are reserved for cases that genuinely need an artifact or interactive debugging.

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
├─ crates/                 Rust core layers: domain / application / infrastructure / presentation
├─ docs/                   Plans, architecture, security, support matrix, and ADRs
├─ packaging/              Three-platform packaging notes (artifacts come from the Tauri bundler)
├─ src/                    React 19 frontend
│  ├─ app/                 Application wiring: providers, router
│  ├─ components/          Reusable components
│  ├─ ipc/                 The only Tauri command and error boundary
│  ├─ i18n/                Ten catalogs (auto-discovered)
│  ├─ routes/              Page-level components
│  ├─ state/               Cross-page transient state
│  ├─ styles/              Tokens and global styles
│  └─ test/                Test setup
├─ src-tauri/              Tauri v2 shell, window configuration, and composition root
│  ├─ capabilities/        Per-window capability sets
│  ├─ icons/               Application icons
│  ├─ src/                 Composition root: installs state, mounts commands, starts the runtime
│  └─ tests/               Cross-boundary command contract tests
├─ Cargo.toml / Cargo.lock  Workspace manifest and dependency lock
├─ package.json / pnpm-lock.yaml
├─ rust-toolchain.toml
├─ tsconfig.json / vite.config.ts / eslint.config.js
└─ .prettierrc.json
```

The Rust core is a Cargo workspace: `mnelys-domain` (failure vocabulary), `mnelys-application` (use-cases and ports), `mnelys-infrastructure` (adapters), and `mnelys-presentation` (Tauri commands and handler); `src-tauri` only composes them. Provider and platform crates will split out when the capabilities that need them exist.

## Contributing and clean-room boundary

Read [CONTRIBUTING.md](CONTRIBUTING.md) before submitting changes. Do not copy or adapt source code, assets, translations, credentials, CI files, packaging scripts, or internal models from Prism Launcher, MultiMC, HMCL, PCL2, or other launchers.

Follow [SECURITY.md](docs/SECURITY.md) when reporting a vulnerability. Never place credentials or unredacted personal data in a public issue.

## License and disclaimer

Source code is released under the [MIT License](LICENSE).

Mnelys is not an official Mojang Studios or Microsoft product and is not endorsed by or affiliated with either company. Minecraft is a trademark of its respective owners. Original authorship and public redistribution terms for the branding artwork must be recorded before the first public release.
