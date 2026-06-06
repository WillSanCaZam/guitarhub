# Archive Report: enhance-price-insight-confidence

> **Change**: enhance-price-insight-confidence
> **Date archived**: 2026-06-05
> **Final commit**: `5d2d58a` — `feat(insight): add confidence score to price insight with 3-dot UI`
> **PR strategy**: single-pr (170 LoC production; 623 LoC production + tests; 400-line budget risk: Low at planning; Medium at final due to TDD-mandated tests)

## Change Summary

Added a 0-100 confidence score to the existing price insight feature. The score is computed server-side from 4 weighted factors (quantity 50, recency 25, sources 15, stability 10) and surfaced in the frontend via a 3-dot scale (`•••` / `••○` / `••○`) and an enriched `aria-label`. The change is fully additive — no schema migration, no `main.rs` registration, no new dependencies.

## Test Status

- **Total**: 224/224 tests passing (181 pre-existing + 32 new from this change + 11 from follow-up clippy sync in `042e82d`)
- **New tests in this change** (32):
  - 24 `compute_confidence_*` unit tests in `src-tauri/src/commands/price_command.rs`
  - 5 SQL aggregate tests in `src-tauri/src/repository/price_history.rs`
  - 3 integration tests for `get_price_insight_cmd`
- **Quality gates**: clippy clean, `npm run build` green, `tsc --noEmit` clean

## Files Changed (commit 5d2d58a)

| File | Action | LoC delta |
|------|--------|-----------|
| `src-tauri/src/commands/price_command.rs` | Modified | +406 (6 consts, `compute_confidence`, `confidence: u8`, 27 tests) |
| `src-tauri/src/repository/price_history.rs` | Modified | +147 (4 SQL aggregates, 5 struct fields, 5 tests) |
| `src/lib/components/PriceBadge.svelte` | Modified | +59 (confidence prop, 3-dot scale, tooltip, aria) |
| `src/lib/components/ProductCard.svelte` | Modified | +2/-1 (pass confidence prop) |
| `src/lib/types/price.ts` | **New** | +24 (TypeScript mirror) |
| **Total** | | **+623 / -15** |

## Deviations from Design

1. **Tooltip shows tier only, not full 3-line data-quality tooltip** (v1)
   - Designed: `Confidence: NN% (Tier) / N points · S sources · last Dd ago / Min 30d $X · Avg 90d $Y · Current $Z`
   - Shipped: `Confidence: NN% (Tier)` only
   - Reason: the full tooltip requires `cnt_30d`, `source_count_30d`, and `last_recorded_at` on the IPC payload, which was not in the design.md `PriceInsight` contract. Widening the IPC contract is deferred to the follow-up change.
   - Severity: low. Dots + tier word still convey the meta-signal. 3-dot scale is the primary confidence carrier.

2. **Line budget overage: 623 LoC vs 400-line budget**
   - Production code: 168 LoC (on target for the 170-line forecast).
   - Overage: ~450 LoC from TDD-mandated tests (24 unit + 5 SQL + 3 integration + clippy sync).
   - Mitigation: tests are the contract surface; trimming them would weaken the regression barrier.

3. **`now_epoch` added to `PriceInsightRow`**
   - Not exposed on IPC; included only to share a single clock between the repo aggregate and the command scoring.
   - Defensive but unused by external consumers.

## Follow-Ups

| ID | Title | Reason | Owner |
|----|-------|--------|-------|
| F1 | **enhance-price-insight-full-tooltip** | Widen IPC `PriceInsight` to expose `cnt_30d`, `source_count_30d`, `last_recorded_at`; render the full 3-line data-quality tooltip per UI spec REQ-BADGE-3. | Next change |
| F2 | Confidence threshold tuning UI | User-tunable `TIER_HIGH_MIN` / `TIER_MEDIUM_MIN` constants via Settings. | Future |
| F3 | Custom popover for tooltip | Replace native `title=` with a real popover for narrow product cards. | Future |
| F4 | Pre-existing flaky test | `get_insight_returns_stats_within_windows` flakiness at day boundary. Not introduced by this change. | Backlog |
| F5 | `Makefile test-app` target | Invalid cargo syntax. Not introduced by this change. | Backlog |

## Specs Synced

| Domain | Action | Requirements | File |
|--------|--------|-------------|------|
| `price-insight` | **Created** (new top-level) | 7 | `openspec/specs/price-insight/spec.md` |
| `ui` | **Created** (new top-level) | 6 | `openspec/specs/ui/spec.md` |
| `frontend-types` | **Created** (new top-level) | 4 | `openspec/specs/frontend-types/spec.md` |

All three delta specs were promoted as full top-level specs (no prior main spec existed). Delta-format headers (`## ADDED Requirements`, `# Delta for X`) were normalized to the project's existing main-spec convention (`# {Domain} Specification`, `## Purpose`, `## Requirements`) used by `scraper`, `structured-errors`, `frontend-scaffolding`, and other archived capabilities.

## Capabilities Added

1. **`price-insight`** — Rust scoring algorithm, SQL aggregates, IPC contract, tier cutoffs, named `pub const` thresholds.
2. **`ui`** (badge scope) — `PriceBadge` 3-dot scale, tooltip, `aria-label`; `ProductCard` prop forwarding.
3. **`frontend-types`** — `src/lib/types/price.ts` mirror of `PriceInsight`, `ConfidenceTier` alias.

## Archive Location

```
openspec/changes/archive/2026-06-05-enhance-price-insight-confidence/
├── exploration.md
├── proposal.md
├── spec.md
├── specs/                    (3 domain specs preserved as archived deltas)
│   ├── price-insight/spec.md
│   ├── ui/spec.md
│   └── frontend-types/spec.md
├── design.md
├── tasks.md
└── archive-report.md         (this file)
```

## Source of Truth Updated

- `openspec/specs/price-insight/spec.md` — new top-level capability spec
- `openspec/specs/ui/spec.md` — new top-level capability spec (badge scope)
- `openspec/specs/frontend-types/spec.md` — new top-level capability spec

## SDD Cycle Complete

The change has been fully planned (explore → propose → spec → design → tasks), implemented under strict TDD (apply), verified (clippy + test + build + tsc), and now archived. Ready for the next change.
