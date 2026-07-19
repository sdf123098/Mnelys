# Support matrix

Status: M0 baseline
Last updated: 2026-07-19

## Platforms

| Platform | 1.0 tier | Planned artifact | M0 status |
|---|---:|---|---|
| Windows 10/11 x64 | A | Self-contained single `Mnelys.exe` | EXE built and locally smoke-tested |
| macOS 13+ arm64 | A | Signed and notarized DMG | Payload and `.app` layout built; macOS validation pending |
| macOS 13+ x64 | D | No 1.0 artifact | Out of scope |
| Linux x86_64 | A | AppImage | Payload and AppDir layout built; Linux validation pending |
| Windows arm64 | D | Not committed for 1.0 | Deferred |
| Linux arm64 | D | Not committed for 1.0 | Deferred |

## Runtime and UI

| Component | Baseline | Policy |
|---|---|---|
| .NET SDK | 10.0.302 | Accept later 10.0.3xx patches only |
| Target framework | `net10.0` | LTS baseline |
| C# | 14.0 | Nullable and implicit usings enabled |
| Avalonia | 12.1.0 | Stable 12.x patches/minors require tested dependency update |
| Bindings | Compiled | Required by default |

## Minecraft capability tiers

The detailed loader/version compatibility matrix begins with the Vanilla vertical slice. Until a combination has a fixture, automated contract coverage, and platform smoke evidence, it is not marked supported.

Tier meanings:

- Tier A: complete CI and manual representative matrix.
- Tier B: supported with documented version or platform restrictions.
- Tier C: experimental and may require advanced settings.
- Tier D: recognized or reserved without a runtime guarantee.
