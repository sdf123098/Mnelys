# Architecture baseline

Status: Accepted for M0
Last updated: 2026-07-19

## Dependency direction

```text
Mnelys.Desktop
  └─ Mnelys.Presentation
       └─ Mnelys.Application
            └─ Mnelys.Domain

Mnelys.Infrastructure ───────┐
Mnelys.Platform.* ───────────┼─ implements inward-facing ports
Mnelys.Providers.* ──────────┘
```

The Domain project has no dependency on Avalonia, HTTP, SQLite, operating-system APIs, or concrete providers. Application owns use-case orchestration. Presentation owns Views, ViewModels, navigation, and design-system integration but never handles credentials or performs high-risk I/O directly.

## Process boundaries

The desktop process hosts the Avalonia UI, application/domain services, ordinary background tasks, and the local IPC client. Java/Minecraft, loader processors, Terracotta/EasyTier, the Developer Daemon, and the update helper execute out of process when their milestone is implemented.

## Provider model

Providers are explicitly registered at compile time and implement capability-specific ports. Arbitrary in-process binary plugins are out of scope. A future third-party extension mechanism must use a restricted out-of-process protocol and requires a separate security decision.

## Cross-cutting rules

- I/O is asynchronous and cancellable.
- JSON contracts use source generation.
- Services are registered explicitly; runtime assembly scanning is prohibited.
- Persistent formats carry schema versions and support rollback-aware migration.
- Long operations report immutable task snapshots and never block the UI thread.
- Download mirrors may change transport location but never the original trust hash.

Detailed decisions are recorded in [ADR](adr/README.md).
