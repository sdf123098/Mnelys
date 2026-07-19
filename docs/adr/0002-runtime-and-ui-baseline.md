# ADR 0002: Runtime and UI baseline

- Status: Accepted
- Date: 2026-07-19
- Decision owners: Mnelys maintainers

## Context

The product requires a cross-platform desktop UI, self-contained delivery, predictable dependency updates, and an architecture that can be evaluated for Native AOT without making AOT a 1.0 release blocker.

## Decision

- Target .NET 10 LTS and C# 14.
- Pin SDK feature band `10.0.3xx`, beginning with `10.0.302`.
- Use Avalonia `12.1.0` and central package management.
- Enable compiled bindings by default.
- Keep the root window opaque and prohibit Mica, Acrylic, desktop sampling, and undocumented DWM blur.
- Use self-contained/single-file publishing as the reliable baseline; treat Native AOT as an evidence-driven experiment.
- Upgrade Avalonia minor or major versions only with a focused ADR and three-platform regression evidence. Stable patches may use the dependency update process with the same smoke matrix.

## Consequences

Contributors need a compatible .NET 10 SDK. Reflection, runtime scanning, and dynamic loading require explicit justification and AOT tracking. The M0 publishing PoC must prove Windows, macOS, and Linux artifacts independently.
