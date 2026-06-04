# Tasks: MVP Foundation

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~240 (additions + deletions) |
| 400-line budget risk | Low |
| Chained PRs recommended | No |
| Suggested split | Single PR |
| Delivery strategy | ask-on-risk |
| Chain strategy | pending |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: pending
400-line budget risk: Low

### Suggested Work Units

| Unit | Goal | Notes |
|------|------|-------|
| 1 | Tauri Wiring — configs + main.rs builder | Independent, no deps on other WUs |
| 2 | Security Hardening — SSRF, MIME, WAL, configs | Independent |
| 3 | CI/CD Hardening — workflow reorder + concurrency | Independent |
| 4 | Repo Hygiene — .gitignore, FTS5 audit | Independent |

## Phase 1: Foundation

- [x] 1.1 Create `src-tauri/tauri.conf.json` with CSP object format (`connect-src ipc:`, `img-src https: data:`)
- [x] 1.2 Create `src-tauri/capabilities/default.json` — `core:default` + `main-capability` identifier
- [x] 1.3 Add `PRAGMA journal_mode=WAL` to `initialize_database()` in `lib.rs` (after pool creation, before migrations)
- [x] 1.4 Create `.gitignore` at project root (`target/`, `.env`, `__pycache__/`, `*.pyc`, `node_modules/`, `.DS_Store`)

## Phase 2: Core Implementation

- [x] 2.1 Wire `tauri::Builder::default()` in `main.rs` — `.manage(state)`, `invoke_handler` with `get_product_image`
- [x] 2.2 Add `validate_image_url()` to `image_command.rs` — `url::Url::parse`, domain allowlist, IP literal rejection
- [x] 2.3 Restrict `ImageCacheService::http_get()` MIME to allowlist — remove `detect_mime_from_bytes()` fallback
- [x] 2.4 Create `.cargo-audit.toml` (threshold `"high"`), `rust-toolchain.toml` (stable + rustfmt/clippy), `SECURITY.md`

## Phase 3: CI/CD & Tooling

- [x] 3.1 `scrape.yml` — move `pip-audit` before scraper exec; add `--validate-input` step before `--publish-index`
- [x] 3.2 `ci.yml` — move `pip-audit` before `pytest`
- [x] 3.3 `release.yml` — add `concurrency: group: gh-pages-publish / cancel-in-progress: false`
- [x] 3.4 Add `gitleaks` hook to `.pre-commit-config.yaml` after existing hooks

## Phase 4: Verify

- [x] 4.1 Write unit tests for `validate_image_url()` — 8 cases (HTTP, IPv4, IPv6, non-allowlisted, malformed, valid reverb, subdomain, exact domain)
- [x] 4.2 Verify FTS5 triggers in `001_init.sql` — `products_meta` includes all columns (`name`, `brand`, `model`, `category`, `subcategory`, `specs_json`) referenced by triggers ✓
- [x] 4.3 `cargo build` + `cargo clippy` pass (43 tests pass, 0 warnings); `gen/schemas/` will generate on first `tauri dev` build
