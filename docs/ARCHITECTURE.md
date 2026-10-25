# Architecture baseline

Status: Accepted (Tauri v2 baseline, see [ADR 0005](adr/0005-runtime-and-ui-baseline-tauri.md))
Last updated: 2026-10-26

## Dependency direction

The Rust core is a Cargo workspace and depends inward only:

```text
src-tauri (composition root, package `mnelys`)
  ├─ mnelys-presentation
  │    └─ mnelys-application
  │         └─ mnelys-domain
  └─ mnelys-infrastructure ── implements ports declared by ──> mnelys-application
```

- `mnelys-domain` holds stable vocabulary and rules. It has no dependency on Tauri, HTTP, SQLite, or operating-system APIs.
- `mnelys-application` owns use-cases and the ports they need. `LogService` and the `LogStore` port are the first real example.
- `mnelys-infrastructure` implements those ports. `MemoryLogStore` is the bounded in-memory adapter used by the M1 shell.
- `mnelys-presentation` owns the Tauri command surface, the generated handler and the state commands receive. It translates transport requests into application calls and contains no business rules.
- `src-tauri` is the composition root: it chooses adapters, installs state, configures the window and starts the runtime. Its own source stays limited to wiring.

Providers and platform adapters do not exist yet. They will be added as sibling crates that implement ports from `mnelys-application`, never as dependencies from the inside out.

The webview frontend is an outer adapter that may call only the commands the backend explicitly exposes; it never reaches the filesystem, shell, or network on its own. View models, routing and design-system integration live in `src/`, not in `mnelys-presentation`.

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
