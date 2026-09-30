# Contributing to Mnelys

## Clean-room boundary

Mnelys is an independent implementation. Contributions must not copy or adapt source code, resources, translations, API keys, CI configuration, packaging scripts, internal data models, or other protected implementation details from Prism Launcher, MultiMC, HMCL, PCL2, or other launchers.

Behavioral compatibility may be studied through public documentation, public protocols and file formats, black-box testing, and independently produced fixtures. Record non-obvious external sources in the pull request or adjacent documentation.

If a contribution intentionally incorporates third-party code, stop before submission and document its origin, exact license, required notices, and compatibility with the repository license.

## Engineering baseline

- Keep the Rust core directed inward: `presentation → application → domain`, with infrastructure, provider, and platform crates implementing inward-facing ports. Tauri types stay inside the command and adapter boundary.
- Do not place credentials, authentication calls, archive extraction, installer execution, or launch-command construction in frontend components or state stores.
- Treat the Tauri capability set as security surface. Add a capability only when a named command requires it, and justify it in the pull request.
- Make asynchronous APIs cancellable and keep blocking work off the UI thread.
- Use semantic design tokens; do not enable Mica, Acrylic, macOS vibrancy, desktop sampling, undocumented DWM blur, or transparent window composition.
- Add or update tests for behavior changes and keep logs free of secrets.

## Frontend conventions

- The frontend is React 19 with TypeScript in strict mode, built by Vite. Keep components typed; do not introduce `any` to silence the compiler.
- `invoke` and any other `@tauri-apps/api` import may only appear inside `src/ipc/`. Everything else consumes the typed wrappers from there, which return `CoreError`-shaped failures rather than raw rejections.
- Server state belongs in TanStack Query; transient cross-page state belongs in Zustand. Components do not cache backend data in `useState`.
- Any user-visible string must go through i18next. `zh-Hans`, `en`, and `ja` catalogs must have identical key sets; `src/i18n/catalogs.test.ts` fails the build otherwise.
- Routing uses `createHashRouter`. Do not switch to browser history routing: packaged builds run under a custom protocol where unknown paths are not rewritten to `index.html` on every platform.

## Local checks

Run these before opening a pull request. On Windows the Rust checks need the MSVC toolchain: `x86_64-pc-windows-gnu` compiles and passes `clippy`, but its test binaries fail to load with `0xC0000139` because Rust's GNU target is msvcrt-based while MSYS2's UCRT64 toolchain is UCRT-based.

```bash
pnpm format:check
pnpm lint
pnpm typecheck
pnpm test
pnpm build                     # required before any cargo command, so src-tauri can find dist/
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

Warnings are errors in CI. `src-tauri/tests/contract.rs` reads the TypeScript command and error definitions directly; if you change `src/ipc/commands.ts` or `src/ipc/errors.ts`, update the Rust side in the same commit or that test will fail.
