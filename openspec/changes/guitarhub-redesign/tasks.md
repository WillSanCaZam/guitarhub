# Tasks: GuitarHub Redesign

## Review Workload Forecast

Decision needed before apply: Yes
Chained PRs recommended: Yes
Chain strategy: pending
400-line budget risk: High

| Unit | Goal | PR | Base |
|------|------|----|------|
| 1 | Design system tokens + global CSS + font loading | PR 1 | master |
| 2 | Navbar + ProductCard rewrite | PR 2 | PR 1 |
| 3 | FilterBar + SearchPanel | PR 3 | PR 2 |
| 4 | Wishlist, Dashboard, Collection pages | PR 4 | PR 3 |

## Phase 1: Design System Base

- [ ] 1.1 Rewrite `tokens.css`: obsidian/graphite/amber/fuzz palette, display/body/mono fonts, spacing-1–12, radius, shadows. ~80 lines
- [ ] 1.2 Update `typography.css`: Syne headings, Inter body, JetBrains Mono. ~40 lines
- [ ] 1.3 Update `app.html`: Google Fonts preconnect + link for Syne/Inter/JetBrains Mono, font-display=swap. ~8 lines
- [ ] 1.4 Replace hardcoded colors in `dashboard.css` and `page.css` with token vars. ~20 lines
- [ ] 1.5 Grep+replace hardcoded hex colors in all `.svelte` files with token vars. ~60 lines

## Phase 2: Navbar + Routing

- [ ] 2.1 Redesign `Sidebar.svelte`: SVG icons, `/collection` link, wishlist badge, Syne amber logo. ~60 lines
- [ ] 2.2 Simplify `src/routes/+layout.svelte`: remove legacy header, keep sync button, add collection link. ~40 lines
- [ ] 2.3 Verify all routes accessible from nav (manual check).

## Phase 3: ProductCard

- [ ] 3.1 Verify `url` field in `RawProduct` (already exists — no change). 0 lines
- [ ] 3.2 Rewrite `ProductCard.svelte`: 16:10 image + shimmer skeleton, condition badge, Syne title, mono price, store link, wishlist toggle. ~200 lines
- [ ] 3.3 Add `open_url` Tauri command if missing (`src-tauri/src/commands/`). ~15 lines

## Phase 4: FilterBar

- [ ] 4.1 Rewrite `FilterBar.svelte`: remove collapse toggle, always visible, category chips (amber active), styled sort, filter pills with ×. ~200 lines

## Phase 5: SearchPanel

- [ ] 5.1 Add recent searches as clickable chips below search bar. ~30 lines
- [ ] 5.2 Replace spinner with 6-8 shimmer skeleton cards during loading. ~40 lines
- [ ] 5.3 Add CSS stagger animation (50ms delay increment, <300ms total). ~20 lines

## Phase 6: Wishlist Page

- [ ] 6.1 Rewrite `wishlist/+page.svelte`: header with count/value, ProductCard grid, empty state SVG + CTA. ~100 lines

## Phase 7: Dashboard Bento

- [ ] 7.1 Redesign `+page.svelte`: 7 cells (Search, Stats KPIs, Sync, Featured Deal, Collection preview). Remove 3 cells. ~100 lines
- [ ] 7.2 Rewrite `DashboardCell.svelte`: remove glassmorphism, graphite bg, amber glow hover, SVG icon prop. ~80 lines

## Phase 8: Collection Page

- [ ] 8.1 Rewrite `collection/+page.svelte`: stats header, grid/list toggle, value delta, CSV export. ~120 lines

## Phase 9: QA & Polish

- [ ] 9.1 Verify dark mode (prefers-color-scheme) on all routes. Manual.
- [ ] 9.2 Verify responsive at ~1024px: sidebar, bento grid, card reflow. Manual.
- [ ] 9.3 Verify focus-visible on all interactive controls. Manual.
- [ ] 9.4 Verify ProductCard store link opens browser. Manual.
- [ ] 9.5 Run `npm run check` — fix TS/Svelte errors.
- [ ] 9.6 Run `make test` — fix regressions.

**Total estimated lines**: ~1200–1600
