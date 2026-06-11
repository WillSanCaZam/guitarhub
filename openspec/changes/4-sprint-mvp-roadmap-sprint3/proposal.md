# Proposal: Sprint 3 — Optimization & Stability

## Intent

Sync performance degrades linearly with catalog size — each product is INSERT OR REPLACE'd individually outside a transaction (`sync.rs:132-216`). For 1000+ products, sync takes ~10s. Simultaneously, the frontend renders all search results in the DOM without virtualization, causing scroll jank at 100+ items. This sprint fixes both bottlenecks.

## Scope

### In Scope
- **A. SQLite batch upserts**: Wrap product insertion loop in a transaction; add `batch_upsert_products()` to the repository trait
- **B. Virtual scrolling**: Implement windowed rendering for SearchPanel results list
- Verification: sync time benchmark, scroll performance with 1000+ results

### Out of Scope
- FTS5 query caching — deferred to Sprint 4
- Image cache concurrent download limits — deferred to Sprint 4
- SQLite connection pool tuning (`max_connections=1` is adequate for single-user desktop)

## Capabilities

### New Capabilities
None

### Modified Capabilities
- `sync-service`: Add `batch_upsert_products()` method to repository trait; wrap insertion phase in a SQLite transaction with rollback on failure
- `search-panel`: Add virtual scrolling to results rendering — only render visible viewport + buffer rows

## Approach

**Backend (sync.rs)**:
1. Add `batch_upsert_products(products: &[Product]) -> Result<u32, AppError>` to the `ProductRepository` trait
2. Implement in `SqliteProductRepository`: open transaction → batch INSERT OR REPLACE → commit on success, rollback on error
3. Update `CatalogSyncService::upsert_products` to call the batch method instead of the per-product loop
4. Price history writes remain inside the same transaction for atomicity

**Frontend (SearchPanel.svelte)**:
1. Evaluate Svelte 5-compatible virtual scrolling: `@tanstack/svelte-virtual` or lightweight custom windowing
2. Implement scroll container with fixed-height items, rendering only visible range + overscan buffer (~5 items)
3. Maintain existing filter/sort/pagination props unchanged

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src-tauri/src/services/sync.rs` | Modified | Replace per-product loop with `batch_upsert_products()` call |
| `src-tauri/src/repositories/product_repo.rs` | Modified | Add batch method to trait + SQLite impl with transaction |
| `src/lib/components/SearchPanel.svelte` | Modified | Replace full DOM render with virtualized list |
| `package.json` | Modified | Add virtual scrolling dependency (if external lib chosen) |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Transaction rollback loses all products on single-row error | Low | Log failed rows, use savepoints or skip-and-continue strategy |
| Virtual scrolling lib incompatible with Svelte 5 runes | Medium | Evaluate 2+ options in task phase; fall back to custom windowing |
| CSS grid layout breaks with virtualized items | Medium | Use fixed-height list layout for virtual scroll; keep grid for small result sets |

## Rollback Plan

Two independent commits (backend, frontend). Revert either without affecting the other:
- Backend: `git revert` the batch upsert commit restores the per-product loop
- Frontend: `git revert` the virtual scroll commit restores full DOM rendering

## Dependencies

None — no new infrastructure. Virtual scrolling library (if used) is a frontend npm dependency only. Zero-cost implication: no new services, no server costs.

## Success Criteria

- [ ] Sync of 1000 products completes in <3s (down from ~10s)
- [ ] Transaction rollback on error leaves DB in consistent state
- [ ] SearchPanel renders 1000+ results with smooth 60fps scrolling
- [ ] Existing search filters, sort, and pagination work unchanged with virtual scroll
- [ ] `cargo test` and `npm run build` pass with zero errors
