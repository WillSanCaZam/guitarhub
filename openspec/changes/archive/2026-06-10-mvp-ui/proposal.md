# Proposal: MVP UI — TypeScript Strict + Search Filters

## Intent

Two deferred items from the MVP review: (1) 5 of 9 Svelte components lack `lang="ts"`, making all props/state `any` and masking type errors at compile time. (2) The search page hardcodes null filters despite the Rust backend supporting full filtering and sorting — users have no way to narrow results.

## Scope

### In Scope
- Add `lang="ts"` and typed interfaces to 5 untyped components
- Add search filter controls (category, price range, source, sort order) to `+page.svelte`

### Out of Scope
- New Rust backend filter fields (existing `SearchFilters` + `SortOrder` are sufficient)
- CollectionView or collection route (already typed)
- Dark mode for ProductDetail (deferred)

## Capabilities

### New Capabilities
- `search-filters-ui`: Interactive filter bar on the search page — category dropdown, price min/max inputs, source select, sort order picker

### Modified Capabilities
None — TypeScript strict is type enforcement only, no behavior change. Search filters are a new UI capability; the search-service already supports the contract.

## Approach

**Phase 1 — TypeScript strict**: For each of 5 components, add `lang="ts"`, define a local `interface Props` (following `DashboardCell.svelte` pattern), type all `$state()` and `invoke()` results. Preserve all existing behavior.

**Phase 2 — Search filters**: Add a collapsible filter bar below the search input. Category dropdown populated from distinct `products_meta.category` values via a new `get_categories` invoke. Price range as two number inputs. Source as a text input or dropdown. Sort order as radio/select. Each change triggers `search(true)` (reset + re-search).

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src/lib/components/Settings.svelte` | Modified | Add `lang="ts"`, type state + invoke results |
| `src/lib/components/PriceChart.svelte` | Modified | Add `lang="ts"`, type `sku`, `windowDays`, `points`, `error` |
| `src/lib/components/PriceBadge.svelte` | Modified | Add `lang="ts"` (props already destructured) |
| `src/lib/components/ProductCard.svelte` | Modified | Add `lang="ts"`, type `product`, `priceInsight` |
| `src/lib/components/ProductDetail.svelte` | Modified | Add `lang="ts"`, type `product` prop |
| `src/routes/+page.svelte` | Modified | Add filter controls, wire state into `invoke()` |
| `src/lib/types/search.ts` | Modified | Add `get_categories` return type or `Category` type |
| `src-tauri/src/commands/dashboard_command.rs` | Modified | Add `get_categories` Tauri command (if not existing) |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Adding `lang="ts"` surfaces pre-existing type errors | Med | Fix errors inline; do NOT change runtime behavior |
| `get_categories` returns stale/empty set | Low | Fall back to static category list from known product taxonomy |
| Filter controls break existing search UX | Low | Keep existing search bar unchanged; filters are additive above results |

## Rollback Plan

Revert each component's `lang="ts"` addition individually. Remove filter bar markup and restore hardcoded null filters. Each phase is independently revertible.

## Dependencies

- Tauri IPC command `get_categories` (to be added in `dashboard_command.rs`)
- Follows existing `invoke` + TS type mirror pattern from `search.ts`

## Success Criteria

- [ ] `npm run check` passes with zero type errors after all changes
- [ ] All 5 components compile under Svelte strict TypeScript mode
- [ ] Search filters produce different results than unfiltered search
- [ ] Filter state persists across pagination (loading more pages)
