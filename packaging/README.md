# Packaging

Platform artifacts are produced by the **Tauri bundler**, driven from the frontend project's scripts. The custom packaging scripts that targeted `dotnet publish` were removed on 2026-09-30 together with the .NET/Avalonia baseline (see [ADR 0005](../docs/adr/0005-runtime-and-ui-baseline-tauri.md)).

The frontend project and the Tauri shell have landed (see [ADR 0006](../docs/adr/0006-frontend-framework-and-application-stack.md)); the commands below are the scripts defined in `package.json`. The `Desktop packages` workflow has produced unsigned artifacts for all three targets, but the sizes in the M0 packaging report are Avalonia-era and are not reused as Tauri evidence.

## CI packaging

The primary path is the [`Desktop packages`](../.github/workflows/desktop-package.yml) workflow. It runs on `workflow_dispatch` and on `v*` tags, and uploads:

| Artifact                      | Runner       | Contents                                 |
| ----------------------------- | ------------ | ---------------------------------------- |
| `mnelys-windows-x64-unsigned` | Windows      | NSIS installer and portable `mnelys.exe` |
| `mnelys-macos-arm64-unsigned` | macOS arm64  | Unsigned DMG                             |
| `mnelys-linux-x64-unsigned`   | Ubuntu 22.04 | AppImage                                 |

Artifacts are **unsigned test builds**, not releases. They expire after 14 days and are not attached to a GitHub Release. Windows SmartScreen and macOS Gatekeeper will warn; installing them is a deliberate test action. Promotion to a release requires code signing, notarization, and the manual gates listed below.

First recorded run: [`Desktop packages` run 40262368596](https://github.com/sdf123098/Mnelys/actions/runs/40262368596) — all three jobs green. Downloaded artifact zip sizes were 6,782,945 B (Windows x64), 6,696,204 B (macOS arm64) and 81,688,402 B (Linux x64). Nobody has installed or launched these yet; cold-start and runtime behaviour remain unverified.

Latest run at HEAD: [`Desktop packages` run 40269539520](https://github.com/sdf123098/Mnelys/actions/runs/40269539520) on `771a62e` — all three jobs green. Artifact zip sizes: 6,796,357 B (Windows x64), 6,701,920 B (macOS arm64), 81,691,098 B (Linux x64). The Windows artifact was downloaded and inspected (both files carry an `MZ` header) but not installed or launched:

| File                                     | Size (B)  | SHA-256                                                            |
| ---------------------------------------- | --------- | ------------------------------------------------------------------ |
| `bundle/nsis/Mnelys_0.1.0_x64-setup.exe` | 3,140,080 | `84DB3980D09AE53FDA05AC650F03612800DFFFD987A300DBBBB686F762CE0BE9` |
| `mnelys.exe` (portable)                  | 6,827,008 | `05E954F0895AAFEE6A540C18D4D82699143A0817FB6E5B4406B3C22245F3A47D` |

You do not need a local toolchain to obtain them. Open the run in the repository's Actions tab and download the artifact, or use the CLI:

```bash
gh run list --workflow desktop-package.yml
gh run download <run-id> -n mnelys-windows-x64-unsigned
```

Unzip the artifact to get the installer and the portable executable. Each run is tied to a commit SHA, so a test build can always be traced back to its source.

The Rust crates form one workspace rooted at the repository, so the bundler writes to `target/release/bundle/` rather than `src-tauri/target/`.

The [`CI`](../.github/workflows/ci.yml) workflow runs the frontend checks and a Windows/macOS/Linux Rust matrix (`fmt`, `clippy -D warnings`, `cargo test`). Local desktop builds are only needed when a specific artifact or interactive debugging requires them.

## Windows x64

Run on Windows with WebView2 and the MSVC build tools:

```powershell
pnpm tauri:build --target x86_64-pc-windows-msvc -- --bundles nsis
```

`nsis` produces the installer; the compiled binary can also be shipped as a portable executable without the installer wrapper. Authenticode signing requires a code-signing certificate. Unsigned local builds are acceptable for a proof of concept but must not be published.

## macOS arm64

Run on an Apple Silicon Mac with Xcode command-line tools:

```bash
pnpm tauri:build --target aarch64-apple-darwin -- --bundles dmg
```

The CI workflow uses the same target and produces an unsigned DMG. A universal binary is intentionally not built: Intel/x64 is out of scope under ADR 0004.

Without a Developer ID Application identity the build is only ad-hoc signed. A public artifact requires Developer ID signing, Hardened Runtime, notarization, ticket stapling, and Gatekeeper testing. Intel/x64 remains out of scope per [ADR 0004](../docs/adr/0004-macos-arm64-only.md).

## Linux x86_64

Run on the selected old-glibc build image with the Tauri system dependencies installed (`webkit2gtk`, `libayatana-appindicator`, `librsvg`):

```bash
pnpm tauri:build --target x86_64-unknown-linux-gnu -- --bundles appimage,deb
```

Validate the result across the distribution, desktop environment, display-server, GPU, FUSE, and Secret Service matrix before promotion.

## Signing and secrets

Signing identities, notarization credentials, and update signing keys are release-stage secrets. They are never committed to this repository and are supplied through the release environment.

See [the M0 packaging report](../docs/M0_PACKAGING_POC.md) for the superseded Avalonia-baseline evidence. The Tauri baseline must produce its own numbers.
