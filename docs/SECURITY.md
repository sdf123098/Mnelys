# Security baseline

Status: Accepted for M0
Last updated: 2026-07-19

## Reporting

Do not disclose suspected vulnerabilities in a public issue. Until a dedicated private reporting channel is published, contact the repository owner privately and include the affected version, reproduction steps, and impact. Do not include live credentials or personal data.

## Trust boundaries

- Views and ViewModels never read or persist raw tokens.
- Credentials are persisted only through platform vault adapters: Windows Credential Manager/DPAPI, macOS Keychain, and Linux Secret Service.
- Passwords are never persisted. Short-lived access tokens remain in memory only as long as required.
- OAuth uses the system browser, PKCE, state validation, and a loopback callback with a random port and timeout.
- Downloads retain and verify the authoritative upstream hash even when a mirror is used.
- Archives are extracted through a guarded staging area with path traversal, link, count, and expansion limits.
- Loader processors, game processes, multiplayer components, the developer daemon, and the update helper have explicit process and permission boundaries.
- Logs, task snapshots, crash reports, and diagnostic exports apply centralized secret redaction.

## Supply chain

- Dependencies use central package management and locked stable versions.
- Release inputs must be traceable; release builds produce an SBOM and third-party notices.
- Update manifests and remote rule packages require signature verification before use.
- Signing keys, OAuth client secrets, API keys, and notarization credentials never enter the repository.

## Repository hygiene

The `.gitignore` excludes common credential containers and local application data, but ignore rules are not a security control. Review staged changes before every commit and rotate any secret that enters Git history.
