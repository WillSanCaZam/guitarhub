# Fix Critical Fallas — Specification

> **Change**: fix-critical-fallas  
> **Date**: 2026-06-04  
> **Mode**: hybrid (OpenSpec + Engram)  
> **Strict TDD**: true

---

## Domains

| Fall | Domain | Type | Requirements | Scenarios |
|------|--------|------|-------------|-----------|
| 1 | `image-command-fix` | Implementation delta | 1 | 2 |
| 2 | `frontend-scaffolding` | New infrastructure | 2 | 3 |
| 3 | `sync-service` | New capability | 3 | 4 |
| 4 | `structured-errors` | New capability | 3 | 4 |

---

## FALLA 1 — Image Command Fix

### Requirement: Access ImageCacheService via AppState

`get_product_image` MUST accept `State<'_, AppState>` and access `state.image_cache_service` instead of `State<'_, ImageCacheService>`, consistent with all sibling commands (price, settings, export).

**Scenario: Image load succeeds via AppState**  
- GIVEN `AppState` is registered with `.manage(state)` in `main.rs`  
- AND `AppState` contains `image_cache_service: ImageCacheService`  
- WHEN `get_product_image` is invoked with a valid URL  
- THEN it accesses `state.image_cache_service.get(url)`  
- AND returns a `data:{mime};base64,{b64}` string  

**Scenario: No panic on missing direct State**  
- GIVEN only `AppState` is `.manage()`'d in the Tauri builder  
- WHEN `get_product_image` is invoked  
- THEN no runtime panic occurs from an unmanaged `State<'_, ImageCacheService>`  

---

## FALLA 2 — Frontend Scaffolding

### Requirement: SvelteKit project files MUST exist

The `src/` directory MUST contain `package.json` (with `@sveltejs/kit`, `@sveltejs/adapter-static`, `svelte`, `vite`, `typescript`), `svelte.config.js` (adapter-static, no prerender), `vite.config.ts` (sveltekit plugin), `tsconfig.json` (strict mode), and `src/app.html` (minimal entry). `tauri.conf.json` `frontendDist` and `beforeDevCommand` SHOULD remain unchanged if already correct.

**Scenario: Static build produces output**  
- GIVEN all scaffold files exist under `src/`  
- WHEN `npm install && npm run build` runs  
- THEN exit code is 0  
- AND static output is produced (e.g., `src/build/index.html`)  

**Scenario: Dev server starts**  
- GIVEN all scaffold files exist  
- WHEN `npm run dev` runs  
- THEN the Vite dev server starts on `localhost:5173`  
- AND no compilation errors occur  

### Requirement: Routes MUST reference existing components

`src/routes/+layout.svelte` and `src/routes/+page.svelte` MUST exist. `+page.svelte` MAY render `ProductCard`, `PriceBadge`, `PriceChart`, `Settings`, and `ProductDetail` from `$lib/components/`.

**Scenario: Page imports existing components**  
- GIVEN `+page.svelte` imports `ProductCard` from `$lib/components/ProductCard.svelte`  
- WHEN `npm run dev` serves the page  
- THEN no module-not-found errors occur  
- AND the page renders without runtime errors  

---

## FALLA 3 — Sync Service

### Requirement: SyncService trait MUST be defined

A `SyncService` trait MUST be defined in `src-tauri/src/services/sync.rs` with method `async fn sync_from_json(&self, path: &str) -> Result<SyncResult, AppError>`.

**Scenario: Trait compiles**  
- GIVEN `SyncService` trait is defined  
- AND `SyncResult` contains `products_loaded: u32`, `products_updated: u32`  
- WHEN `cargo build` runs  
- THEN compilation succeeds  

### Requirement: JsonFixtureLoader MUST upsert products from JSON

`JsonFixtureLoader` implementing `SyncService` MUST read a JSON file matching a `CatalogFile` schema (containing `products: Vec<CatalogProduct>`), parse it, and upsert each product into `products_meta`. A sample test fixture JSON MUST exist.

**Scenario: Load valid fixture into empty DB**  
- GIVEN a fixture JSON with 2 valid products  
- WHEN `JsonFixtureLoader::sync_from_json(path)` is called  
- THEN `products_meta` contains 2 rows  
- AND `sync_state` is updated with the source  

**Scenario: Duplicate SKU is updated**  
- GIVEN product "P001" exists in `products_meta` with price 100.0  
- WHEN fixture with same SKU and price 150.0 is loaded  
- THEN `products_meta` still has 1 row for "P001"  
- AND price is now 150.0  

**Scenario: Malformed JSON returns error**  
- GIVEN the fixture file contains invalid JSON  
- WHEN `sync_from_json` is called  
- THEN a `AppError::InvalidInput` error is returned  

### Requirement: sync_catalog Tauri command MUST exist

A `#[tauri::command] sync_catalog(path: String, state: State<'_, AppState>) -> Result<SyncResult, AppError>` MUST be registered in `main.rs` invoke handler.

**Scenario: Frontend triggers catalog sync**  
- GIVEN `sync_catalog` is in `generate_handler![]`  
- WHEN frontend calls `invoke('sync_catalog', { path })`  
- THEN the loader runs and returns `SyncResult` with product counts  

---

## FALLA 4 — Structured Errors

### Requirement: AppError enum MUST be defined

An `AppError` enum in `lib.rs` (or new `errors.rs`) MUST use `thiserror` and define variants: `NotFound`, `InvalidInput(String)`, `Database(String)`, `Network(String)`, `Internal(String)`. It MUST implement `Serialize` for the Tauri IPC boundary.

**Scenario: Error variant serializes to frontend**  
- GIVEN a command returns `Err(AppError::NotFound)`  
- WHEN the frontend catches the error  
- THEN the serialized error includes a discriminant identifying the variant  

**Scenario: Validation errors carry context**  
- GIVEN a command receives invalid input  
- WHEN it returns `Err(AppError::InvalidInput("sku_required".into()))`  
- THEN the error string includes the context message  

### Requirement: All Tauri commands MUST return AppError

Every `#[tauri::command]` function MUST return `Result<T, AppError>` instead of `Result<T, String>`.

**Scenario: Export command wraps DB error**  
- GIVEN `export_data_cmd` hits a DB error  
- WHEN it returns `Err(AppError::Database(msg))`  
- THEN the variant discriminant and message propagate to the frontend  

### Requirement: AlertService MUST reuse AppState http_client

`NtfyAlert::new` and `WebhookAlert::new` MUST accept `reqwest::Client` from the caller. `test_alert_channel_cmd` MUST receive `state.http_client` rather than creating a new client inline.

**Scenario: Settings command passes shared client**  
- GIVEN `AppState.http_client` is configured with 30s timeout  
- WHEN `test_alert_channel` is invoked for "ntfy"  
- THEN `state.http_client` is passed to `NtfyAlert::new`  
- AND no additional `reqwest::Client` is created  

**Scenario: AppNotificationAlert needs no client**  
- GIVEN "app" channel is tested  
- WHEN `test_alert_channel_cmd` receives `"app"`  
- THEN it returns `Err("unsupported_in_test")`  
- AND the http_client parameter is not used  

---

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| SvelteKit version incompatibility with Tauri 2 `@tauri-apps/api` | Medium | Pin `@sveltejs/kit` to 2.x; test `npm run dev` before `cargo tauri dev` |
| `AppError` refactor breaks existing test assertions that match on `String` content | High | Update all test assertions to match on `AppError` variants; run full suite after refactor |
| `JsonFixtureLoader` schema mismatch with `products_meta` columns | Low | Match columns exactly from `001_init.sql`; integration test with real DB |

## Rollback

| Fall | Rollback |
|------|----------|
| 1 | Revert `image_command.rs` to `State<'_, ImageCacheService>`, add `.manage(ImageCacheService::new_default(...))` |
| 2 | `rm -rf src/package.json src/svelte.config.js src/vite.config.ts src/tsconfig.json src/app.html src/routes/` |
| 3 | Revert `services/sync.rs`, `commands/sync_command.rs`, `main.rs` handler, and frontend button |
| 4 | Revert `AppError`, return to `Result<..., String>`; restore inline `reqwest::Client` creation |
