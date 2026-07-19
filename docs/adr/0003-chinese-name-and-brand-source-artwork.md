# ADR 0003: Chinese product name and brand source artwork

- Status: Accepted
- Date: 2026-07-19
- Decision owners: Mnelys maintainers

## Context

The project needs an official Chinese display name and a canonical source image from which platform icon packages can be generated without repeatedly transforming an already compressed derivative.

## Decision

- The international product name remains `Mnelys`.
- The Simplified Chinese product name is `忆涟`.
- `assets/branding/mnelys-icon-source.jpg` is the canonical project icon source supplied by the project owner.
- The canonical JPEG is retained byte-for-byte. Platform-specific PNG, ICO, and ICNS files are generated as derived build assets and do not replace it.
- UI strings use the localized Chinese name where locale-specific display names are supported. Package IDs, namespaces, executable names, and command names remain ASCII `Mnelys` forms unless a platform requires otherwise.

## Consequences

Packaging work must create and visually inspect multi-size derivatives for each platform. Small sizes may need a separately approved simplified crop because automatic downscaling of the full 690×690 illustration can lose facial and eye detail.

The original creator and redistribution terms must be captured in the brand asset record before the first public release.
