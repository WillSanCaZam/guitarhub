# Tasks: MVP UI — TypeScript Strict + Search Filters

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~650-750 |
| 400-line budget risk | High |
| Chained PRs recommended | Yes |
| Suggested split | PR 1 (Rust backend) → PR 2 (TS strict + filter store) → PR 3 (FilterBar UI) |
| Delivery strategy | ask-on-risk |
| Chain strategy | pending |

Decision needed before apply: Yes
Chained PRs recommended: Yes
Chain strategy: pending
400-line budget risk: High

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | Rust backend: extend SearchFilters + search.rs WHERE + get_categories | PR 1 | Tests included. Base: main |
| 2 | TS strict: type 5 components + filter store + search.ts mirror | PR 2 | Tests for store. Base: main (parallel to PR 1) |
| 3 | FilterBar UI: component + +page.svelte wiring + URL sync | PR 3 | Tests for FilterBar. Base: main |

## Phase 1: Rust Backend (WU1)

- [x] 1.1 **RED** — Add test for `condition` / `listing_currency` round-trip in `SearchFilters`; write failing rust test
- [x] 1.2 **GREEN** — Add `condition: Option<String>`, `listing_currency: Option<String>` to `SearchFilters` in `product.rs`
- [x] 1.3 **RED** — Write integration test for `FtsSearchService::search` filtering by condition + currency
- [x] 1.4 **GREEN** — Add `AND m.condition = ?` / `AND m.currency = ?` to WHERE builder in `search.rs` + dynamic bind params
- [ ] 1.5 **RED** — Write test for `get_categories` returning distinct sorted categories
- [ ] 1.6 **GREEN** — Add `get_categories` command in `dashboard_command.rs` with `SELECT DISTINCT category FROM products_meta ORDER BY category`
- [ ] 1.7 — Mirror new fields in `src/lib/types/search.ts` — add `condition`, `listing_currency` to `SearchFilters` interface

## Phase 2: TypeScript Strict + Filter Infrastructure (WU2)

- [x] 2.1 — Type `Settings.svelte`: `lang="ts"`, `interface Props`, typed `$state<>()`, typed `invoke<>()`
- [x] 2.2 — Type `PriceChart.svelte`: Props for `sku`/`windowDays`, typed `$state` for `points`/`error`, typed `invoke<PricePoint[]>`
- [x] 2.3 — Type `PriceBadge.svelte`: Props interface for `level`/`pct`/`confidence`/`cnt_30d`/etc
- [x] 2.4 — Type `ProductCard.svelte`: Props for `product`/`inCollection`, typed `$state` for `imageData`/`priceInsight`, typed `invoke<>`
- [x] 2.5 — Type `ProductDetail.svelte`: Props interface for `product`
- [x] 2.6 — Create `src/lib/stores/filter.ts`: `FilterState` interface (category, price_min, price_max, source, condition, listing_currency, sort), writable store
- [x] 2.7 — Add URL sync helpers: `syncFiltersToUrl()` (debounced 300ms) + `restoreFiltersFromUrl()` (parse `location.search` → FilterState)

## Phase 3: FilterBar UI (WU3)

- [x] 3.1 — Create `FilterBar.svelte`: collapsible controls for category (dropdown), price min/max (number inputs), condition (select), currency (select), sort (select)
- [x] 3.2 — Wire FilterBar into `+page.svelte`: import component, read `filterStore` in `search()` invoke call replacing hardcoded nulls
- [x] 3.3 — Add URL restoration on mount: call `restoreFiltersFromUrl()`, populate store; search uses filters from store
- [x] 3.4 — Add clear/reset buttons — individual (×) per filter + "Reset all" — each triggers search via store update

## Phase 4: Testing

- [x] 4.1 — Unit test filterStore URL round-trip (parse → serialize → assert identity)
- [x] 4.2 — Unit test `pageFromOffset` (unchanged — verify still passes)
- [x] 4.3 — svelte-check — 0 type errors in +page.svelte and all 5 typed components; 6 pre-existing infra errors remain
- [x] 4.4 — `cargo test` — verify all Rust tests pass including new filter field tests

## Phase 5: Verification

- [ ] 5.1 — Confirm `npm run check` passes with zero type errors
- [x] 5.2 — Confirm `cargo test` passes with existing + new tests
- [ ] 5.3 — Verify each component renders identically before/after typing
- [ ] 5.4 — Verify filter UI produces different results than unfiltered search
- [ ] 5.5 — Verify filter state persists in URL and restores on page load
