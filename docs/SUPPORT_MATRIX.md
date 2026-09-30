# Support matrix

Status: M0 baseline (Tauri v2)
Last updated: 2026-09-30

## Platforms

| Platform          | 1.0 tier | Planned artifact                              | M0 status                         |
| ----------------- | -------: | --------------------------------------------- | --------------------------------- |
| Windows 10/11 x64 |        A | NSIS installer plus a portable executable     | Not started on the Tauri baseline |
| macOS 13+ arm64   |        A | Signed and notarized DMG                      | Not started on the Tauri baseline |
| macOS 13+ x64     |        D | No 1.0 artifact                               | Out of scope                      |
| Linux x86_64      |        A | AppImage, with `.deb` as a secondary artifact | Not started on the Tauri baseline |
| Windows arm64     |        D | Not committed for 1.0                         | Deferred                          |
| Linux arm64       |        D | Not committed for 1.0                         | Deferred                          |

## Runtime and UI

| Component                  | Baseline                                                                                            | Policy                                                                                                                                               |
| -------------------------- | --------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| Desktop shell              | Tauri 2.x                                                                                           | Minor/major upgrades require a focused ADR and three-platform regression evidence                                                                    |
| Core language              | Rust, stable toolchain pinned in `rust-toolchain.toml`                                              | Toolchain bumps follow the dependency update process with a full test run                                                                            |
| Frontend framework         | React 19 with TypeScript ([ADR 0006](../docs/adr/0006-frontend-framework-and-application-stack.md)) | Framework or language changes require a focused ADR and a full frontend test run                                                                     |
| Frontend build and routing | Vite, `createHashRouter`                                                                            | Hash routing is required because packaged builds run under a custom protocol where unknown paths are not rewritten to `index.html` on every platform |
| Frontend state             | TanStack Query for server state, Zustand for transient state                                        | `invoke` may only be called from `src/ipc/`                                                                                                          |
| Localization               | i18next with `zh-Hans`, `en`, `ja`                                                                  | All three catalogs must have identical key sets; `src/i18n/catalogs.test.ts` enforces this                                                           |
| Windows webview            | WebView2                                                                                            | Follows the evergreen runtime shipped with current Windows 10/11                                                                                     |
| macOS webview              | WKWebView                                                                                           | macOS 13+ system webview                                                                                                                             |
| Linux webview              | WebKitGTK                                                                                           | The version shipped by the oldest supported distribution is the compatibility floor                                                                  |
| Window material            | Opaque                                                                                              | Mica, Acrylic, macOS vibrancy, desktop sampling, and undocumented blur are prohibited                                                                |
| Packaging                  | Tauri bundler                                                                                       | NSIS, DMG, AppImage are produced by `tauri build` rather than custom scripts                                                                         |

## Minecraft capability tiers

The detailed loader/version compatibility matrix begins with the Vanilla vertical slice. Until a combination has a fixture, automated contract coverage, and platform smoke evidence, it is not marked supported.

Tier meanings:

- Tier A: complete CI and manual representative matrix.
- Tier B: supported with documented version or platform restrictions.
- Tier C: experimental and may require advanced settings.
- Tier D: recognized or reserved without a runtime guarantee.
