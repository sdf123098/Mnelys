# ADR 0004: macOS release targets Apple Silicon only

- Status: Accepted
- Date: 2026-07-19
- Decision owners: Mnelys maintainers

## Context

The original plan included separate Apple Silicon and Intel macOS artifacts. The project owner has removed Intel/x64 from the required product scope before the first macOS packaging implementation.

## Decision

- macOS 13+ on Apple Silicon (`osx-arm64`) remains a Tier A target.
- Mnelys does not build or publish a macOS Intel/x64 artifact for 1.0.
- Mnelys does not build a Universal DMG.
- macOS Java discovery and download select arm64 distributions and diagnose x64-only installations as incompatible.
- Restoring Intel/x64 support requires a superseding ADR and a dedicated build, signing, packaging, and test matrix.

## Consequences

M0 produces one macOS payload and one DMG path. CI, signing, notarization, update manifests, documentation, and support testing omit `osx-x64`. This reduces macOS release cost while intentionally excluding Intel Macs.
