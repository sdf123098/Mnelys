# Security baseline

Status: Accepted through M1–M6
Last updated: 2026-10-26

## Reporting

Do not disclose suspected vulnerabilities in a public issue. Until a dedicated private reporting channel is published, contact the repository owner privately and include the affected version, reproduction steps, and impact. Do not include live credentials or personal data.

## Trust boundaries

- The webview never receives an access token or refresh token. `launch_instance` returns a session descriptor (session id and PID); the launch plan, which necessarily contains the access token as a game argument, stays inside the Rust core.
- Account summaries crossing IPC contain identity and status only. Token refresh, selection, and removal are core-side operations.
- Credentials are persisted only through the platform keystore adapter: Windows Credential Manager, macOS Keychain, and Linux Secret Service. On a machine without a usable keystore the core returns an error rather than falling back to plaintext.
- `accounts.json` contains account metadata, not secrets. Passwords are used once and dropped. Refresh tokens live in the keystore; access tokens stay in core memory only as long as needed.
- Microsoft sign-in uses the device-code flow. The client id is supplied through `MNELYS_MS_CLIENT_ID`; nothing is built in. Unconfigured sign-in fails with a readable error.
- CurseForge access uses an API key supplied through `MNELYS_CURSEFORGE_API_KEY`. The key is never sent to the webview and never written into the instance manifest.
- Downloads verify the authoritative upstream SHA-1 even when a mirror (BMCLAPI) supplies the bytes. A mirror may change transport location, never the trust hash. The final attempt goes straight to the source.
- Archives are extracted only after `enclosed_name` and regular-path-component checks; entries with traversal paths or unexpected components are skipped. Loader installers and libraries stay inside the instance and library roots.
- Long-running work is a core-side task with an immutable snapshot. The frontend polls state; it cannot cancel by aborting arbitrary I/O and it never holds a secret to perform the work itself.

## Configuration and secrets

| Value                                   | Source                      | Stored where                   |
| --------------------------------------- | --------------------------- | ------------------------------ |
| Microsoft OAuth client id               | `MNELYS_MS_CLIENT_ID`       | Environment only               |
| CurseForge API key                      | `MNELYS_CURSEFORGE_API_KEY` | Environment only               |
| Data root override                      | `MNELYS_HOME`               | Environment only               |
| Offline / Yggdrasil / Microsoft secrets | user sign-in                | Platform keystore via the core |
| Settings and instance metadata          | user actions                | `<root>/` on disk, no secrets  |

No credential is read from the webview, written to a log, or included in an error message. Log and task messages may name an account or a URL path, but not a token or key.

## Supply chain

- Dependencies use locked stable versions: `Cargo.lock` and `pnpm-lock.yaml` are committed and must be consistent with their manifests.
- Release inputs must be traceable; release builds are expected to produce an SBOM and third-party notices before the first official release.
- Update manifests and remote rule packages will require signature verification before use.
- Signing keys, OAuth client secrets, API keys, and notarization credentials never enter the repository.
- Artifacts uploaded by the `Desktop packages` workflow are unsigned test builds with a 14-day retention. They are not official releases and must not be presented as such.

## Repository hygiene

The `.gitignore` excludes common credential containers and local application data, but ignore rules are not a security control. Review staged changes before every commit and rotate any secret that enters Git history.
