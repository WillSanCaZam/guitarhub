# Tasks: Fix Critical Fallas

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~420 |
| 400-line budget risk | Medium |
| Chained PRs recommended | Yes |
| Suggested split | PR 1: FALLA 4+1 (~200 lines Rust), PR 2: FALLA 2 (~130 lines JS/TS), PR 3: FALLA 3 (~240 lines Rust+JSON) |
| Delivery strategy | ask-on-risk |
| Chain strategy | pending |

Decision needed before apply: Yes
Chained PRs recommended: Yes
Chain strategy: pending
400-line budget risk: Medium

## Phase 1: AppError & Return Types (FALLA 4)

- [ ] 1.1 **RED→GREEN** — Write `app_error_serializes_to_frontend` test verifying each variant produces correct serialized discriminant. Define `#[derive(thiserror::Error, Serialize)] pub enum AppError` in `src-tauri/src/lib.rs` with variants NotFound, InvalidInput(String), Database(String), Network(String), Internal(String).
- [ ] 1.2 Modify `src-tauri/src/commands/image_command.rs`: `get_product_image` returns `Result<String, AppError>`; map `validate_image_url` err → `InvalidInput`, cache err → `Network`/`Internal`.
- [ ] 1.3 Modify `src-tauri/src/commands/price_command.rs`: `get_price_history`/`get_price_insight` return `Result<_, AppError>`; map `validate_sku` → `InvalidInput`, repo → `Database`.
- [ ] 1.4 Modify `src-tauri/src/commands/settings_command.rs`: `get_setting`, `save_setting`, `delete_setting`, `test_alert_channel` return `Result<_, AppError>`; map empty key → `InvalidInput`, send err → `Network`.
- [ ] 1.5 Modify `src-tauri/src/commands/export_command.rs`: `export_data` returns `Result<_, AppError>`; map `ExportError::Write` → `Internal`, `ExportError::Query` → `Database`.
- [ ] 1.6 **Verify only** — `NtfyAlert::new` / `WebhookAlert::new` already accept `reqwest::Client` param; `test_alert_channel` already passes `state.http_client`. Confirm no inline client creation in production code.

## Phase 2: Image State Fix (FALLA 1)

- [ ] 2.1 Change `get_product_image` signature: `state: State<'_, ImageCacheService>` → `state: State<'_, AppState>`; access `state.image_cache_service.get(url)`.
- [ ] 2.2 Verify `cargo test` passes — existing `validate_image_url` tests are unaffected (pure fn, no State dependency).

## Phase 3: SvelteKit Scaffolding (FALLA 2)

- [x] 3.1 Create `package.json` at project root with `@sveltejs/kit@^2`, `@sveltejs/adapter-static@^3`, `svelte@^5`, `vite@^6`, `typescript@^5`.
- [x] 3.2 Create `svelte.config.js` (adapter-static SPA fallback), `vite.config.ts` (sveltekit plugin), `tsconfig.json` (strict mode).
- [x] 3.3 Create `src/app.html` (minimal entry with `%sveltekit.head%` / `%sveltekit.body%`), `src/routes/+layout.svelte`, `src/routes/+page.svelte` importing existing `$lib/components/*`.
- [ ] 3.4 Verify `npm install && npm run build` produces static output in `build/` (CI step — deps not installed yet).

## Phase 4: Sync Service Stub (FALLA 3)

- [ ] 4.1 **RED** — Write tests for `JsonFixtureLoader` in `src-tauri/src/services/sync.rs`: valid fixture loads 2 products; duplicate SKU upserts price; malformed JSON returns `AppError::InvalidInput`.
- [ ] 4.2 **GREEN** — Create `src-tauri/src/services/sync.rs` with domain types (`CatalogFile`, `CatalogProduct`, `SyncResult`), `SyncService` trait, `JsonFixtureLoader` impl with `ON CONFLICT(sku) DO UPDATE` UPSERT into `products_meta` + `sync_state` update.
- [ ] 4.3 Create `src-tauri/src/commands/sync_command.rs` with `#[tauri::command] sync_catalog(path: String, state: State<'_, AppState>) -> Result<SyncResult, AppError>`.
- [ ] 4.4 Wire modules: add `pub mod sync;` to `services/mod.rs`, `pub mod sync_command;` to `commands/mod.rs`, register `sync_catalog` in `main.rs` `generate_handler![]`.
- [ ] 4.5 Create `src-tauri/fixtures/test_products.json` with 2+ products matching `CatalogProduct` schema.
- [ ] 4.6 Verify `cargo test` passes all 123+ tests (existing + new SyncService tests).
