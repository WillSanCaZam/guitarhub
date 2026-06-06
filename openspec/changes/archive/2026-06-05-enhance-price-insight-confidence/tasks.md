# Tasks: Enhance Price Insight with Confidence Scoring

**Change**: enhance-price-insight-confidence
**Date**: 2026-06-05
**Phase**: sdd-tasks
**Strategy**: single-pr | **Strict TDD**: enabled

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~170 |
| 400-line budget risk | Low |
| Chained PRs recommended | No |
| Suggested split | single PR |
| Delivery strategy | single-pr |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: pending
400-line budget risk: Low

### Work Units

| Unit | Goal | PR | Notes |
|------|------|----|-------|
| 1 | Full feature (backend + frontend) | PR 1 | base: main; ~170 LoC; tests incl. |

## Phase 1: Backend — Pure Confidence Logic (TDD)

- [ ] **1.1 [RED]** 10+ failing `compute_confidence_*` tests in `src-tauri/src/commands/price_command.rs` tests mod — quantity (cnt<30→0, cnt=60→25, cnt=90→50), recency (days=0→25, days≥7→0, negative clamp→25), sources (0/1→5, 2→10, 3+→15), stability (CoV<0.05→10, CoV≥0.20→0, avg=0 guard→10), B1/B2/B3, zero-data, stale.
- [ ] **1.2 [GREEN]** Add 6 `pub const`s in `price_command.rs` (`MIN_DATA_POINTS=30`, `RECENT_DATA_DAYS=7`, `STABLE_COV_THRESHOLD=0.05`, `VOLATILE_COV_THRESHOLD=0.20`, `CONFIDENCE_TIER_HIGH=80`, `CONFIDENCE_TIER_MEDIUM=50`). Implement `compute_confidence(cnt_30d, days_since_last, source_count, price_range_30d, avg_30d) -> u8` (pure, sum weighted, clamp 0-100). `cargo test` green.

## Phase 2: Backend — SQL Aggregates (TDD)

- [ ] **2.1 [RED]** Failing tests in `src-tauri/src/repository/price_history.rs` for new `PriceInsightRow` fields `last_recorded_at: i64`, `source_count_30d: i64`, `max_30d: f64`, `avg_30d: f64` — empty SKU = zeros; multi-source distinct; `last_recorded_at` = newest.
- [ ] **2.2 [GREEN]** Extend `get_insight` SQL: `MAX(recorded_at) AS last_recorded_at`, `MAX(CASE WHEN recorded_at >= ?2 THEN price END) AS max_30d`, `COUNT(DISTINCT CASE WHEN recorded_at >= ?2 THEN source_id END) AS source_count_30d`, 30-day `AVG` as `avg_30d` (keep `avg_90d`). Add 4 struct fields. 2.1 tests pass.

## Phase 3: Backend — Wire Command (TDD)

- [ ] **3.1 [RED]** Tests in `price_command.rs`: `get_price_insight_cmd_returns_confidence_in_payload` (≥30 points, 3 sources, stable → confidence ≥80), `stale_data_yields_low_confidence` (14d-old → recency=0), `under_threshold_returns_confidence_zero` (cnt<30 → quantity=0).
- [ ] **3.2 [GREEN]** Add `pub confidence: u8` to `PriceInsight`. Compute `days_since_last = (now - last_recorded_at) / 86_400`, `price_range_30d = max_30d - min_30d`. Call `compute_confidence(...)` at 3 non-`None` return sites. `make test` green; 8 existing + 3.1 tests pass.

## Phase 4: Frontend — TypeScript Types

- [ ] **4.1** Create `src/lib/types/price.ts` (SPDX GPL-3.0-or-later): `PriceInsight` interface (6 fields: `level: 'green'|'amber'|'hidden'`, `pct`, `current_price`, `min_30d`, `avg_90d`, `confidence: number /* 0-100 */`), `export type ConfidenceTier = 'high'|'medium'|'low'`. Verify with `npx tsc --noEmit`.

## Phase 5: Frontend — Badge UI

- [ ] **5.1** `PriceBadge.svelte`: add `confidence: number = 0` prop, `getTier(c)` helper (≥80 high / ≥50 med / else low), 3-dot scale (`•••`/`••○`/`•○○`) with `aria-hidden="true"`, tier-tinted. Replace `title` with 3-line tooltip (Confidence: NN% (Tier) / N points · S sources · last Dd ago / Min 30d $X.XX · Avg 90d $Y.YY · Current $Z.ZZ). `aria-label` includes `Confidence NN%, <tier>`. `ProductCard.svelte`: pass `confidence={priceInsight.confidence}` (1 line). `npm run build` green.

## Out of Scope

SQL migrations, popovers, threshold-tuning UI, scraper/search/sync changes, `main.rs` registration, `serde(default)` (additive only).

## Order & Acceptance

1 → 2 → 3 (backend RED-GREEN) → 4 (types) → 5 (UI). Each task = one session. ~170 LoC, single PR. `make test` green (181+ existing + ~13 new); `confidence ∈ [0, 100]`; 3-tier dots + 3-line tooltip + aria-label; 6 `pub const`s; no schema/migration/dep changes.
