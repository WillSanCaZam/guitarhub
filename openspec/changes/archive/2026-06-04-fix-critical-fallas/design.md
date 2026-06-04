# Design: Fix Critical Fallas

> **Change**: `fix-critical-fallas`
> **Strict TDD**: true

## Technical Approach

Four fixes applied in dependency order. **FALLA 4 first** (provides `AppError` type all other code needs), then **FALLA 1** (uses `AppState`), then **FALLA 2** (independent frontend scaffolding), then **FALLA 3** (uses `AppError` + frontend).

Key constraint: **only `#[tauri::command]` return types** change from `Result<T, String>` to `Result<T, AppError>`. Core logic functions (`*_cmd`, validators) keep `String` — they don't cross the IPC boundary, are tested with string assertions, and changing them adds risk without benefit at the IPC boundary. Commands map errors inline.

## Architecture Decisions

### Decision: AppError location

| Option | Tradeoff | Decision |
|--------|----------|----------|
| `lib.rs` | Flat, accessible as `crate::AppError` | **CHOSEN** — 5 variants don't warrant a separate file |
| `errors.rs` | Cleaner separation | Rejected — more files, same visibility |

### Decision: String→AppError boundary

| Option | Tradeoff | Decision |
|--------|----------|----------|
| Only `#[tauri::command]` returns change | 8 fn changes, 0 test changes | **CHOSEN** — minimal disruption |
| All functions change | 15+ fn changes, all test assertions change | Rejected — breaks 123 passing tests |

### Decision: SyncService file

| Option | Tradeoff | Decision |
|--------|----------|----------|
| `services/sync.rs` | Matches spec | **CHOSEN** |
| `services/sync_service.rs` | Consistent naming | Rejected — diverges from spec |

### Decision: SvelteKit adapter mode

| Option | Tradeoff | Decision |
|--------|----------|----------|
| adapter-static SPA (`fallback`) | Client-rendered, no prerender | **CHOSEN** — Tauri needs SPA mode |
| adapter-static prerender | Static HTML | Rejected — needs SSR-compatible setup |

## Data Flow

### FALLA 1 — Image via AppState

```
invoke('get_product_image', {imageUrl})
  → image_command::get_product_image
    → validate_image_url()          [pure fn, Err → AppError::InvalidInput]
    → state.image_cache_service.get(url)
      → SQLite / HTTP → (bytes, mime)
    → base64 encode → "data:{mime};base64,{b64}"
```

### FALLA 3 — Catalog sync

```
invoke('sync_catalog', {path})
  → sync_command::sync_catalog
    → JsonFixtureLoader::sync_from_json(path)
      → read file → serde_json::from_str<CatalogFile>
      → UPSERT each product INTO products_meta
      → UPDATE sync_state(status='done')
      → return SyncResult{products_loaded, products_updated}
```

## Module Map

```
src-tauri/src/
  lib.rs                          ← + AppError enum
  commands/
    mod.rs                        ← + sync_command
    image_command.rs              ← State<AppState>, AppError
    price_command.rs              ← AppError return
    settings_command.rs           ← AppError return
    export_command.rs             ← AppError return
    sync_command.rs               ← NEW
  services/
    mod.rs                        ← + sync
    sync.rs                       ← NEW: SyncService trait + JsonFixtureLoader
  main.rs                         ← + sync_catalog handler
src/
  package.json, svelte.config.js  ← NEW
  vite.config.ts, tsconfig.json   ← NEW
  app.html                        ← NEW
  routes/+layout.svelte           ← NEW
  routes/+page.svelte             ← NEW
  lib/components/*.svelte         ← UNCHANGED
src-tauri/fixtures/
  test_products.json              ← NEW
```

## Interfaces / Contracts

### AppError (lib.rs)

```rust
#[derive(Debug, thiserror::Error, serde::Serialize)]
pub enum AppError {
    #[error("Not found")]
    NotFound,
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Internal error: {0}")]
    Internal(String),
}
```

`serde::Serialize` is required for Tauri IPC — the frontend receives serialized error objects with a discriminant.

### SyncService trait + domain types (services/sync.rs)

```rust
#[async_trait]
pub trait SyncService: Send + Sync {
    async fn sync_from_json(&self, path: &str) -> Result<SyncResult, AppError>;
}

pub struct SyncResult {
    pub products_loaded: u32,
    pub products_updated: u32,
}

pub struct CatalogFile {
    pub source_id: String,
    pub synced_at: i64,
    pub products: Vec<CatalogProduct>,
}

pub struct CatalogProduct {
    pub sku: String,
    pub name: Option<String>,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub condition: Option<String>,
    pub url: String,
    pub image_url: Option<String>,
    // + category, subcategory, specs_json, availability, seller, location — all Option<String>
}
```

`JsonFixtureLoader` implements `SyncService`, uses `sqlx::query` with `ON CONFLICT(sku) DO UPDATE` for upserts.

### Error mapping per command

| Command | Source error → AppError variant |
|---------|--------------------------------|
| `get_product_image` | `validate_image_url` err → `InvalidInput`, `ImageCacheError` → `Network`/`Internal` |
| `get_price_history` | `validate_sku` err → `InvalidInput`, repo → `Database` |
| `get_price_insight` | same as above |
| `save_setting` | empty key → `InvalidInput`, repo err → `Database` |
| `test_alert_channel` | `invalid_channel` → `InvalidInput`, send err → `Network` |
| `export_data` | empty path → `InvalidInput`, `ExportError::Write`→ `Internal`, `ExportError::Query`→ `Database` |
| `sync_catalog` | file I/O → `Internal`, parse → `InvalidInput`, DB → `Database` |

## File Changes

| File | Action |
|------|--------|
| `src-tauri/src/lib.rs` | **Modify** — add `AppError` enum, add `pub mod sync_catalog;` when sync is wired |
| `src-tauri/src/commands/image_command.rs` | **Modify** — `State<'_, AppState>`, import `AppState` + `AppError`, map errors |
| `src-tauri/src/commands/price_command.rs` | **Modify** — return `Result<_, AppError>`, map `validate_sku` + repo errors |
| `src-tauri/src/commands/settings_command.rs` | **Modify** — return `Result<_, AppError>`, map channel/repo errors |
| `src-tauri/src/commands/export_command.rs` | **Modify** — return `Result<_, AppError>`, map `ExportError` variants |
| `src-tauri/src/commands/mod.rs` | **Modify** — add `pub mod sync_command;` |
| `src-tauri/src/commands/sync_command.rs` | **Create** — `sync_catalog` command wrapping `JsonFixtureLoader` |
| `src-tauri/src/services/sync.rs` | **Create** — trait, `JsonFixtureLoader`, domain types, tests |
| `src-tauri/src/services/mod.rs` | **Modify** — add `pub mod sync;` |
| `src-tauri/src/main.rs` | **Modify** — register `sync_catalog` in `generate_handler![]` |
| `src/package.json` | **Create** |
| `src/svelte.config.js` | **Create** |
| `src/vite.config.ts` | **Create** |
| `src/tsconfig.json` | **Create** |
| `src/app.html` | **Create** |
| `src/routes/+layout.svelte` | **Create** |
| `src/routes/+page.svelte` | **Create** |
| `src-tauri/fixtures/test_products.json` | **Create** |

11 new, 7 modified, 0 deleted.

## Testing Strategy

| Area | What | Approach |
|------|------|----------|
| Existing unit tests | `validate_image_url`, `validate_sku`, `validate_key`, `validate_webhook_url`, all `*_cmd` fns | **UNCHANGED** — pure fns return `Result<..., String>`, no Tauri command tests exist to break |
| New SyncService tests | `JsonFixtureLoader` with valid JSON, duplicate SKU, malformed JSON | In-memory SQLite with products_meta + sync_state tables |
| SvelteKit build | `npm install && npm run build` | CI: verify static output in `src/build/` |

**Key insight**: Changing the `#[tauri::command]` return types requires **zero existing test modifications** because no tests directly test the Tauri command fns — they test the `*_cmd` core logic and pure validators below them.

## Migration Path

**Apply order** (strict):

1. **FALLA 4 first** — Define `AppError` in `lib.rs`. Update all command return types. No test changes expected.
2. **FALLA 1** — Change `image_command.rs` State + error mapping. Tests unchanged.
3. **FALLA 2** — Create all SvelteKit scaffolding files. No Rust deps. Verify `npm run build`.
4. **FALLA 3** — Create sync.rs, sync_command.rs, test fixture, frontend button.

PR split recommendation: **3 chained PRs** — (FALLA 4 + 1) ~200 lines, (FALLA 2) ~150 lines, (FALLA 3) ~250 lines.

## Risk Assessment

| Risk | L | Mitigation |
|------|---|------------|
| AppError migration breaks existing tests despite "core logic unchanged" claim | Low | Run full suite after each command migration — verify assertion |
| SvelteKit `@sveltejs/kit` v2.x incompatibility with Tauri 2 `@tauri-apps/api` | Medium | Pin versions; test `npm run dev` before `cargo tauri dev` |
| `CatalogProduct` field mismatch with `products_meta` columns | Low | Schema derived from `001_init.sql`; integration test with real DB |
| 4 fallas exceed 400-line review budget | Medium | Split into 3 chained PRs per migration path above |

## Open Questions

None — all design decisions resolved against existing codebase analysis.
