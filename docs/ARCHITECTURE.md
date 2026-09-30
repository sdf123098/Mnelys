# Architecture baseline

Status: Accepted for M0 (Tauri v2 baseline, see [ADR 0005](adr/0005-runtime-and-ui-baseline-tauri.md))
Last updated: 2026-09-30

## Dependency direction

The Rust core is layered and depends inward only:

```text
mnelys-desktop
  └─ mnelys-presentation
       └─ mnelys-application
            └─ mnelys-domain

mnelys-infrastructure ───────┐
mnelys-platform-* ───────────┼─ implement inward-facing ports
mnelys-provider-* ───────────┘
```

`mnelys-desktop` is the Tauri host: it owns window and tray setup, the command surface exposed to the webview, updater glue, and nothing else of substance. The webview frontend is an outer adapter that may call only the commands the backend explicitly exposes; it never reaches the filesystem, shell, or network on its own.

`mnelys-domain` has no dependency on Tauri, HTTP, SQLite, or operating-system APIs. `mnelys-application` owns use-case orchestration. `mnelys-presentation` owns view models, navigation state, and design-system integration but never handles credentials or performs high-risk I/O directly.

## Process boundaries

The Tauri main process hosts the webview, the Rust application and domain services, ordinary background tasks, and the local IPC client. Java/Minecraft, loader processors, Terracotta/EasyTier, the Developer Daemon, and the update helper execute out of process when their milestone is implemented.

## Provider model

Providers are explicitly registered at compile time and implement capability-specific ports. Arbitrary in-process binary plugins are out of scope. A future third-party extension mechanism must use a restricted out-of-process protocol and requires a separate security decision.

## Cross-cutting rules

- I/O is asynchronous and cancellable.
- Contracts crossing the Rust/webview command boundary are typed and versioned; untyped JSON blobs are not an interface.
- Services are registered explicitly; runtime scanning and dynamic discovery are prohibited.
- Persistent formats carry schema versions and support rollback-aware migration.
- Long operations report immutable task snapshots and never block the UI thread.
- Download mirrors may change transport location but never the original trust hash.
- Webview capabilities are declared per window and reviewed; a new capability requires a stated justification in the pull request.

Detailed decisions are recorded in [ADR](adr/README.md).
