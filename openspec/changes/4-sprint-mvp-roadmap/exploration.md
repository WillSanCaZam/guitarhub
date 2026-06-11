## Exploration: 4-Sprint MVP Consolidation Roadmap

### Current State

GuitarHub is a Tauri 2 + Svelte 5 desktop app at v0.3.0 with a mature Rust backend (Clean Architecture: commands → services → repository) and a frontend that mixes Svelte 5 runes (`$state`, `$derived`, `$props`) with Svelte 4 `writable` stores. The codebase has strong unit test coverage in Rust (inline `#[cfg(test)]` modules with httpmock) and 7 E2E spec files using WebdriverIO + tauri-driver.

**High-severity debt identified:**
1. `sync_command.rs` lines 22-82 and 97-157 are **byte-for-byte identical** — 60 lines of alert dispatch logic duplicated across `sync_catalog` and `sync_local_catalog`
2. `+page.svelte` is an **800-line monolith** (133 lines script + 205 lines template + 460 lines `<style>`) containing search logic, dashboard composition, collection stats, and all CSS

**Medium-severity debt:**
3. No Rust integration tests (`src-tauri/tests/` contains only `fixtures/sample_catalog.json`)
4. E2E tests exist but have never been verified green in CI (`.github/workflows/e2e.yml` exists but the file appears truncated at line 35)
5. Sequential product upserts in `sync.rs` lines 132-216 — each product is individually INSERTed and price_history-recorded in a loop with no batching or transactions
6. No virtual scrolling — `ProductCard` renders all results in a CSS grid; with large catalogs this will degrade

---

### Sprint 1: Backend Alert Refactoring (3.5h)

**Focus:** Eliminate the 60-line duplication in `sync_command.rs` by extracting a shared `dispatch_price_drops()` function.

#### Files to Modify

| File | Lines | Action |
|------|-------|--------|
| `src-tauri/src/commands/sync_command.rs` | 22-82, 97-157 | Replace both blocks with a single call to `dispatch_price_drops()` |
| `src-tauri/src/commands/sync_command.rs` | new function | Add `async fn dispatch_price_drops(result: &mut SyncResult, app: &AppHandle, pool: &SqlitePool, http_client: &reqwest::Client)` |

#### Current Duplication Analysis

Lines 22-82 (`sync_catalog`) and lines 97-157 (`sync_local_catalog`) are **identical**:
```rust
if !result.drops.is_empty() {
    let settings_repo = SqliteSettingsRepository::new(state.pool.clone());
    let channel = settings_repo.get("alert_channel").await;
    let config = settings_repo.get("alert_config").await;
    if let Some(channel) = channel {
        let now = ...;
        match channel.as_str() {
            "app" => { /* ~25 lines of tauri_plugin_notification dispatch */ }
            _ => { /* try_build_dispatcher + dispatch_drops call */ }
        }
    }
}
```

#### Proposed Implementation

Extract into a private function in `sync_command.rs`:

```rust
/// Dispatch all price drops in `result.drops` through the configured alert channel.
/// Updates `result.drops_sent` with the count of successfully sent alerts.
async fn dispatch_price_drops(
    result: &mut SyncResult,
    app: &AppHandle,
    pool: &sqlx::SqlitePool,
    http_client: &reqwest::Client,
) {
    if result.drops.is_empty() { return; }
    
    let settings_repo = SqliteSettingsRepository::new(pool.clone());
    let channel = settings_repo.get("alert_channel").await;
    let config = settings_repo.get("alert_config").await;
    
    let Some(channel) = channel else { return; };
    
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    
    match channel.as_str() {
        "app" => {
            let repo = PriceDropNotificationsRepo::new(pool.clone());
            for drop in &result.drops {
                let title = format!("Price Drop: {}", drop.sku);
                let message = format!(
                    "{} dropped from ${:.2} to ${:.2}",
                    drop.sku, drop.previous_price, drop.new_price
                );
                match app.notification().builder().title(&title).body(&message).show() {
                    Ok(_) => {
                        if let Err(e) = repo.upsert(&drop.sku, now, drop.new_price, "app").await {
                            tracing::error!("failed to record notification cooldown: {}", e);
                        }
                        result.drops_sent += 1;
                    }
                    Err(e) => { tracing::error!("alert dispatch failed: {}", e); }
                }
            }
        }
        _ => {
            if let Some(dispatcher) = try_build_dispatcher(&channel, config.as_deref()) {
                let sent = dispatch_drops(
                    &result.drops, dispatcher.as_ref(), pool, http_client, &channel, now,
                ).await;
                result.drops_sent += sent;
            }
        }
    }
}
```

Both `sync_catalog` and `sync_local_catalog` then become:
```rust
let mut result = service.sync_catalog(&url).await?;
dispatch_price_drops(&mut result, &app, &state.pool, &state.http_client).await;
Ok(result)
```

#### Dependencies & Prerequisites
- None — this is a pure internal refactor. No public API changes.
- Existing unit tests in `sync_command.rs` (lines 227-378) test `dispatch_drops` and `try_build_dispatcher` directly — they remain valid.
- The `AppHandle` dependency is the reason the "app" channel logic lives in the command layer (documented in `alert_service.rs` lines 80-88: `AppNotificationAlert` is a stub because the service layer has no `AppHandle`).

#### Risk Assessment
- **Risk**: Low. The extracted function is a 1:1 copy of the existing logic. No behavioral change.
- **Regression test**: `cargo test` — all 4 existing `sync_command` tests + 12 `sync.rs` tests must pass.

#### Deliverables
- [ ] `dispatch_price_drops()` function extracted
- [ ] `sync_catalog` body reduced to ~5 lines
- [ ] `sync_local_catalog` body reduced to ~5 lines
- [ ] `cargo test` passes (no test changes needed)
- [ ] `cargo clippy` clean

---

### Sprint 2: Frontend Decomposition (3.5h)

**Focus:** Decompose the 800-line `+page.svelte` into focused Svelte 5 rune-based components.

#### Current Structure Analysis

`+page.svelte` contains **9 logical sections** rendered in a bento-grid:

| # | Cell | Lines (template) | Logic | Extract? |
|---|------|-------------------|-------|----------|
| 1 | Hero (Search + Results) | 138-205 | search(), loadMore(), handleKeydown() | **YES** — `SearchPanel.svelte` |
| 2 | Sync Status | 208-232 | reads `$syncResult` | **YES** — `SyncStatusCell.svelte` |
| 3 | Total Products | 235-242 | reads `$dashboardStats` | Keep inline (trivial) |
| 4 | Wishlist Count | 245-252 | reads `$dashboardStats` | Keep inline (trivial) |
| 5 | Recent Searches | 255-265 | reads `$dashboardStats` | Keep inline (trivial) |
| 6 | Featured Deal | 268-274 | `$derived` from results | Keep inline (trivial) |
| 7 | Quick Settings | 277-284 | scrollIntoView | Keep inline (trivial) |
| 8 | Collection Stats | 287-320 | reads `$collectionStore`, calculates gain/loss | **YES** — `CollectionStatsCell.svelte` |
| 9 | About | 323-332 | reads `pkg.version` | Keep inline (trivial) |

#### Proposed Component Decomposition

**1. `SearchPanel.svelte`** (~120 lines)
- Props: none (reads from filter store internally)
- State (runes): `query`, `results`, `total`, `page`, `pageSize`, `loading`, `error`, `searched`
- Functions: `search()`, `handleSearch()`, `handleKeydown()`, `loadMore()`
- Derived: `hasMore`, `featuredProduct`
- Uses: `ProductCard`, `FilterBar`, `DashboardCell`

**2. `SyncStatusCell.svelte`** (~40 lines)
- Props: none (reads `$syncResult` from store)
- Derived: `drops`, `dropsSent`
- Uses: `DashboardCell`

**3. `CollectionStatsCell.svelte`** (~50 lines)
- Props: none (reads `$collectionStore` from store)
- Derived: `collectionGainLoss`, `collectionGainLossFormatted`
- Uses: `DashboardCell`

**4. `DashboardGrid.svelte`** (~80 lines)
- The bento-grid layout wrapper that composes all cells
- Receives the 3 extracted components + inline trivial cells

#### Store Migration Strategy

Current stores use Svelte 4 `writable`:
- `sync.ts` → `writable<SyncResult | null>`
- `dashboard.ts` → `writable<DashboardStats>`
- `collection.ts` → `writable<CollectionStore>`
- `filter.ts` → `writable<FilterState>`
- `wishlist.ts` → `writable<WishlistStore>`

**Recommendation for Sprint 2**: Do NOT migrate stores to runes yet. The stores work correctly and are consumed via `$storeName` syntax which is compatible with Svelte 5. A store migration is a separate concern that would expand scope beyond 3.5h. The new components use `$state` for local UI state and `$derived` for computed values, but continue reading shared state from writable stores.

#### CSS Extraction
- Move cell-specific styles into their respective components (scoped `<style>` blocks)
- Keep grid layout styles (`.bento-grid`, `.cell-*`) in `+page.svelte`
- Move shared dark-mode and responsive overrides into a global CSS file or into each component

#### Files to Create/Modify

| File | Action |
|------|--------|
| `src/lib/components/SearchPanel.svelte` | **CREATE** — search bar + results grid + load more |
| `src/lib/components/SyncStatusCell.svelte` | **CREATE** — sync status display |
| `src/lib/components/CollectionStatsCell.svelte` | **CREATE** — collection stats display |
| `src/routes/+page.svelte` | **MODIFY** — reduce to ~150 lines (grid layout + trivial cells + imports) |
| `src/lib/components/__tests__/SearchPanel.test.ts` | **CREATE** — basic render tests |

#### Dependencies & Prerequisites
- Existing `DashboardCell.svelte` already uses Svelte 5 `$props()` with snippets — good pattern to follow
- Existing `ProductCard.svelte` uses `$state()` and `$props()` — already Svelte 5 native
- `FilterBar.svelte` is already extracted

#### Risk Assessment
- **Risk**: Medium. The search panel extraction moves the most complex logic. The `search()` function reads `filterStore` via `get()` (Svelte 4 pattern) — this must be preserved or converted to `$derived` in the new component.
- **Regression**: E2E test `02-search.spec.ts` and `06-dashboard.spec.ts` must still pass.
- **CSS scoping**: Svelte's scoped styles mean moving CSS into components should work, but dark-mode overrides that target child components may need `:global()` or `:deep()`.

#### Deliverables
- [ ] `SearchPanel.svelte` created with all search logic
- [ ] `SyncStatusCell.svelte` created
- [ ] `CollectionStatsCell.svelte` created
- [ ] `+page.svelte` reduced to ≤200 lines
- [ ] `npm run test` passes (vitest)
- [ ] No visual regression (manual check)

---

### Sprint 3: Optimization & Stability (3.5h)

**Focus:** Virtual scrolling for product results + batch upserts in SQLite.

#### 3A: Virtual Scrolling (Frontend — ~1.5h)

**Current rendering** (`+page.svelte` line 186):
```svelte
{#each results as item (item.sku)}
  <ProductCard product={item} inCollection={$collectionStore.collectedSkus.has(item.sku)} />
{/each}
```

All results render at once. With `pageSize=20` and "Load More" pagination, a user could accumulate 100+ cards in the DOM. Each `ProductCard` also fires 2 `invoke()` calls on mount (`get_product_image` + `get_price_insight`), so 100 cards = 200 IPC calls at render time.

**Library choice:**

| Option | Pros | Cons | Effort |
|--------|------|------|--------|
| `@tanstack/virtual` (Svelte adapter) | Framework-agnostic, well-maintained, headless | Requires manual DOM wiring, CSS grid support needs work | Medium |
| `svelte-tiny-virtual-list` | Zero-deps, Svelte-native, simple API | Less flexible for CSS grid layouts | Low |
| `@sveltejs/svelte-virtual-list` | Official Svelte package | Minimal docs, limited features | Low |

**Recommendation**: `svelte-tiny-virtual-list` — zero dependencies, Svelte-native, and the product grid uses a fixed card height (~300px with image + text). For a CSS grid with `minmax(220px, 1fr)`, we'd virtualize per-row rather than per-item, or switch to a single-column virtual list on the search results section.

**Alternative (simpler)**: Instead of virtual scrolling, implement **windowed pagination** — replace "Load More" (which accumulates DOM nodes) with page-based navigation that replaces results. This is simpler, doesn't require a new dependency, and solves the DOM growth problem.

**Implementation (windowed pagination approach):**
- Modify `search()` in `SearchPanel.svelte` to replace results instead of appending
- Add prev/next page buttons instead of "Load More"
- Keep `pageSize=20` — only 20 cards in DOM at any time

#### 3B: Batch Upserts (Backend — ~2h)

**Current code** (`sync.rs` lines 132-216): Each product in the catalog is individually:
1. `INSERT OR REPLACE INTO products_meta` (line 148-173)
2. `price_history.record_price()` (line 180-190)
3. Price drop detection + cooldown check (lines 194-214)

For a catalog of 500 products, this is 1000+ individual SQL statements.

**Proposed batch approach:**

```rust
async fn upsert_products(
    &self,
    source_id: &str,
    products: &[RawProduct],
) -> Result<(u32, u32, Vec<PriceDrop>), AppError> {
    let synced_at = /* ... */;
    let mut tx = self.pool.begin().await.map_err(|e| AppError::Database(e.to_string()))?;
    
    // Batch 1: Upsert all products_meta in a single transaction
    for p in products {
        sqlx::query(/* INSERT OR REPLACE */)
            .bind(/* ... */)
            .execute(&mut *tx)
            .await?;
    }
    
    // Batch 2: Insert all price_history rows
    for p in products {
        sqlx::query("INSERT INTO price_history (sku, price, recorded_at, source_id) VALUES (?, ?, ?, ?)")
            .bind(&p.sku).bind(p.price).bind(synced_at).bind(source_id)
            .execute(&mut *tx)
            .await?;
    }
    
    tx.commit().await.map_err(|e| AppError::Database(e.to_string()))?;
    
    // Drop detection runs AFTER the transaction (reads from committed data)
    let drops = self.detect_drops(products, source_id, synced_at).await?;
    
    Ok((products.len() as u32, products.len() as u32, drops))
}
```

**Key insight**: The current code reads `price_history.get_last_price()` BEFORE inserting the new row (line 135). With batching, we need to either:
- (A) Read all previous prices in a single batch query before the transaction, then insert all new prices
- (B) Keep the per-product read-then-write but wrap everything in a transaction

**Recommendation**: Option (B) — wrap the existing loop in a transaction. This is the minimal change that gives the biggest performance win (SQLite transactions are ~100x faster than individual statements). Drop detection logic stays the same.

#### Files to Modify

| File | Lines | Action |
|------|-------|--------|
| `src-tauri/src/services/sync.rs` | 110-217 | Wrap `upsert_products` loop in a `sqlx` transaction |
| `src/lib/components/SearchPanel.svelte` | (from Sprint 2) | Replace "Load More" with page-based navigation |

#### Dependencies & Prerequisites
- Sprint 2 must be complete (SearchPanel.svelte exists)
- `sqlx` already supports transactions via `pool.begin()` — no new dependency

#### Risk Assessment
- **Risk (batch)**: Low-Medium. Wrapping in a transaction changes error semantics — if one product fails, the entire batch rolls back. Current behavior: partial success. Need to decide: fail-all or skip-and-continue within a transaction.
  - **Mitigation**: Use savepoints or catch per-product errors within the transaction.
- **Risk (pagination)**: Low. Replacing "Load More" with page navigation is a UX change — users lose the ability to scroll through all results. But with FTS5 search, results should be targeted enough.

#### Deliverables
- [ ] `upsert_products` wrapped in a SQLite transaction
- [ ] Benchmark: sync time before/after for a 500-product catalog
- [ ] Search results use page-based navigation (no DOM accumulation)
- [ ] `cargo test` passes (existing sync tests validate behavior)
- [ ] `npm run test` passes

---

### Sprint 4: Testing & Final Polish (3.5h)

**Focus:** Rust integration tests + E2E test verification + CI pipeline hardening.

#### 4A: Rust Integration Tests (~1.5h)

**Current state**: `src-tauri/tests/` contains only `fixtures/sample_catalog.json`. All Rust tests are inline `#[cfg(test)]` modules. The inline tests are thorough (sync.rs has 1185 lines with ~20 tests, sync_command.rs has 150 lines of tests, alert_service.rs has 587 lines with ~20 tests).

**What's missing**: Integration tests that exercise the full command → service → repository pipeline with a real (in-memory) database.

**Proposed integration test file**: `src-tauri/tests/integration_test.rs`

```rust
// Tests that exercise the full stack: command → service → repository → SQLite
use guitarhub_lib::*; // or however the lib crate exports

#[tokio::test]
async fn sync_and_search_roundtrip() {
    // 1. Set up in-memory SQLite with all migrations
    // 2. Call CatalogSyncService::sync_catalog() with a mock HTTP server
    // 3. Verify products in DB
    // 4. Call search logic
    // 5. Verify search results match
}

#[tokio::test]
async fn sync_detects_drops_and_records_cooldown() {
    // 1. Seed price_history with old prices
    // 2. Sync catalog with lower prices
    // 3. Verify drops detected
    // 4. Verify cooldown rows written
}

#[tokio::test]
async fn collection_add_and_stats() {
    // 1. Sync products
    // 2. Add to collection
    // 3. Verify collection stats
}
```

**Challenge**: The `guitarhub_lib` crate exports need to be checked. The `Cargo.toml` defines `crate-type = ["staticlib", "cdylib", "rlib"]` which means integration tests can import via `guitarhub_lib`.

#### 4B: E2E Test Verification (~1h)

**Current state**: 7 spec files exist:
1. `01-app-launch.spec.ts` — verifies window title + dashboard cells
2. `02-search.spec.ts` — searches for "Fender", expects results
3. `03-sync.spec.ts` — clicks sync, expects toast
4. `04-collection.spec.ts` — searches "Gibson", adds to collection
5. `05-settings.spec.ts` — changes alert channel to ntfy
6. `06-dashboard.spec.ts` — verifies 9 bento grid cells
7. `07-filters.spec.ts` — toggles filters, sets/clears values

**Framework**: WebdriverIO + Mocha + tauri-driver. Config in `wdio.conf.ts` builds the app in `onPrepare`, seeds DB in `beforeSession`, spawns `tauri-driver` on port 4444.

**CI**: `.github/workflows/e2e.yml` runs on push/PR to main + weekly Saturday. Uses `xvfb-run` for headless WebKit.

**Issues to fix**:
- The `e2e.yml` file appears truncated (line 31 has raw YAML content mixed into the `+layout.svelte` output — likely a file read artifact). Need to verify the actual CI workflow is complete.
- `seedDb.ts` needs to be verified — does it populate the SQLite DB with test data that the specs expect?
- Selectors in `utils/selectors.ts` need to match the actual DOM after Sprint 2 refactoring.

#### 4C: CI Pipeline Hardening (~1h)

**Current workflows**:
- `ci.yml` — likely runs `cargo test`, `npm test`, linting
- `e2e.yml` — E2E tests with tauri-driver
- `release.yml` — build + publish
- `scrape.yml` — Python scraper cron

**Polish items**:
- Verify `ci.yml` runs `cargo clippy -- -D warnings` and `cargo fmt --check`
- Add coverage reporting for Rust (`cargo tarpaulin` or `cargo llvm-cov`)
- Ensure E2E workflow has proper artifact upload on failure (screenshots)

#### Files to Create/Modify

| File | Action |
|------|--------|
| `src-tauri/tests/integration_test.rs` | **CREATE** — 3-5 integration tests |
| `.github/workflows/e2e.yml` | **VERIFY/FIX** — ensure complete and correct |
| `e2e-tests/utils/selectors.ts` | **VERIFY** — selectors match post-refactor DOM |
| `e2e-tests/utils/seedDb.ts` | **VERIFY** — seeds match test expectations |

#### Dependencies & Prerequisites
- Sprints 1-3 complete (refactored code must be stable)
- `tauri-driver` installed locally for E2E verification

#### Risk Assessment
- **Risk (integration tests)**: Low. The inline tests already prove the logic works. Integration tests validate the wiring.
- **Risk (E2E)**: Medium. Tauri-driver + WebdriverIO on Linux/WebKit can be flaky. The `xvfb-run` approach is standard but timing issues are common.
- **Risk (CI)**: Low. Workflow files are standard GitHub Actions.

#### Deliverables
- [ ] `integration_test.rs` with ≥3 passing tests
- [ ] E2E tests run green locally (`npm run test:e2e`)
- [ ] E2E workflow verified in CI (or fixed)
- [ ] `cargo clippy -- -D warnings` clean
- [ ] Coverage report generated (even if not enforced)

---

### Approaches

1. **Sequential sprints (recommended)** — Execute sprints 1→2→3→4 in order. Each sprint builds on the previous. Backend refactoring first (lowest risk), then frontend (medium risk), then optimization (needs stable components), then testing (needs stable code).
   - Pros: Dependencies are clean, each sprint is independently shippable
   - Cons: Sprint 3 depends on Sprint 2's SearchPanel existing
   - Effort: 14h total (4 × 3.5h)

2. **Parallel backend + frontend** — Sprints 1 and 2 could run in parallel since they touch different layers.
   - Pros: Faster calendar time
   - Cons: Requires 2 developers, integration risk
   - Effort: Same 14h, but 2 calendar sprints

### Recommendation

**Sequential approach (Option 1)** is recommended for a solo developer. Each sprint produces a clean, shippable commit. The ordering prioritizes high-severity debt first (Sprint 1: duplication, Sprint 2: monolith), then performance (Sprint 3), then test coverage (Sprint 4).

### Risks

- **Sprint 2 scope creep**: The component extraction could reveal additional coupling (e.g., `FilterBar` reading from `filterStore` which is also read by `search()`). Mitigation: keep stores as-is, only extract template + local state.
- **Sprint 3 transaction semantics**: Wrapping upserts in a transaction changes partial-failure behavior. Mitigation: keep per-product error handling inside the transaction, only commit if all succeed.
- **E2E flakiness**: Tauri-driver E2E tests are notoriously timing-sensitive. Mitigation: generous `waitUntil` timeouts (already 10-30s in existing specs).
- **Sprint 4 integration test imports**: The `guitarhub_lib` crate may not export all needed types for integration tests. Mitigation: check `lib.rs` exports, add `pub` visibility as needed.

### Ready for Proposal

**Yes** — the orchestrator should present this 4-sprint roadmap to the user for approval. Each sprint is independently scoped and deliverable within 3.5 hours. The user should confirm:
1. Priority ordering (backend-first vs frontend-first)
2. Whether windowed pagination (Sprint 3) is acceptable vs true virtual scrolling
3. Whether E2E tests should be run in CI or only locally for now
