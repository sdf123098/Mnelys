# ADR 0005: Runtime and UI baseline — Tauri v2

- Status: Accepted
- Date: 2026-09-30
- Decision owners: Mnelys maintainers
- Supersedes: [ADR 0002](0002-runtime-and-ui-baseline.md)

## Context

ADR 0002 selected .NET 10 with Avalonia 12 as the desktop runtime and UI framework. Two implementations were produced against it: an earlier Prism Launcher derived C++ client, and a clean-room .NET/Avalonia shell that reached M0 with three-platform packaging proofs of concept.

The project owner has changed the delivery direction. Mnelys will be rebuilt on a web-technology desktop shell so that one UI implementation is shared across all three platforms and the team can move faster on the large amount of ordinary application UI this product needs (instance lists, logs, task center, download views, developer panels). The existing source is discarded rather than migrated; only the abandoned working copies were removed, and the Avalonia implementation remains permanently recoverable from this repository's Git history.

Project identity is unchanged. [ADR 0001](0001-project-identity-and-clean-room-boundary.md) (clean-room boundary), [ADR 0003](0003-chinese-name-and-brand-source-artwork.md) (Chinese name and brand artwork), and [ADR 0004](0004-macos-arm64-only.md) (macOS Apple Silicon only) remain in force. Only the runtime and UI baseline of ADR 0002 is replaced.

## Decision

- Use **Tauri 2.x** as the desktop application shell and runtime.
- Implement the core — account, instance metadata, Java management, download, cache, launch pipeline, IPC, and provider ports — in **Rust** in the Tauri backend. The core is where correctness-critical and security-sensitive work lives.
- Pin the Rust toolchain in `rust-toolchain.toml`, and pin the Node.js version and package manager in the repository, so that both halves of the build are reproducible.
- Render the UI in the platform webview: **WebView2** on Windows, **WKWebView** on macOS, **WebKitGTK** on Linux. Treat the oldest webview shipped by the declared platform floor as the compatibility baseline, not the newest engine on the development machine.
- Keep the root window **opaque**. Prohibited: Windows Mica, Acrylic, desktop wallpaper sampling, undocumented DWM blur, macOS vibrancy and `NSVisualEffectView`, and any transparent window or `backdrop-filter` composition that samples the desktop. This carries forward the ADR 0002/0003 product rule into the webview world, where the equivalent APIs are `transparent`, `decorations`, and macOS private window APIs.
- Deliver through the Tauri bundler: **NSIS** installer plus a portable executable on Windows x64, a signed and notarized **DMG** on macOS arm64, and **AppImage** (with `.deb` as a secondary artifact) on Linux x86_64.
- Enforce capability isolation. The frontend reaches native functionality only through explicitly declared Tauri commands and a reviewed capability set. Frontend-side filesystem, shell, and arbitrary HTTP access stay disabled unless a named command requires it.
- Credentials and tokens are held only in the Rust core and platform secure storage (Windows Credential Manager, macOS Keychain, libsecret). The webview never receives a refresh token or a raw credential.
- Keep dependencies directed inward. The Rust core is layered and must not depend on Tauri types outside the command and adapter boundary.
- Upgrade Tauri minor or major versions only with a focused ADR and three-platform regression evidence.

## Frontend framework

The frontend framework is deliberately **not fixed by this ADR**. Framework, language, state management, routing, and the design-token pipeline require their own decision (ADR 0006) because they constrain hiring, build tooling, and the design system in ways this ADR does not examine.

Until ADR 0006 is accepted, no production UI code is committed. Scaffolding may exist only on a throwaway branch.

## Consequences

- The .NET 10 / C# 14 / Avalonia 12 baseline is removed: `global.json`, `Directory.Build.props`, `Directory.Packages.props`, `Mnelys.slnx`, and `src/Mnelys.Desktop/` are deleted from `main`. Native AOT is no longer a tracked concern.
- Contributors now need a Rust toolchain and a Node.js toolchain instead of a .NET SDK. The documented setup steps and toolchain pins change accordingly.
- The webview is a shared dependency owned by the operating system, so rendering and CSS behavior can differ between platforms. Regression testing must run on all three engines; a change that only looks correct in one webview is not verified.
- Tauri's capability model becomes load-bearing security surface. The allowlist is reviewed as part of the security baseline, not treated as configuration.
- The M0 evidence recorded in [M0 packaging PoC](../M0_PACKAGING_POC.md) measures the Avalonia implementation (including a 50,602,905-byte Windows x64 self-contained EXE). It is retained as a historical record and is **not** evidence for the Tauri stack; the Tauri M0 must produce its own numbers.
- Product surfaces that ADR 0002 constrained for AOT reasons — reflection, runtime scanning, dynamic loading — are now governed by Rust and JavaScript conventions instead. The prohibition on runtime assembly scanning survives in spirit: services and providers are registered explicitly.
