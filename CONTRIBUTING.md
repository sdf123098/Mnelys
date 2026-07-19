# Contributing to Mnelys

## Clean-room boundary

Mnelys is an independent implementation. Contributions must not copy or adapt source code, resources, translations, API keys, CI configuration, packaging scripts, internal data models, or other protected implementation details from Prism Launcher, MultiMC, HMCL, PCL2, or other launchers.

Behavioral compatibility may be studied through public documentation, public protocols and file formats, black-box testing, and independently produced fixtures. Record non-obvious external sources in the pull request or adjacent documentation.

If a contribution intentionally incorporates third-party code, stop before submission and document its origin, exact license, required notices, and compatibility with the repository license.

## Engineering baseline

- Keep dependencies directed inward: Presentation → Application → Domain, with Infrastructure, Providers, and Platform adapters implementing inward-facing ports.
- Do not place credentials, authentication calls, archive extraction, installer execution, or launch-command construction in Views or ViewModels.
- Accept `CancellationToken` in I/O APIs and do not block asynchronous work with `.Result` or `.Wait()`.
- Use semantic design tokens; do not enable Mica, Acrylic, desktop sampling, or undocumented DWM blur APIs.
- Add or update tests for behavior changes and keep logs free of secrets.
