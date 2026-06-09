# Tasks: Post-Exploration Hardening

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~145 |
| 400-line budget risk | Low |
| Chained PRs recommended | No |
| Suggested split | Single PR |
| Delivery strategy | ask-on-risk |
| Chain strategy | pending |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: pending
400-line budget risk: Low

## Phase 1: Foundation — Config & DB wiring

- [x] 1.1 Add `"AppImage"` to `bundle.targets` in `src-tauri/tauri.conf.json`
- [x] 1.2 Inject `Arc<dyn SettingsRepository>` into `ImageCacheService` (field, constructors, Clone)
- [x] 1.3 Wire `SqliteSettingsRepository` in `lib.rs::initialize_database()`, pass to `ImageCacheService::new()`

## Phase 2: Core — Domain allowlist + defense-in-depth

- [x] 2.1 Remove `static ALLOWED_DOMAINS` from `image_command.rs`; add `allowed_domains: &[&str]` param to `validate_image_url()`
- [x] 2.2 Add `get_allowed_image_domains()` helper: read from settings, comma-parse, fallback to `["reverb.com", "mlstatic.com"]`
- [x] 2.3 Add domain validation in `ImageCacheService::get()` before HTTP call (read domain list from `settings_repo`)
- [x] 2.4 Add `allowed_image_domains` text input + load/save logic in `Settings.svelte`

## Phase 3: Core — Updater signing + packaging

- [x] 3.1 Remove `TAURI_SKIP_SIGNING` from `release.yml`; add `tauri signer sign` step with `TAURI_PRIVATE_KEY`/`TAURI_PRIVATE_KEY_PASSWORD`
- [x] 3.2 Update `scripts/generate_latest_json.py`: accept `.sig` arg, build real `url` + `signature`, drop darwin/windows platforms

## Phase 4: Testing

- [x] 4.1 Update `validate_image_url` tests to pass domain list param; add settings parse-and-fallback tests
- [x] 4.2 Add `ImageCacheService` domain rejection test with `MockSettingsRepository` — assert `Placeholder` for blocked domain
- [x] 4.3 Add pure-function test for comma-separated settings parse + fallback logic
- [x] 4.4 Test `generate_latest_json.py` rejects empty `.sig` file — exits with error before writing
- [x] 4.5 Add CI signing dry-run: verify `tauri signer sign` produces valid signature (test key, non-release)

## Phase 5: Setup (pre-merge, ops task)

- [ ] 5.1 Run `tauri signer generate` locally, copy pubkey to `tauri.conf.json`, add `TAURI_PRIVATE_KEY` + `TAURI_PRIVATE_KEY_PASSWORD` to GH secrets
