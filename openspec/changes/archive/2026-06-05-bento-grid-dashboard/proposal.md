# Proposal: Bento Grid Dashboard

## Intent

Replace the vertical-stack landing page with a modular bento grid that surfaces search, sync status, catalog stats, price alerts, and recent activity in a single glanceable layout. The current UI buries rich data behind scroll; a grid makes it visible.

## Scope

### In Scope
- CSS Grid bento layout (4-col desktop, stacked mobile)
- 9 dashboard cells: Search, Featured Deal, Catalog Stats, Sync Status, Price Alerts, Browse Categories, Wishlist Preview, Recent Searches, Search Results
- New Rust IPC: `get_catalog_stats`, `get_sync_state`, `get_wishlist_summary`
- New stores: `recentSearches` (localStorage), `dashboardPrefs`
- Mobile stacking via `@media (max-width: 768px)`

### Out of Scope
- Cell customization (hide/reorder/drag-and-drop)
- Settings relocation (stays inline bottom cell for now)
- CSS design tokens/variables (hardcoded hex remains)
- New routes (dashboard stays `/`)

## Capabilities

### New Capabilities
- `dashboard-layout`: CSS Grid bento composition, 9 cells, responsive stacking
- `catalog-stats`: Rust commands for product count, source count, category distribution, sync state
- `wishlist-summary`: Rust command + frontend preview cell for wishlist count and recent items

### Modified Capabilities
- `ui`: `+page.svelte` rewritten from vertical stack to bento grid; new dashboard cell components

## Approach

Progressive enhancement of existing `+page.svelte`. Keep search as the dominant action; wrap existing components and new cells in a CSS Grid container (`repeat(4, 1fr)`). Each cell is a scoped-style Svelte component receiving grid span classes. Search Results collapses to 0 height until a query is submitted. Mobile flattens to single column.

**2-PR delivery** (review budget >400 lines):
- PR 1: Grid layout + cells using existing data (Search, Featured Deal, Catalog Stats, Sync Status, Price Alerts, Recent Searches) — ~300 lines
- PR 2: New backend commands + remaining cells (Categories, Wishlist) + polish — ~260 lines

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src/routes/+page.svelte` | Modified | Vertical stack → bento grid wrapper |
| `src/lib/components/` | New | 8 dashboard cell components |
| `src/lib/stores/` | New | `recentSearches.ts`, `dashboardPrefs.ts` |
| `src-tauri/src/commands/` | New | `get_catalog_stats`, `get_sync_state`, `get_wishlist_summary` |
| `src-tauri/src/services/` | New | Aggregation queries for stats |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Review overload (~560 LoC) | High | Split into 2 chained PRs |
| Wishlist backend gap | Med | New IPC commands with tests in PR 2 |
| Mobile grid fragility | Low | Single `@media` rule, all spans reset to 1 |
| Empty-state complexity | Med | Each cell renders own empty state; no global blocker |
| IPC thundering herd on load | Low | Stagger cell mounting or lazy-load on visibility |

## Rollback Plan

Revert `+page.svelte` to previous vertical-stack version. Cell components and new Rust commands are additive — removing them does not break existing flows. No schema changes.

## Dependencies

- `wishlist` table (migration 006) — already exists
- `sync_state` table — already exists
- `products_meta` table — already exists

## Success Criteria

- [ ] Bento grid renders on `/` with all 9 cells positioned correctly
- [ ] Mobile viewport (<768px) stacks cells in single column
- [ ] `make test` passes (Rust + frontend tests)
- [ ] No regression in search, sync, or settings behavior
- [ ] PR 1 and PR 2 each stay under 400 changed lines
