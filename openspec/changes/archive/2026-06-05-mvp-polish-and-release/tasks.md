# Tasks: MVP Polish and Release (v0.1.0)

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~620 (excl. LICENSE boilerplate) |
| 400-line budget risk | High |
| Chained PRs recommended | Yes |
| Suggested split | PR 1 → PR 2 → PR 3 → PR 4 |
| Delivery strategy | ask-on-risk |
| Chain strategy | stacked-to-main |

Decision needed before apply: Yes
Chained PRs recommended: Yes
Chain strategy: stacked-to-main
400-line budget risk: High

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | UI Polish | PR 1 | Settings, DashboardCell, PriceBadge, +page logic; ~180 lines |
| 2 | Frontend Tests | PR 2 | 4 new vitest files; mocks per spec; ~280 lines |
| 3 | CI/CD & Docs | PR 3 | CLI flags, workflow, CHANGELOG, README, LICENSE, SPDX; ~160 lines |
| 4 | Collection Dashboard | PR 4 | Cell 8 markup, formatting, color classes; ~40 lines |

## Phase 1: UI Polish

- [x] 1.1 `src/lib/components/DashboardCell.svelte`: add `.glassmorphism` class and `@supports` fallback for `backdrop-filter`
- [x] 1.2 `src/lib/components/Settings.svelte`: wire save button to `saveAll()` with `saving`/`saved` state and 2s feedback
- [x] 1.3 `src/lib/components/PriceBadge.svelte`: add optional props, build 3-line `title`/`aria-label`, omit missing fields
- [x] 1.4 `src/routes/+page.svelte`: add `$derived` `collectionGainLoss` from `$collectionStore.items`
- [x] 1.5 `src/routes/+page.svelte` CSS: set grid gap `16px`, cell radius `12px`, cell `focus-visible` ring

## Phase 2: Frontend Tests

- [x] 2.1 Create `src/lib/components/__tests__/ProductCard.test.ts`: render, badge, add action, hide in collection
- [x] 2.2 Create `src/lib/components/__tests__/Settings.test.ts`: form fields, save feedback, disabled state
- [x] 2.3 Create `src/lib/components/__tests__/PriceChart.test.ts`: data rendering, empty state, loading state
- [x] 2.4 Create `src/routes/__tests__/+page.test.ts`: 9 cells, gain/loss, loading/empty states

## Phase 3: CI/CD & Docs

- [x] 3.1 `scraper/cli.py`: add `--validate-input`/`--input-dir`; reuse `CatalogFile`; exit 0/1 with logs
- [x] 3.2 `.github/workflows/scrape.yml`: add `validate-input` step before publish boundary
- [x] 3.3 `CHANGELOG.md`: rewrite `[Unreleased]` to dated `v0.1.0` with summary and link
- [x] 3.4 `README.md`: add v0.1.0 badge linking to releases
- [x] 3.5 Create `LICENSE` at repo root with full GPL-3.0 text
- [x] 3.6 Add `SPDX-License-Identifier: GPL-3.0-or-later` header to all `.rs` and `.py` source files

## Phase 4: Collection Dashboard

- [x] 4.1 `src/routes/+page.svelte` Cell 8: render gain/loss with sign prefix, locale format, and green/red/neutral color class
- [x] 4.2 Hide gain/loss when collection is empty; preserve empty state message
- [x] 4.3 Verify `/collection` route preserves per-item gain/loss and remove action updates dashboard reactively
