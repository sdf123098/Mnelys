# Support matrix

Status: M6 core baseline (Tauri v2)
Last updated: 2026-10-26

## Platforms

| Platform          | 1.0 tier | Planned artifact                              | Current status                                |
| ----------------- | -------: | --------------------------------------------- | --------------------------------------------- |
| Windows 10/11 x64 |        A | NSIS installer plus a portable executable     | Implemented; CI build pending remote evidence |
| macOS 13+ arm64   |        A | Signed and notarized DMG                      | Implemented; CI build pending remote evidence |
| macOS 13+ x64     |        D | No 1.0 artifact                               | Out of scope                                  |
| Linux x86_64      |        A | AppImage, with `.deb` as a secondary artifact | Implemented; CI build pending remote evidence |
| Windows arm64     |        D | Not committed for 1.0                         | Deferred                                      |
| Linux arm64       |        D | Not committed for 1.0                         | Deferred                                      |

Artifacts produced by the `Desktop packages` workflow are unsigned test builds. They are not official releases, and macOS signing/notarization still requires credentials supplied outside the repository.

## Runtime and UI

| Component                  | Baseline                                                                                        | Policy                                                                                                                                               |
| -------------------------- | ----------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| Desktop shell              | Tauri 2.x                                                                                       | Minor/major upgrades require a focused ADR and three-platform regression evidence                                                                    |
| Core language              | Rust, stable toolchain pinned in `rust-toolchain.toml`                                          | Toolchain bumps follow the dependency update process with a full test run                                                                            |
| Frontend framework         | React 19 with TypeScript ([ADR 0006](adr/0006-frontend-framework-and-application-stack.md))     | Framework or language changes require a focused ADR and a full frontend test run                                                                     |
| Frontend build and routing | Vite, `createHashRouter`                                                                        | Hash routing is required because packaged builds run under a custom protocol where unknown paths are not rewritten to `index.html` on every platform |
| Frontend state             | TanStack Query for server state, Zustand for transient state                                    | `invoke` may only be called from `src/ipc/`                                                                                                          |
| Localization               | i18next with 10 bundled catalogs ([ADR 0007](adr/0007-accent-color-and-localization-policy.md)) | Every catalog must have the exact key set of `en`; `src/i18n/catalogs.test.ts` enforces this; adding a language is adding one JSON file              |
| Bundled languages          | `en`, `zh-Hans`, `zh-Hant`, `ja`, `ko`, `fr`, `de`, `es`, `pt-BR`, `ru`                         | Core four are listed first in the picker; additional catalogs are ordered by code                                                                    |
| Product naming             | One localized name per catalog                                                                  | `忆涟` in Simplified Chinese, `憶漣` in Traditional Chinese, `Mnelys` elsewhere (including Japanese); a second name is a test failure                |
| Accent colour              | Brand pink `#C2185B` by default, with presets and a custom hex                                  | Custom colours are expanded into five tokens; lightness may move to reach WCAG AA, hue and saturation do not                                         |
| Windows webview            | WebView2                                                                                        | Follows the evergreen runtime shipped with current Windows 10/11                                                                                     |
| macOS webview              | WKWebView                                                                                       | macOS 13+ system webview                                                                                                                             |
| Linux webview              | WebKitGTK                                                                                       | The version shipped by the oldest supported distribution is the compatibility floor                                                                  |
| Window material            | Opaque                                                                                          | Mica, Acrylic, macOS vibrancy, desktop sampling, and undocumented blur are prohibited                                                                |
| Window frame               | Drawn title bar; macOS native traffic lights                                                    | Windows/Linux draw minimize/maximize/close; macOS keeps native controls and reserves their space                                                     |
| Packaging                  | Tauri bundler                                                                                   | NSIS, DMG, AppImage are produced by `tauri build` rather than custom scripts                                                                         |

## Minecraft capability tiers

Tier meanings:

- Tier A: complete CI and manual representative matrix.
- Tier B: supported with documented version or platform restrictions.
- Tier C: experimental and may require advanced settings.
- Tier D: recognized or reserved without a runtime guarantee.

| Capability                  | Tier | Current state                                                                                       |
| --------------------------- | ---: | --------------------------------------------------------------------------------------------------- |
| Vanilla install and launch  |    B | Implemented end to end; real-desktop smoke evidence still pending                                   |
| Offline accounts            |    B | Implemented with Java-compatible MD5 v3 UUIDs                                                       |
| Microsoft device-code login |    C | Implemented; requires `MNELYS_MS_CLIENT_ID` and a real account for end-to-end evidence              |
| Yggdrasil accounts          |    B | Authenticate, refresh, and validate implemented                                                     |
| Quilt / Legacy Fabric       |    B | Metadata profiles resolved from their official meta APIs                                            |
| Fabric                      |    B | Metadata profiles resolved from the official meta API                                               |
| Forge / NeoForge installers |    C | Installer JAR and processor pipeline implemented; version coverage is not yet established           |
| Modrinth content            |    B | Search, versions, install, list, and remove implemented                                             |
| CurseForge content          |    C | Implemented; requires `MNELYS_CURSEFORGE_API_KEY` and real-network evidence                         |
| Content kinds               |    B | Mods, resource packs, shaders, worlds, and datapacks                                                |
| Mirrors                     |    B | Direct / BMCLAPI / auto; transport may change, upstream SHA-1 never does                            |
| Instance export / import    |    B | JSON manifest only (2 MiB limit); worlds and user files are not included                            |
| Java discovery and install  |    B | JAVA_HOME, managed root, platform roots, PATH; Temurin fetched from Adoptium with hash verification |
| Multiplayer (Terracotta)    |    D | Not implemented                                                                                     |
| Developer daemon / Gradle   |    D | Not implemented                                                                                     |
| AI crash diagnostics        |    D | Not implemented                                                                                     |

A capability is promoted only after it has fixtures, automated contract coverage, and platform smoke evidence. Implementation alone does not make a row Tier A.
