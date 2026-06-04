# Proposal: MVP Foundation — Fix Blocking Issues

## Intent

Fix all 16 blocking issues found across 4 adversarial reviews before any feature work: Tauri 2 unwired, no CSP/capabilities, SSRF in image fetching, MIME gap, missing `.gitignore`, CI audit order wrong, no security docs, no toolchain pins. Zero new features.

## Scope

**WU1 — Tauri Wiring**: `main.rs` state/invoke_handler, `tauri.conf.json` CSP, `capabilities/default.json` permissions.
**WU2 — Security Hardening**: SSRF domain allowlist + `url::Url` parse, MIME restriction in ImageCacheService, WAL mode, `.cargo-audit.toml`, `rust-toolchain.toml`, `SECURITY.md`, `gitleaks` hook.
**WU3 — CI/CD Hardening**: `pip-audit` before scrape (scrape.yml) + before tests (ci.yml), `--validate-input` before publish, `concurrency` group in release.yml.
**WU4 — Repo Hygiene**: `.gitignore` (target/, .env, \_\_pycache\_\_/, node_modules/, \*.pyc, .DS_Store), verify FTS5 triggers in `001_init.sql`.

**Out of scope**: scraper adapters, frontend UI, backend services, new sources, all features.

## Capabilities

**New**: None. **Modified**: `local-image-cache` — ImageCacheService MUST validate Content-Type against `image/jpeg|png|webp|avif|gif` at HTTP fetch time.

## Approach

### WU1 — Tauri Wiring (researched from Tauri 2 docs)

**`main.rs`**: Use `tauri::Builder::default().manage(state).invoke_handler(tauri::generate_handler![get_product_image]).run(tauri::generate_context!())`. AppState typed as `tauri::State<'_, AppState>`.

**`tauri.conf.json` — CSP** (docs: `https://v2.tauri.app/security/csp/`):
```json
{
  "security": {
    "csp": {
      "default-src": "'self' customprotocol: asset:",
      "connect-src": "ipc: http://ipc.localhost",
      "img-src": "'self' asset: http://asset.localhost blob: data: https:",
      "style-src": "'unsafe-inline' 'self'"
    },
    "dangerousDisableAssetCspModification": false
  }
}
```
Key: Tauri 2 IPC uses `ipc:` scheme. `http://ipc.localhost` is the dev-mode IPC endpoint. Images come via `https:` (catalog URLs) and `data:` (inline cache).

**`capabilities/default.json`** (docs: `https://v2.tauri.app/security/capabilities/`):
```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "main-capability",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default"
  ]
}
```
`core:default` expands to: `core:app:default`, `core:event:default`, `core:image:default`, `core:menu:default`, `core:path:default`, `core:resources:default`, `core:tray:default`, `core:webview:default`, `core:window:default`.

Custom commands (`get_product_image`) get auto-generated permissions from the Tauri 2 build system. After first build, verify `gen/schemas` for the exact permission name and add to capabilities if needed.

### WU2 — Security Hardening (researched)

**SSRF fix**: Parse `image_url` with `url::Url::parse()` at the IPC boundary. Validate:
1. Scheme is `https` (reject anything else)
2. Host ends with `.reverb.com`, `.mlstatic.com`, or explicit allowlist domains
3. No IP literal hosts (prevent SSRF to internal IPs)

**MIME allowlist**: In `ImageCacheService::http_get()`, after fetching, validate `Content-Type` header against `["image/jpeg", "image/png", "image/webp", "image/avif", "image/gif"]`. Remove the `detect_mime_from_bytes()` fallback that defaulted to `image/jpeg` for unrecognized content.

**WAL mode**: Add `PRAGMA journal_mode=WAL;` after pool creation in `initialize_database()`. Must run BEFORE any table operations.

**Config files**:
- `.cargo-audit.toml`: `[severity]\nthreshold = "high"`
- `rust-toolchain.toml`: `[toolchain]\nchannel = "stable"\ncomponents = ["rustfmt", "clippy"]`
- `SECURITY.md`: GitHub private vulnerability reporting link + response SLA

**Pre-commit**: Add `gitleaks` hook after existing hooks.

### WU3 — CI/CD Hardening

- `scrape.yml`: Move `pip-audit --desc on` to run between `pip install` and `run_all.py`
- `ci.yml`: Move `pip-audit` before `pytest`
- `scrape.yml`: Add `run: python scraper/run_all.py --validate-input --input-dir incoming/` between `download-artifact` and `--publish-index`
- `release.yml`: Add `concurrency: group: gh-pages-publish / cancel-in-progress: false`

### WU4 — Repo Hygiene

- `.gitignore`: `target/`, `.env`, `__pycache__/`, `*.pyc`, `node_modules/`, `.DS_Store`
- `001_init.sql`: Verify FTS5 triggers reference `new.name`, `new.brand`, `new.model` etc. (already fixed)

## Affected Areas

| Area | Action |
|------|--------|
| `src-tauri/src/main.rs` | Modify — add Tauri builder + invoke handler |
| `src-tauri/tauri.conf.json` | Create — CSP object format from Tauri 2 docs |
| `src-tauri/capabilities/default.json` | Create — `core:default` minimal set |
| `src-tauri/src/lib.rs` | Modify — add WAL pragma |
| `src-tauri/src/commands/image_command.rs` | Modify — add url::Url parse + domain allowlist |
| `src-tauri/src/services/image_cache.rs` | Modify — add MIME allowlist, remove unsafe fallback |
| `.cargo-audit.toml` | Create |
| `rust-toolchain.toml` | Create |
| `SECURITY.md` | Create |
| `.pre-commit-config.yaml` | Modify — add gitleaks |
| `scrape.yml` | Modify — pip-audit order + validate-input |
| `ci.yml` | Modify — pip-audit before tests |
| `release.yml` | Modify — add concurrency group |
| `.gitignore` | Create |
| `001_init.sql` | Verify (already fixed) |

## Risks (researched and mitigated)

| Risk | Likelihood | Original Mitigation | Researched Fix |
|------|------------|-------------------|----------------|
| CSP breaks IPC | Med | Test in devtools | ✅ **Researched**: Tauri 2 IPC needs `connect-src ipc: http://ipc.localhost`. Using **object format** per official docs (not string format which is deprecated). `ipc:` scheme for production, `http://ipc.localhost` for dev mode |
| CSP breaks gh-pages sync | Med | Test in devtools | ✅ **Researched**: `connect-src` includes `ipc:` but NOT `https://*.github.io`. sync_source IPC command makes HTTP request from Rust side (not from webview), so CSP doesn't apply. Rust-side reqwest calls are not CSP-restricted |
| Capabilities syntax drift | Med | Reference Tauri 2 docs | ✅ **Researched**: Tauri 2 uses `$schema: "../gen/schemas/desktop-schema.json"` for IDE validation. `core:default` expands to 9 sub-permissions. Custom commands get auto-generated permissions at build time in `gen/schemas/`. Current version (2.x) stable since May 2025. Syntax confirmed from official docs 2026 |
| Domain allowlist too narrow | Low | Extend per reports | No change needed — MVP ships with 2 CDN domains + extensible config |

## Rollback Plan

Each WU is independent — revert per file. WU1: revert main.rs, delete configs → Tauri defaults. WU2: revert validation, delete configs → security degrades, app runs. WU3: revert YAMLs → CI less secure. WU4: delete .gitignore.

## Dependencies

`url = "2"` already in `Cargo.toml` (pre-work verified).

## Success Criteria

- [ ] `cargo build` + `cargo clippy` pass
- [ ] `tauri dev` no CSP/console errors
- [ ] `pip-audit` runs before scrape in workflows
- [ ] `.gitignore` prevents staging target/, .env, __pycache\_\_/, node_modules/
- [ ] FTS5 triggers reference real column names
- [ ] No new `unsafe` blocks
