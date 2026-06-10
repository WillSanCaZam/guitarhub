# Archive Report: mvp-ui

**Archived**: 2026-06-10
**Artifact store**: openspec

## Summary

MVP UI change — two deferred items from the MVP review addressed:
1. **TypeScript strict**: Added `lang="ts"`, typed interfaces, typed `$state()`, and typed `invoke()` to 5 Svelte components (Settings, PriceChart, PriceBadge, ProductCard, ProductDetail)
2. **Search filters**: Full interactive filter bar (FilterBar.svelte) with category, price range, source, sort order, condition, and listing currency controls. Filter state in Svelte store, synced with URL search params.

## Artifacts

| Artifact | Path |
|----------|------|
| Proposal | `openspec/changes/archive/2026-06-10-mvp-ui/proposal.md` |
| Design | `openspec/changes/archive/2026-06-10-mvp-ui/design.md` |
| Spec: search-filters-ui | `openspec/changes/archive/2026-06-10-mvp-ui/specs/search-filters-ui/spec.md` |
| Spec: ui-components-typing | `openspec/changes/archive/2026-06-10-mvp-ui/specs/ui-components-typing/spec.md` |

## Specs Synced to Main

| Domain | Action | Details |
|--------|--------|---------|
| search-filters-ui | Created | 6 requirements ADDED (filter controls, store, URL sync, clearable filters, search re-trigger) |
| ui-components-typing | Created | 4 requirements ADDED (lang="ts", typed state, typed invoke, preserve runtime) |

## Implementation Delivery

The change was implemented via 3 PRs:

| PR | Scope | Status |
|----|-------|--------|
| **PR 1** — Rust backend | Extended `SearchFilters` with `condition` + `listing_currency`, dynamic WHERE clauses | **Merged to master** |
| **PR 2** — TypeScript strict | Typed 5 components, created `filterStore` + URL helpers | **Merged to master** |
| **PR 3** — FilterBar UI | Created `FilterBar.svelte`, wired into `+page.svelte`, URL sync | **Branch `feat/filterbar-ui`, PR pending** |

## Test Results

| Suite | Count | Status |
|-------|-------|--------|
| Rust tests | 309 passing | ✅ |
| Vitest tests | 75 passing (up from 39) | ✅ |
| svelte-check | 6 pre-existing infra errors (type conflicts, not code issues) | ⚠️ Pre-existing |

## Source of Truth Updated

- `openspec/specs/search-filters-ui/spec.md` — new capability spec
- `openspec/specs/ui-components-typing/spec.md` — new type enforcement spec

## Verification Notes

No verify-report found in the change artifacts. Implementation was verified via CI test results provided in the close-out summary:
- All Rust and JS tests passing
- No regressions reported
- PR 3 is still pending review on `feat/filterbar-ui` branch

**Status**: Partial — core work is complete and merged; FilterBar UI PR is open awaiting review.
