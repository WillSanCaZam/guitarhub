# Tasks: Sprint 2 — Frontend Page Decomposition

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 750–900 (across all extractions) |
| 400-line budget risk | High |
| Chained PRs recommended | Yes |
| Suggested split | PR 1 (SearchPanel) → PR 2 (SyncStatusCell + CollectionStatsCell) → PR 3 (CSS cleanup + verification) |
| Delivery strategy | ask-always (C1) |
| Chain strategy | feature-branch-chain |

Decision needed before apply: Yes
Chained PRs recommended: Yes
Chain strategy: feature-branch-chain
400-line budget risk: High

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | Extract SearchPanel.svelte with scoped styles | PR 1 | ~500 lines changed (pure code movement); size:exception candidate |
| 2 | Extract SyncStatusCell + CollectionStatsCell | PR 2 | ~290 lines changed; base = main after PR 1 merges |
| 3 | CSS cleanup, +page.svelte final reduction, verification | PR 3 | ~100 lines changed; base = main after PR 2 merges |

## Phase 1: Extract SearchPanel (largest, most complex)

- [x] 1.1 Create `src/lib/components/SearchPanel.svelte` with `$props()` contract: `filterStore` (writable), `dashboardStats` (readable), `collectionStore` (readable)
- [x] 1.2 Move search state (`query`, `results`, `total`, `page`, `pageSize`, `loading`, `error`, `searched`, `hasMore`) into SearchPanel as `$state`/`$derived` runes
- [x] 1.3 Move `search()`, `handleSearch()`, `handleKeydown()`, `loadMore()` functions into SearchPanel `<script>` block
- [x] 1.4 Move search bar template (lines 140–158), FilterBar usage, and results/error/empty/loading states (lines 163–204) into SearchPanel
- [x] 1.5 Move search-related CSS (`.search-bar` through `.load-more-btn`, including `@keyframes spin`) into SearchPanel scoped `<style>` block (~130 lines)
- [x] 1.6 Move search-related dark mode rules (`@media (prefers-color-scheme: dark)` for `.search-input`, `.load-more-btn`) into SearchPanel `<style>`
- [x] 1.7 Move search-related responsive rules (`.search-btn`, `.load-more-btn` min-height 44px) into SearchPanel `<style>`
- [x] 1.8 Replace inline search markup in `+page.svelte` with `<SearchPanel {filterStore} {dashboardStats} collectionStore={$collectionStore} />`
- [x] 1.9 Remove moved CSS and dead code from `+page.svelte`; verify `npm run build` passes

## Phase 2: Extract SyncStatusCell

- [x] 2.1 Create `src/lib/components/SyncStatusCell.svelte` with `$props()`: `drops` (array), `dropsSent` (number), `syncState` (string)
- [x] 2.2 Move sync toast template, drop list rendering, and idle state (lines 208–232) into SyncStatusCell
- [x] 2.3 Move sync CSS (`.sync-toast`, `.drop-list`, `.drop-item`, `.drop-sku`, `.drop-price`, `.drop-reason`, `.sync-idle`) into scoped `<style>` (~50 lines)
- [x] 2.4 Move sync dark mode rules (`.drop-item`, `.drop-sku`, `.sync-idle`) into SyncStatusCell `<style>`
- [x] 2.5 Replace inline sync markup in `+page.svelte` with `<SyncStatusCell drops={drops} {dropsSent} syncState={$syncResult?.state ?? 'idle'} />`
- [x] 2.6 Remove `drops`/`dropsSent` `$derived` from page script; remove moved CSS; verify build

## Phase 3: Extract CollectionStatsCell

- [x] 3.1 Create `src/lib/components/CollectionStatsCell.svelte` with `$props()`: `stats` (object), `items` (array), `loading` (boolean)
- [x] 3.2 Move collection stats template (lines 297–317) into CollectionStatsCell
- [x] 3.3 Move collection CSS (`.collection-stats`, `.top-item`, `.gain-loss-*`) into scoped `<style>` (~30 lines)
- [x] 3.4 Move collection dark mode rules (`.top-item`, `.gain-loss-*`) into CollectionStatsCell `<style>`
- [x] 3.5 Replace inline collection markup in `+page.svelte` with `<CollectionStatsCell stats={$collectionStore.stats} items={$collectionStore.items} loading={$collectionStore.loading} />`
- [x] 3.6 Remove `collectionGainLoss`/`collectionGainLossFormatted` `$derived` from page script; verify build

## Phase 4: CSS Cleanup & Final Reduction

- [x] 4.1 Audit remaining `+page.svelte` `<style>` block — remove any CSS rules now only used by extracted components
- [x] 4.2 Move shared utility styles (`.stat-value`, `.stat-label`) to a shared stylesheet (`src/lib/styles/shared.css`) or keep scoped if only used in page
- [x] 4.3 Verify `+page.svelte` is under 200 lines (script + template + remaining CSS)
- [x] 4.4 Remove unused imports from `+page.svelte` (e.g., `pageFromOffset`, `SearchResult` type if only used by SearchPanel)

## Phase 5: Verification

- [x] 5.1 Run `npm run build` — must pass with zero errors
- [x] 5.2 Visual check: search flow (type query → results render → load more → empty state → error state)
- [x] 5.3 Visual check: sync status cell shows drops and idle state correctly
- [x] 5.4 Visual check: collection stats cell shows items, value, gain/loss with correct color classes
- [x] 5.5 Visual check: dark mode — verify all extracted components render correctly under `prefers-color-scheme: dark`
- [x] 5.6 Visual check: responsive layout at 768px breakpoint — bento grid collapses, touch targets ≥ 44px
