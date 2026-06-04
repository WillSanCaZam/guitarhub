# Proposal: Fix Critical Fallas

## Intent

Fix 4 blocking issues that prevent GuitarHub from running: runtime panic in image loading, missing SvelteKit frontend scaffolding, absent E2E data flow, and architecture inconsistencies that degrade maintainability and error handling.

## Scope

### In Scope
1. **FALLA 1** — Register `ImageCacheService` access via `AppState` in `image_command.rs` (consistent with all sibling commands)
2. **FALLA 2** — Minimal SvelteKit scaffolding: `package.json`, `app.html`, `vite.config.ts`, `svelte.config.js`, `tsconfig.json`, `src/routes/+page.svelte` consuming existing components
3. **FALLA 3** — `SyncService` contract + stub that loads products from JSON fixture into SQLite; wire it as a Tauri command with a frontend button
4. **FALLA 4** — Reuse `http_client` from `AppState` in `alert_service.rs` and `settings_command.rs`; define `AppError` (thiserror enum) replacing `Result<..., String>` across all commands

### Out of Scope
- SearchService (no spec yet, deferred)
- PriceIntelligence, ExportService wiring beyond what exists
- Real `SyncService` implementation (just contract + stub)
- Python scraper side

## Capabilities

### New Capabilities
- `sync-service`: contract and stub for loading products into SQLite from JSON fixture
- `structured-errors`: `AppError` type replacing raw `String` errors in all IPC commands

### Modified Capabilities
- None — all changes are at implementation/architecture level, no spec-level behavior changes

## Approach

| Falla | Strategy | Key Files |
|-------|----------|-----------|
| 1 | Change `get_product_image` to take `State<'_, AppState>` and use `state.image_cache_service.get(...)` | `image_command.rs` |
| 2 | Scaffold SvelteKit with `@sveltejs/adapter-static`, add route `/` that renders `ProductCard` with fixture data | `package.json`, `vite.config.ts`, `app.html`, `routes/+page.svelte` |
| 3 | Define `SyncService` trait + `JsonFixtureLoader` stub; add `load_products` Tauri command; minimal UI button | `services/sync.rs`, `commands/sync_command.rs`, frontend glue |
| 4 | Wire `state.http_client` into `NtfyAlert`/`WebhookAlert` construction; define `AppError` via thiserror; propagate through all commands | `lib.rs` (AppError), `alert_service.rs`, `settings_command.rs`, all commands |

## Affected Areas

| Area | Impact | Description |
|------|--------|------------|
| `src-tauri/src/commands/image_command.rs` | Modified | Switch from `State<'_, ImageCacheService>` to `State<'_, AppState>` |
| `src/` (frontend) | New | SvelteKit scaffolding: package.json, vite.config.ts, app.html, routes/ |
| `src-tauri/src/services/sync.rs` | New | SyncService trait + JsonFixtureLoader stub |
| `src-tauri/src/commands/sync_command.rs` | New | Tauri command wrapping SyncService |
| `src-tauri/src/lib.rs` | Modified | Add `AppError` enum |
| `src-tauri/src/commands/*.rs` | Modified | Return `AppError` instead of `String` |
| `src-tauri/src/services/alert_service.rs` | Modified | Accept `reqwest::Client` instead of creating inline |
| `src-tauri/src/commands/settings_command.rs` | Modified | Pass `state.http_client` to alert constructors |
| `src-tauri/src/main.rs` | Modified | Add `sync_command` to invoke handler |
| `src/lib/components/*.svelte` | Unchanged | Consumed by new route |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| SvelteKit version mismatch with Tauri 2 plugin | Medium | Pin `@sveltejs/kit` to 2.x compatible with adapter-static; test `npm run dev` works before `cargo tauri dev` |
| `AppError` refactor breaks existing test assertions on error strings | High | Update test assertions to match on `AppError` variants instead of string patterns |
| Fallas 2-4 may push past 400-line review budget | Medium | Split into chained PRs: PR1 = falla 1 + 4 (minimal file changes), PR2 = falla 2 (new files), PR3 = falla 3 (new files) |

## Rollback Plan

Per falla:
1. **Falla 1**: revert `image_command.rs` to original `State<'_, ImageCacheService>` and add `.manage(ImageCacheService::new_default(...))` in `main.rs` as fallback if the AppState approach breaks
2. **Falla 2**: `rm -rf src/package.json src/vite.config.ts src/svelte.config.js src/tsconfig.json src/routes/` and revert `tauri.conf.json` if present
3. **Falla 3**: revert `services/sync.rs`, `commands/sync_command.rs`, and any frontend changes
4. **Falla 4**: revert `AppError` type and return to `Result<..., String>`; restore inline `reqwest::Client` creation

Full rollback: `git revert HEAD~N` for the N commits in this change.

## Dependencies

- `npm create @sveltejs/kit@latest` or manual scaffold (manual preferred for minimalism)
- `@sveltejs/adapter-static` for Tauri-compatible static build
- `@tauri-apps/api` must be compatible with Tauri 2
- No new Rust crate dependencies for SyncService (uses existing `sqlx`, `serde_json`)

## Success Criteria

- [ ] `cargo test` passes all 123+ existing tests (no regressions)
- [ ] `cargo build` succeeds
- [ ] `npm run dev` starts Vite dev server without errors
- [ ] `npm run build` produces static output in `src/build/`
- [ ] App does not panic on `get_product_image` invoke
- [ ] Error type propagates meaningful variant info (not just `String`)
- [ ] 1 demonstrable E2E flow: frontend button → invoke `load_products` → SQLite insert → invoke `get_price_insight` works
