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

- `mnelys-domain` holds stable vocabulary and rules: accounts, instances, versions, loaders, launch plans, content, tasks, Java and download identities. It has no dependency on Tauri, HTTP, SQLite, or operating-system APIs.
- `mnelys-application` owns use-cases and the ports they need: settings, metadata, installation, launch, accounts, content, tasks, mirrors, paths and Java. `LogService` and the `LogStore` port were the first real example; the same pattern now covers HTTP, files, secrets, Java, processes and loader providers.
- `mnelys-infrastructure` implements those ports: Mojang/BMCLAPI HTTP and caching, Fabric-family metadata, Forge/NeoForge installer pipelines, Modrinth and CurseForge providers, Adoptium Java installation, process supervision, keyring storage and the on-disk instance layout.
- `mnelys-presentation` owns the Tauri command surface, the generated handler and the state commands receive. It translates transport requests into application calls and contains no business rules.
- `src-tauri` is the composition root: it chooses adapters, installs state, configures the window and starts the runtime. Its own source stays limited to wiring.

Providers and platform adapters live as sibling modules inside infrastructure today. They implement ports from `mnelys-application` and are split into separate crates only when a capability needs its own dependency budget or platform build; the dependency direction does not change either way.

The webview frontend is an outer adapter that may call only the commands the backend explicitly exposes; it never reaches the filesystem, shell, or network on its own. View models, routing and design-system integration live in `src/`, not in `mnelys-presentation`.

## Process boundaries

The Tauri main process hosts the webview, the Rust application and domain services, ordinary background tasks, and the local IPC client. Minecraft and Java subprocesses are started and observed by the core: stdout/stderr are captured per launch session, and stopping a session targets that child process. Loader installer processors also run as child Java processes under the same supervision.

Terracotta/EasyTier, the Developer Daemon, and the update helper will execute out of process when their milestone is implemented.

## Provider model

Providers are explicitly registered at compile time and implement capability-specific ports: mirror policy, version metadata, loader metadata and installation, Java distribution, and content search. Modrinth and CurseForge are the two content providers; Fabric, Quilt, Legacy Fabric, Forge and NeoForge are the loader providers. Arbitrary in-process binary plugins are out of scope. A future third-party extension mechanism must use a restricted out-of-process protocol and requires a separate security decision.

## Data layout

The data root is `MNELYS_HOME` when set, otherwise the platform data directory (`%APPDATA%\Mnelys`, `~/Library/Application Support/Mnelys`, `~/.local/share/Mnelys`). Inside it, the core owns version documents and jars, shared assets, libraries, managed Java runtimes, per-instance directories, settings, account metadata and instance manifests. Secrets are not files in this tree; they live in the platform keystore.

## Cross-cutting rules

- I/O is asynchronous and cancellable.
- Contracts crossing the Rust/webview command boundary are typed and versioned; untyped JSON blobs are not an interface.
- Services are registered explicitly; runtime scanning and dynamic discovery are prohibited.
- Persistent formats carry schema versions and support rollback-aware migration.
- Long operations report immutable task snapshots and never block the UI thread.
- Download mirrors may change transport location but never the original trust hash.
- Webview capabilities are declared per window and reviewed; a new capability requires a stated justification in the pull request.

Detailed decisions are recorded in [ADR](adr/README.md).
