# ADR-004: CSP Security Headers in Tauri

## Status

Accepted

## Context

GuitarHub is a Tauri desktop app that loads web content (Svelte frontend) in a webview. The app also fetches external resources (images, catalog JSON from GitHub Pages). Without proper Content Security Policy, the app is vulnerable to:
- XSS attacks via injected scripts
- Data exfiltration to malicious servers
- Loading untrusted content

The app must be secure by default while still being functional.

## Decision

Implement strict Content Security Policy in `tauri.conf.json` with minimum Tauri permissions:

- CSP headers restrict script, style, and connection sources to known domains
- Tauri permissions follow minimum privilege: NO `shell:all`, `fs:all`, or `http:all`
- Every permission change requires justification and review
- CSP violations are logged but don't break functionality

Key CSP directives:
```
default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' https://pages.guitarhub.app data:;
```

## Consequences

**Easier:**
- Mitigates XSS and data injection risks
- Clear security boundaries — what the app can and cannot load
- Audit trail for permission changes via git history
- User trust — app doesn't phone home or load random scripts

**More difficult:**
- Adding new external resources requires CSP update
- Some legitimate use cases (inline styles) need `unsafe-inline` (mitigated by keeping it minimal)
- Development mode needs relaxed CSP for hot reload

## Alternatives Considered

1. **No CSP** — Rejected: Security risk, any injected script could run unrestricted
2. **Overly broad CSP (`*`)** — Rejected: Defeats the purpose, allows loading anything
3. **CSP with nonce-based scripts** — Rejected: Complex to implement in Tauri, `self` is sufficient for a desktop app
4. **Sandboxed webview** — Rejected: Tauri already provides process isolation; CSP adds defense-in-depth
