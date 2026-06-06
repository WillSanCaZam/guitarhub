## Exploration: bento-grid-dashboard

### Current State

The GuitarHub frontend is a single-page app with one route (`/`). The current `+page.svelte` is a centered, max-width-960px vertical stack:

1. **Search section** — input + button, triggers `search_products` IPC
2. **Sync toast** — ephemeral banner when `$syncResult.drops` > 0
3. **Error/loading/empty/welcome states** — conditional blocks
4. **Results section** — CSS `auto-fill` grid (`minmax(280px, 1fr)`) of `ProductCard`s with "Load More" pagination
5. **Settings section** — `Settings` component rendered inline at the bottom

The app shell (`+layout.svelte`) provides a sticky nav bar (48px, dark) with "GuitarHub" title, Settings anchor, and Sync Catalog button. The sync operation is triggered from the nav and its result is stored in a Svelte writable store (`$lib/stores/sync.ts`).

**Existing components** (5 total, all in `$lib/components/`):
- `ProductCard.svelte` — image + name + brand + price + optional `PriceBadge`
- `PriceBadge.svelte` — green/amber price insight with confidence dots
- `PriceChart.svelte` — SVG multi-source price history (used inside `ProductDetail`)
- `ProductDetail.svelte` — name/brand/price header + `PriceChart`
- `Settings.svelte` — alert channel radio + config input + test button + data export

**Current styling approach**: scoped `<style>` blocks in each Svelte file, no CSS framework, no design system tokens. Colors are hardcoded hex values.

### Affected Areas

| Path | Why affected |
|------|-------------|
| `src/routes/+page.svelte` | Complete layout rewrite — from vertical stack to bento grid |
| `src/routes/+layout.svelte` | May need nav adjustments if dashboard cells replace nav actions |
| `src/lib/components/` | New dashboard cell components needed; existing components remain but may be wrapped |
| `src/lib/stores/` | New stores for recent searches, dashboard preferences |
| `src-tauri/src/commands/` | New IPC commands for catalog stats, wishlist count |
| `src-tauri/src/services/` | New query services for aggregation (total products, category counts) |

### Available Data Sources

**Already exposed via IPC** (can feed dashboard cells today):
- `search_products` — results + total count
- `sync_catalog` — sync result (drops, loaded, updated, state)
- `get_price_insight` / `get_price_history` — per-SKU data (for featured deal cells)
- `get_setting` / `save_setting` — user preferences

**Exists in DB but NOT exposed to frontend** (needs new IPC commands):
- `products_meta` total count → needs `get_catalog_stats`
- `products_meta` category distribution → needs `get_category_counts`
- `sync_state` table (last_synced, status per source) → needs `get_sync_state`
- `wishlist` table (migration 006) → needs `get_wishlist_count` / `list_wishlist`
- `price_drop_notifications` cooldown table → could show "recent alerts" history

**Not persisted anywhere** (needs frontend storage):
- Recent search queries — could use `localStorage` or Tauri `localDataDir` file
- Dashboard layout preferences / hidden cells — could use settings store

### Proposed Cell Layout

A 4-column CSS Grid desktop layout, stacking to single-column on mobile (< 768px).

```
Desktop (4 columns):
+----------------+----------------+----------------+----------------+
|  SEARCH BAR    |  SEARCH BAR    |  SEARCH BAR    |  SEARCH BAR    |  (1x4)
+----------------+----------------+----------------+----------------+
|  FEATURED      |  FEATURED      |  CATALOG       |  SYNC          |  (2x2)
|  DEAL          |  DEAL          |  STATS         |  STATUS        |
+----------------+----------------+----------------+----------------+
|  PRICE ALERTS  |  CATEGORIES    |  CATEGORIES    |  WISHLIST      |  (1x2 + 2x1)
+----------------+----------------+----------------+----------------+
|  RECENT        |  RECENT        |  BROWSE        |  BROWSE        |  (2x2)
|  SEARCHES      |  SEARCHES      |  RESULTS       |  RESULTS       |
+----------------+----------------+----------------+----------------+
```

**Cell definitions**:

1. **Search Bar** (`col-span-4`, `row: auto`) — Full-width search input. The primary action stays prominent at the top.
2. **Featured Deal** (`col-span-2`, `row-span-2`) — Hero card showing the most recent price drop from sync result, with mini price sparkline. Large to grab attention.
3. **Catalog Stats** (`col-span-1`, `row: auto`) — Total products, distinct sources, last sync time. Compact stat card.
4. **Sync Status** (`col-span-1`, `row: auto`) — Current sync state, progress if running, error if failed. Animated when active.
5. **Price Alerts** (`col-span-1`, `row: auto`) — Count of drops detected in last sync + link to settings.
6. **Browse Categories** (`col-span-2`, `row: auto`) — Horizontal chips of top categories (Electric Guitars, Bass, Amps, etc.) that trigger filtered search.
7. **Wishlist Preview** (`col-span-1`, `row: auto`) — Count + last 3 items. Needs new backend commands.
8. **Recent Searches** (`col-span-2`, `row: auto`) — Persisted last 5 queries, click to re-run. Needs localStorage.
9. **Search Results** (`col-span-4`, `row: auto`, collapsible) — The existing product grid, but only rendered after a search. Collapses to 0 height when not searched.

### CSS Grid Approach

Use a CSS Grid container with `grid-template-columns: repeat(4, 1fr)` and `gap: 16px`.

Each cell is a component receiving `class` props for grid placement:

```svelte
<!-- BentoGrid.svelte -->
<div class="bento">
  <SearchCell class="cell cell--search" />
  <FeaturedDealCell class="cell cell--featured" {drop} />
  <CatalogStatsCell class="cell cell--stats" {stats} />
  <SyncStatusCell class="cell cell--sync" {syncState} />
  <PriceAlertsCell class="cell cell--alerts" {drops} />
  <CategoriesCell class="cell cell--categories" {categories} />
  <WishlistCell class="cell cell--wishlist" {wishlist} />
  <RecentSearchesCell class="cell cell--recent" {searches} />
  {#if searched}
    <SearchResultsCell class="cell cell--results" {results} />
  {/if}
</div>

<style>
  .bento {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 16px;
    padding: 16px;
    max-width: 1200px;
    margin: 0 auto;
  }
  .cell {
    background: #fff;
    border-radius: 12px;
    padding: 16px;
    border: 1px solid #e5e5e5;
    /* Subtle shadow for depth */
    box-shadow: 0 1px 3px rgba(0,0,0,0.06);
  }
  .cell--search { grid-column: span 4; }
  .cell--featured { grid-column: span 2; grid-row: span 2; }
  .cell--stats { grid-column: span 1; }
  .cell--sync { grid-column: span 1; }
  .cell--alerts { grid-column: span 1; }
  .cell--categories { grid-column: span 2; }
  .cell--wishlist { grid-column: span 1; }
  .cell--recent { grid-column: span 2; }
  .cell--results { grid-column: span 4; }

  @media (max-width: 768px) {
    .bento { grid-template-columns: 1fr; }
    .cell { grid-column: span 1 !important; grid-row: span 1 !important; }
  }
</style>
```

**Why this approach**:
- No external CSS framework dependency (matches current stack)
- `repeat(4, 1fr)` is flexible and widely supported
- `span` classes allow easy reordering without touching markup
- Mobile `@media` query flattens everything to single column

### Mobile Strategy

**Stack, don't collapse.** On viewports < 768px:
- Grid becomes `grid-template-columns: 1fr`
- All `span` overrides reset to `span 1`
- Cells stack in DOM order
- Search stays first (critical)
- Featured Deal stays second (hero)
- Results expand to full width below

**Touch considerations**:
- Search bar needs larger tap targets (min 44px)
- Category chips should be horizontally scrollable (not wrapping)
- Featured deal card should be tappable to open product detail

### Approaches

1. **Full bento rewrite** — Replace `+page.svelte` entirely with dashboard, search becomes one cell
   - Pros: Clean slate, true dashboard experience
   - Cons: High risk, breaks existing user mental model, large diff
   - Effort: High (~500+ lines)

2. **Dashboard overlay** — Keep current page, add `/dashboard` route, make it the new landing
   - Pros: Non-breaking, can A/B test, preserves old search behavior
   - Cons: Maintains two layouts, more files to manage
   - Effort: Medium (~350 lines)

3. **Progressive enhancement** — Rewrite `+page.svelte` as bento, but keep search as the dominant cell; other cells are supplementary
   - Pros: Single landing page, natural evolution from current UI, all existing behavior preserved
   - Cons: Search results compete for space with dashboard cells
   - Effort: Medium (~400 lines)

**Recommendation**: Approach 3 (Progressive enhancement). The current page IS already a dashboard — it just lacks modular cells. Adding cells around the existing search→results flow is the most natural evolution and keeps the diff reviewable.

### Scope Estimate

| Area | Lines | Notes |
|------|-------|-------|
| `+page.svelte` rewrite | ~80 | Grid wrapper, cell composition |
| New dashboard cell components (8) | ~240 | ~30 lines each, scoped styles |
| New stores (`recentSearches`, `dashboardPrefs`) | ~40 | localStorage-backed |
| Rust: `get_catalog_stats` command | ~30 | COUNT, DISTINCT queries |
| Rust: `get_sync_state` command | ~20 | SELECT from sync_state |
| Rust: `get_wishlist_summary` command | ~30 | COUNT + recent rows |
| Tests (frontend + Rust) | ~120 | Component + command tests |
| **Total** | **~560** | |

**Review budget risk**: The ~560-line estimate exceeds the 400-line PR budget. Recommend slicing into two chained PRs:
- **PR 1**: Bento layout + existing-data cells (search, featured deal, sync status, recent searches frontend) — ~300 lines
- **PR 2**: New backend commands + wishlist/categories/stats cells — ~260 lines

### Risks

1. **Review overload** — Full change is ~560 lines; exceeds 400-line budget. Must chain PRs.
2. **Wishlist backend gap** — Wishlist table exists but has zero IPC surface. Adding it requires new commands + tests, expanding scope.
3. **Mobile grid fragility** — CSS Grid `span` overrides with `!important` in media queries can be brittle if future cells have custom spans.
4. **Empty state complexity** — A bento grid with 8 cells means 8 empty states. The current page has 2 (welcome, no results). Need coherent empty-state design.
5. **Settings relocation** — Settings currently lives at the bottom of `+page.svelte`. In a bento grid, it may need its own cell or a dedicated route.
6. **Performance** — Loading 8 cells concurrently on startup could cause IPC thundering herd. Need staggered or lazy loading.

### Open Questions

1. **Which cells are MUST vs NICE?** — If scope needs reduction, which cells can be deferred?
2. **Should search results live IN the grid or BELOW it?** — Current design has results as a full-width cell, but they could also replace the dashboard after search.
3. **Customization** — Should users hide/reorder cells? Adds significant complexity (drag-and-drop, persistence).
4. **Settings placement** — Keep inline bottom cell, move to modal, or dedicated `/settings` route?
5. **Color system** — Current hardcoded hex values. Should we introduce CSS custom properties (variables) as part of this change?

### Ready for Proposal

**Yes** — with the caveat that the orchestrator should ask the user:
- Which cells are MUST-have for MVP (recommend: Search, Featured Deal, Catalog Stats, Sync Status, Recent Searches)
- Whether to chain PRs (recommended: yes, 2 PRs)
- Settings placement preference
