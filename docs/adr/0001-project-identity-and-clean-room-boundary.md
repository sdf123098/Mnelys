# ADR 0001: Project identity and clean-room boundary

- Status: Accepted
- Date: 2026-07-19
- Decision owners: Mnelys maintainers

## Context

Mnelys is a new launcher and developer workspace. Reusing another launcher's repository history or protected implementation would undermine the intended architecture and introduce licensing and provenance risk.

## Decision

- Product name: `Mnelys`
- Application ID: `io.mnelys.launcher`
- Root namespace: `Mnelys`
- Repository license: MIT
- Copyright notice: `Mnelys Contributors`
- This repository has new Git history and no upstream relationship with Prism Launcher.
- Prism Launcher, MultiMC, HMCL, and PCL2 may be used only for black-box behavior research and compatibility tests.
- Their source, resources, translations, credentials, CI, packaging scripts, class names, and internal models must not be copied or adapted into Mnelys.
- Public specifications, official APIs, public file formats, and independently authored fixtures are allowed sources.

## Consequences

All contributions require traceable provenance. Any proposal to incorporate third-party source code requires a new licensing review and a superseding decision before code is copied.

The application ID and license are inexpensive to change before the first public or code release but become compatibility and legal commitments afterward.
