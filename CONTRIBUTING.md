# Contributing to Mnelys

## Clean-room boundary

Mnelys is an independent implementation. Contributions must not copy or adapt source code, resources, translations, API keys, CI configuration, packaging scripts, internal data models, or other protected implementation details from Prism Launcher, MultiMC, HMCL, PCL2, or other launchers.

Behavioral compatibility may be studied through public documentation, public protocols and file formats, black-box testing, and independently produced fixtures. Record non-obvious external sources in the pull request or adjacent documentation.

If a contribution intentionally incorporates third-party code, stop before submission and document its origin, exact license, required notices, and compatibility with the repository license.

## Engineering baseline

- Keep the Rust core directed inward: `presentation → application → domain`, with infrastructure, provider, and platform crates implementing inward-facing ports. Tauri types stay inside the command and adapter boundary.
- Do not place credentials, authentication calls, archive extraction, installer execution, or launch-command construction in frontend views or view models.
- Treat the Tauri capability set as security surface. Add a capability only when a named command requires it, and justify it in the pull request.
- Make asynchronous APIs cancellable and keep blocking work off the UI thread.
- Use semantic design tokens; do not enable Mica, Acrylic, macOS vibrancy, desktop sampling, undocumented DWM blur, or transparent window composition.
- Add or update tests for behavior changes and keep logs free of secrets.
