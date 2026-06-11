# Proposal: Sprint 2 — Frontend Page Decomposition

## Intent

`+page.svelte` is an 800-line monolith (340 lines template/logic + 460 lines inline CSS) mixing search, dashboard rendering, collection stats, sync status, and app info. This sprint extracts three self-contained Svelte 5 rune-based components, moving each component's styles into scoped `<style>` blocks. The result: a readable page file under 200 lines, testable components, and zero behavior changes.

## Scope

### In Scope
- Extract `SearchPanel.svelte` — search input, filters, results grid, pagination, loading/empty states
- Extract `SyncStatusCell.svelte` — sync toast, price drop list, idle state
- Extract `CollectionStatsCell.svelte` — item count, total value, top item, gain/loss summary
- Migrate each component's CSS from the page `<style>` block into component-scoped `<style>` blocks
- Convert extracted component state from `$state` locals to `$props()` contracts
- Verify: `npm run build` passes, all routes render, no visual regressions

### Out of Scope
- Store migration (writable → runes) — deferred to a later sprint
- ProductCard IPC optimization — separate sprint
- Windowed pagination — separate sprint
- AppInfoCell, RecentSearchesCell, StatCells extraction — low complexity, not worth extracting now

## Capabilities

### New Capabilities
- `search-panel`: SearchPanel component contract — props, events, internal state, search/filter/results rendering
- `sync-status-cell`: SyncStatusCell component contract — sync result display, price drop list, idle state
- `collection-stats-cell`: CollectionStatsCell component contract — stats display, gain/loss formatting, reactive updates

### Modified Capabilities
None — this is a pure structural refactor. Existing capability requirements (collection-ui, search-filters-ui, sync-service) are preserved unchanged; only the rendering location moves from inline to component.

## Approach

Extract one component at a time, verify after each:

1. **SearchPanel** (largest extraction, ~180 lines of logic + ~130 lines CSS): Move search state (`query`, `results`, `total`, `page`, `loading`, `error`, `searched`), `search()` function, search bar, results grid, load-more, and all search-related styles. Page passes `filterStore` and `dashboardStats` as props.
2. **SyncStatusCell** (~40 lines logic + ~50 lines CSS): Move sync toast, drop list rendering, idle state. Receives `syncResult` store value as prop.
3. **CollectionStatsCell** (~30 lines logic + ~30 lines CSS): Move collection stats rendering, gain/loss calculation display. Receives `collectionStore` value and `calculateCollectionGainLoss` result as props.
4. **Final cleanup**: Remove dead CSS from `+page.svelte`, verify line count < 200.

Each extraction follows: create component → wire props → replace inline markup with `<Component />` → move CSS → build → visual check.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src/routes/+page.svelte` | Modified | Reduced from 800 to <200 lines; imports 3 new components |
| `src/lib/components/SearchPanel.svelte` | New | Search, filter, results, pagination component |
| `src/lib/components/SyncStatusCell.svelte` | New | Sync status and price drop display |
| `src/lib/components/CollectionStatsCell.svelte` | New | Collection stats and gain/loss display |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Store coupling requires prop drilling | Medium | Pass store values as props; components stay stateless where possible |
| CSS scope leakage after extraction | Low | Use component-scoped `<style>` blocks; Svelte auto-scopes by default |
| Dark mode styles missed during CSS move | Low | Copy `@media (prefers-color-scheme: dark)` rules into each component's style block |

## Rollback Plan

Each component extraction is an independent commit. Revert any single commit to restore that section inline. Full rollback: `git revert` the 4 commits (3 extractions + cleanup) in reverse order.

## Dependencies

None — all dependencies already exist in the project.

## Success Criteria

- [ ] `+page.svelte` is under 200 lines
- [ ] All 3 new components use Svelte 5 runes (`$props()`, `$state`, `$derived`)
- [ ] `npm run build` passes with zero errors
- [ ] All routes render correctly with no visual regressions
- [ ] Dark mode styles preserved in all extracted components
- [ ] No inline CSS remains in `+page.svelte` for extracted sections
