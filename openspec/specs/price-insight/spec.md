# Price Insight Specification

> **Status**: New capability
> **Change**: enhance-price-insight-confidence

## Purpose

Govern the `PriceInsight` IPC payload and the server-side confidence scoring that quantifies how trustworthy the green/amber price claim is. The capability defines the `confidence: u8` field, the four-factor `compute_confidence()` algorithm, tier cutoffs (High / Medium / Low), and the named `pub const` thresholds that keep the scoring tunable. The capability covers the `PriceHistoryRepo::get_insight` SQL aggregates that feed the scoring function and the `get_price_insight_cmd` that returns the payload to the frontend.

## Requirements

### Requirement: PriceInsight payload MUST include a `confidence: u8` field

The system MUST add a new `confidence: u8` field to the `PriceInsight` struct in `src-tauri/src/commands/price_command.rs`. The field MUST be in the inclusive range `[0, 100]` and MUST be serialized on every `Some(PriceInsight)` return site (green, amber, hidden).

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Confidence serialised | `get_price_insight` resolves with `Some(...)` | Tauri IPC delivers payload | Frontend receives `confidence: <0-100>` |
| Confidence is 8-bit unsigned | Any input | Field assigned | Value clamped to `[0, 100]` |
| Field present on hidden level | `cnt_30d < 30` | `get_price_insight` returns `Some(hidden)` | Payload still carries `confidence` (may be 0) |

### Requirement: Confidence MUST be computed from 4 weighted factors

The system MUST compute confidence in a pure function `compute_confidence(cnt_30d, days_since_last, source_count, price_range_30d, avg_30d) -> u8` with the following weights:

| Factor | Weight | Formula |
|--------|--------|---------|
| Quantity | 50 | `0` when `cnt_30d < 30`; linear `(cnt_30d.min(90) - 30) / 60 * 50` from 30 to 90 points |
| Recency | 25 | `(1.0 - clamp(days_since_last, 0, 7) / 7.0) * 25`; 0 at ≥7 days |
| Source diversity | 15 | 5 for 0–1 sources, 10 for 2, 15 for 3+ |
| Stability | 10 | `CoV = price_range_30d / avg_30d`; 10 when `CoV < 0.05`, linear to 0 at `CoV >= 0.20` |

The final score MUST be the sum rounded to the nearest integer and clamped to `[0, 100]`.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| All factors max | cnt=90, days=0, sources=3+, CoV=0 | compute | `confidence = 100` |
| Only quantity | cnt=30, days=14, sources=1, CoV=0.5 | compute | `confidence = 0` (quantity=0, recency=0, source=5, stability=0 → 5) |
| Zero data | cnt=0, days=∞, sources=0, range=0 | compute | `confidence = 0` (defensive) |

### Requirement: Confidence MUST map to three named tiers

The system MUST classify `confidence` into tiers:

| Tier | Range | Visual encoding |
|------|-------|-----------------|
| High | `>= 80` | Green dot row (•••) |
| Medium | `50..=79` | Amber dot row (••○) |
| Low | `< 50` | Grey dot row (•○○) |

The tier is a derived label, not a stored field; it is recomputed in the UI from `confidence`.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| High boundary | `confidence = 80` | Tier check | "High" |
| Medium lower | `confidence = 50` | Tier check | "Medium" |
| Low upper | `confidence = 49` | Tier check | "Low" |

### Requirement: Hidden-data invariant — `cnt_30d < 30` collapses quantity to zero

The system MUST set the quantity factor to 0 whenever `cnt_30d < 30`. The badge is hidden by the existing classification gate, but the `confidence` field on the payload is computed without shortcuts.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Under threshold | `cnt_30d = 25` | compute | Quantity contribution = 0; total ≤ 25 |
| Exactly 30 | `cnt_30d = 30` | compute | Quantity contribution = 0; total ≤ 25 |
| Plenty of data | `cnt_30d = 60` | compute | Quantity contribution = 25 (linear) |

### Requirement: Zero-data safety — confidence MUST be 0 when no data exists

When the SKU has zero rows in `price_history`, the system MUST return `confidence = 0`. The `level` in this case is `hidden` (existing behaviour) and the badge is not rendered.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| No rows | `get_price_insight("UNKNOWN")` | SQL returns no row | `Ok(None)` (existing) — no `confidence` exposed |
| current_price missing | Row exists, no `current_price` | `get_price_insight` | `Ok(None)` (existing) |
| current_price present, no 30d points | Edge: latest point >30d old | compute | `confidence = 0`, `level = "hidden"` |

### Requirement: Thresholds MUST be named `pub const` in `price_command.rs`

The system MUST export the following constants from `src-tauri/src/commands/price_command.rs`:

| Constant | Value | Purpose |
|----------|-------|---------|
| `MIN_POINTS_FOR_INSIGHT` | `30` | `cnt_30d` floor for the green/amber gate |
| `QUANTITY_FLOOR` | `30` | Lower bound of the quantity linear ramp |
| `QUANTITY_CEILING` | `90` | Upper bound of the quantity linear ramp |
| `RECENCY_FLOOR_DAYS` | `7` | Recency factor reaches 0 at this many days |
| `STABILITY_LOW_COV` | `0.05` | CoV below this yields full 10 stability points |
| `STABILITY_HIGH_COV` | `0.20` | CoV at or above this yields 0 stability points |
| `TIER_HIGH_MIN` | `80` | Confidence ≥ this is "High" |
| `TIER_MEDIUM_MIN` | `50` | Confidence ≥ this (and < 80) is "Medium" |

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Constants compile | `cargo build` | Compile | `pub const`s exported and reachable from tests |
| Tuning is local | Future change wants a different cutoff | Edit one constant | Single source of truth, no scattered magic numbers |

### Requirement: Tier display scenarios (B1–B10)

The following scenarios are derived from the exploration's edge-case table and MUST all pass under the scoring formula above.

| ID | Scenario | Given | When | Then |
|----|----------|-------|------|------|
| B1 | High confidence | 60 points in 30d, last 1d ago, 3 sources, low CoV | compute | `confidence >= 80` |
| B2 | Medium confidence | 45 points, last 3d ago, 2 sources | compute | `50 <= confidence <= 79` |
| B3 | Low confidence | 30 points, last 6d ago, 1 source | compute | `confidence < 50` |
| B4 | Zero data | no rows | invoke | `confidence = 0`, `level = "hidden"` (or `None`) |
| B5 | Under threshold | 25 points in 30d | compute | quantity factor = 0, total ≤ 25 |
| B6 | Stale data | last point 14d ago | compute | recency factor = 0 |
| B7 | Single source | `source_count = 1` | compute | source factor = 5 |
| B8 | Multi source | `source_count = 3` | compute | source factor = 15 |
| B9 | Stable prices | `CoV < 0.05` | compute | stability factor = 10 |
| B10 | Volatile prices | `CoV > 0.20` | compute | stability factor = 0 |

## Out of Scope

- Recomputing historical confidence (live only)
- User-tunable thresholds (hard-coded `pub const` for now)
- Confidence threshold tuning UI
- Custom popovers for tooltips (native `title=` only)
- Price prediction / AI / ML features
- SQL schema migrations (all data already in `price_history`)
- Components other than `PriceBadge` and `ProductCard`
