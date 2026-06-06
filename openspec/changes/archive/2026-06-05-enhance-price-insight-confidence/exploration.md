# Exploration: enhance-price-insight-confidence

**Change name**: `enhance-price-insight-confidence`
**Date**: 2026-06-05
**Phase**: sdd-explore

---

## Current State

### Data flow

```
SQLite (price_history)
  └── PriceHistoryRepo.get_insight(sku)        [1 SQL round-trip, conditional aggregates]
        └── PriceInsightRow { min_30d, avg_90d, cnt_30d, current_price }
              └── commands/price_command.rs::get_price_insight_cmd
                    ├── gates: current_price.is_some()  → returns None
                    ├── gates: row.cnt_30d < 30         → "hidden"
                    ├── green: current_price <= min_30d * 1.05
                    ├── amber: current_price >= avg_90d * 1.20
                    └── else: "hidden"
                          └── PriceInsight { level, pct, current_price, min_30d, avg_90d }
                                └── Tauri IPC: get_price_insight(sku)
                                      └── ProductCard.svelte (onMount invoke)
                                            └── PriceBadge.svelte (level, pct props)
```

### What the system computes today (and what it doesn't)

| Signal | Available today? | Source |
|---|---|---|
| `current_price` | yes | subquery `ORDER BY recorded_at DESC LIMIT 1` |
| `min_30d` | yes | `MIN(CASE WHEN recorded_at >= -30d THEN price END)` |
| `avg_90d` | yes | `AVG(CASE WHEN recorded_at >= -90d THEN price END)` |
| `cnt_30d` | yes | `COUNT(CASE WHEN recorded_at >= -30d THEN 1 END)` |
| `last_recorded_at` | **no** | would need `MAX(recorded_at)` |
| `source_diversity_30d` | **no** | would need `COUNT(DISTINCT source_id)` in 30d window |
| `cnt_90d` | **no** | would need a 90d count column |
| `stddev_30d` (or CoV) | **no** | SQLite has no SQRT; needs Rust aggregation or `AVG(p^2)-AVG(p)^2` SQL trick |
| `max_30d` (range) | **no** | easy SQL add |
| Window coverage (point density vs. expected) | **no** | derived: `cnt_30d / 30.0` (avg points per day) |

### Existing UI behaviour (PriceBadge.svelte)

- Two states only: `green` ("✓ Good price") and `amber` ("↑ Above average").
- `level === 'hidden'` → ProductCard.svelte skips rendering the badge entirely.
- `title=` (browser-native tooltip) currently says only "near 30-day low" or "above 90-day average" — no numeric detail.
- `aria-label` mentions `pct%` but not min/avg/current.
- No confidence indicator exists in any form.

### Existing spec coverage

- `openspec/specs/structured-errors/spec.md` only requires typed errors for `get_price_insight`.
- `openspec/changes/archive/price-intelligence-phase-3/price-intelligence-specs.md` defines the **Capability: price-insight** spec with REQ-PI-1..6 (currently not promoted to a top-level spec under `openspec/specs/`). That change is archived — the capability is not yet governed by a top-level spec.
- **No `openspec/specs/price-insight/` directory exists today.** This change should create it.

### Tests already in place (Rust)

`src-tauri/src/commands/price_command.rs` has 8 unit tests covering: green, amber, hidden, boundary cases, no-data, empty-sku. They are the contract we must keep green.

---

## Confidence Factors (proposal)

A confidence score should answer: *"How much should I trust this badge's claim?"*

| Factor | Weight | What it measures | Data source | Implementation cost |
|---|---|---|---|---|
| **Quantity** | 50 / 100 | More points in 30d window → higher confidence | `cnt_30d` (already available) | 0 — no SQL change |
| **Recency** | 25 / 100 | How fresh is the latest data point | `MAX(recorded_at)` (new) | 1 SQL column add |
| **Source diversity** | 15 / 100 | Multiple sources = less likely to be a single-store anomaly | `COUNT(DISTINCT source_id WHERE recorded_at >= -30d)` (new) | 1 SQL column add |
| **Stability** | 10 / 100 | Low variance → min/avg are meaningful | stddev or `MAX-MIN` range in 30d (new) | 1 SQL column add (use `MAX-MIN` as cheap proxy) |

**Rationale for weights**:
- Quantity dominates (50%) because the existing classification already requires `cnt_30d >= 30`; volume of data is the single strongest signal of reliability.
- Recency (25%) matters because a 3-month-old data point shouldn't drive a "good price" badge today.
- Source diversity (15%) protects against one scraper being broken or stuck on stale data.
- Stability (10%) is a tiebreaker — high variance is normal for some markets (vintage gear), so we don't penalize it heavily.

---

## Scoring Formula

```rust
fn compute_confidence(
    cnt_30d: i64,
    days_since_last: i64,        // (now - max_recorded_at) / 86400
    source_count: i64,
    price_range_30d: f64,        // max_30d - min_30d; 0 when stddev=0
    avg_30d: f64,                // for CoV
) -> u8 {
    // 1) Quantity: linear ramp from 0 (cnt<30) to 50 (cnt>=90)
    let q = if cnt_30d < 30 { 0.0 }
            else { (cnt_30d.min(90) - 30) as f64 / 60.0 * 50.0 };

    // 2) Recency: 25 at 0d, 0 at 7d+
    let r = (1.0 - (days_since_last.clamp(0, 7) as f64 / 7.0)) * 25.0;

    // 3) Source diversity: 5 (1), 10 (2), 15 (3+)
    let s = match source_count {
        0 | 1 => 5.0,
        2 => 10.0,
        _ => 15.0,
    };

    // 4) Stability: coefficient of variation over 30d
    //    CoV = stddev/mean ≈ range/(2*sqrt(3)*mean) for uniform; use range/mean as cheap proxy
    //    CoV < 5% → 10, 5-20% → linear, 20%+ → 0
    let cov = if avg_30d > 0.0 { price_range_30d / avg_30d } else { 0.0 };
    let st = if cov < 0.05 { 10.0 }
             else if cov < 0.20 { (1.0 - (cov - 0.05) / 0.15) * 10.0 }
             else { 0.0 };

    let total = q + r + s + st;
    total.round().clamp(0.0, 100.0) as u8
}
```

**Tier mapping** (for UI):
- `80..=100` → "High"
- `50..=79`  → "Medium"
- `0..=49`   → "Low"

**SQL change** to `get_insight` (additive — no schema migration):

```sql
SELECT
    MIN(CASE WHEN recorded_at >= ?2 THEN price END) AS min_30d,
    AVG(CASE WHEN recorded_at >= ?2 THEN price END) AS avg_30d,
    COUNT(CASE WHEN recorded_at >= ?2 THEN 1 END) AS cnt_30d,
    MAX(CASE WHEN recorded_at >= ?2 THEN price END) AS max_30d,
    MAX(recorded_at) AS last_recorded_at,
    COUNT(DISTINCT CASE WHEN recorded_at >= ?2 THEN source_id END) AS source_count_30d,
    (SELECT price FROM price_history
     WHERE sku = ?1 ORDER BY recorded_at DESC LIMIT 1) AS current_price
FROM price_history
WHERE sku = ?1
```

`avg_30d` (30d, not 90d) is needed for the stability CoV calc — adding it is a one-line change.

---

## UI Changes

### PriceBadge.svelte (additive)

New optional prop: `confidence: number = 0`. Render logic:
- If `level === 'hidden'` → still no badge (unchanged).
- Else: existing badge + small confidence dot/ring next to it (3-dot scale) AND enriched tooltip.

Visual sketch:
- **High (≥80)**: `✓ Good price •●●` (green dot row)
- **Medium (50-79)**: `✓ Good price •●○` (amber dot row)
- **Low (<50)**: `✓ Good price •○○` (grey dot row)

Tooltip (`title=` attribute, native browser) for both green & amber:
```
Confidence: 78% (Medium)
45 data points · 2 sources · last 2 days ago
Min 30d: $X.XX  |  Avg 90d: $Y.YY  |  Current: $Z.ZZ
```

`aria-label` updated to embed confidence: `"Good price, 2% above 30-day low. Confidence 78%, medium."`

### ProductCard.svelte (additive)

Pass `confidence={priceInsight.confidence}` to `<PriceBadge>`. No structural changes — one prop added.

### Types

`src/lib/types/` does **not** yet have a `price.ts` mirror of `PriceInsight`. The existing code relies on Tauri auto-typing via `invoke('get_price_insight', ...)`. Two options:
1. **No new types file** — keep current pattern, TS just trusts the IPC payload.
2. **Add `src/lib/types/price.ts`** — declare `PriceInsight` interface with the new `confidence: number` field. This is the more disciplined path and matches the existing `search.ts` pattern.

**Recommendation**: option 2 (type file). Low cost, future-proofs the IPC contract, prevents silent drift between Rust and TS.

---

## Edge Cases

| Case | Current behaviour | Desired behaviour with confidence |
|---|---|---|
| No data at all | Returns `None` → no badge | Unchanged (badge hidden) |
| `cnt_30d < 30` | level = "hidden" → no badge | Unchanged (badge hidden) |
| `cnt_30d == 30` exactly | level = "green" or "amber" | confidence quantity = 0, total ≤ 25 → "Low" tier |
| `cnt_30d == 90` | full | confidence quantity = 50, total could be 90+ |
| 1 source, 100 points, 0 days old, all stable | confidence ≈ 50+25+5+10 = 90 → "High" | OK |
| 3+ sources, 100 points, 0 days old, all stable | confidence ≈ 50+25+15+10 = 100 → "High" | OK |
| 1 source, 100 points, **30 days old last point** | confidence ≈ 50+0+5+10 = 65 → "Medium" | Recency gates this correctly |
| 1 source, 100 points, 0 days old, **price varies 50%** | confidence ≈ 50+25+5+0 = 80 → "High" borderline | OK — variance is normal for some SKUs |
| All prices equal | `price_range_30d = 0` → CoV = 0 → stability = 10 | Handled by the `cov == 0.0` branch |
| `cnt_30d >= 30` but `current_price` is from > 30 days ago | Edge case: `current_price` is from `(SELECT ... LIMIT 1)` (most recent of all time) | Confidence recency will reflect staleness |
| `source_count_30d == 0` (all data older than 30d but `current_price` exists) | Should not happen — if `current_price` is set, that point is the most recent; if it's in the 30d window, sources > 0 | Defensive `match` returns 5 (treat as 1 source) |

**Important UX question**: when confidence is "Low" but level is still "green" or "amber", the badge will look the same color. We should:
- **Option A**: dim/desaturate the badge when confidence < 50.
- **Option B**: keep color, let the dots + tooltip carry the warning.
- **Option C**: hide badge entirely when level is green/amber but confidence < 30 (very low trust).

**Recommendation**: Option B. Color is the primary signal (good/bad price); confidence is meta-information. Hiding at low confidence removes information the user wants.

---

## Fix Scope Estimate

| File | Change | LoC |
|---|---|---|
| `src-tauri/src/repository/price_history.rs` | Extend `get_insight` SQL: add 4 columns (`max_30d`, `last_recorded_at`, `source_count_30d`, `avg_30d`); add fields to `PriceInsightRow` | ~30 |
| `src-tauri/src/commands/price_command.rs` | Add `confidence: u8` to `PriceInsight`; new `compute_confidence()` fn (pure, testable); wire into all 4 return sites; ~5 new tests (each tier, 0-data, all-stable) | ~80 |
| `src/lib/types/price.ts` (new) | Mirror `PriceInsight` interface | ~15 |
| `src/lib/components/PriceBadge.svelte` | Add `confidence` prop, render dots, enriched tooltip, updated aria | ~40 |
| `src/lib/components/ProductCard.svelte` | Pass `confidence` prop | ~3 |
| **Total** | | **~170 lines** |

Well under the **400-line PR review budget** — single PR is appropriate. No chained/stacked PR needed.

---

## Risks

1. **SQL cost**: 4 additional aggregate expressions on a `price_history` table that's already indexed by `(sku, recorded_at)`. Still single-row aggregate, expected < 50ms. **Mitigation**: add a benchmark assertion (existing spec already requires < 50ms on 10k rows).

2. **No source diversity in old data**: rows inserted before migration 004 default `source_id = ''`. A SKU with 100 points all from `''` will get `source_count_30d = 1` → confidence penalty 10 points. **Mitigation**: this is correct behavior (treat empty source as 1 source) and existing data is rare for the price_history table.

3. **Frontend type drift**: if `src/lib/types/price.ts` is not created, TS may not catch missing `confidence` field. **Mitigation**: create the type file (recommendation above).

4. **Backward compat**: callers that JSON-deserialize `PriceInsight` and ignore unknown fields will work. Callers that strict-parse may break. **Mitigation**: add `confidence` as a new field with a default; serde will serialize it for all callers.

5. **Tooltip overflow**: long tooltip strings in narrow product cards may be truncated by the browser. **Mitigation**: keep tooltip to 3 lines max, consider switching to a custom popover in a future change (not in this scope).

6. **Confidence "Low" with green/amber**: could confuse users. **Mitigation**: documented in UX recommendation (Option B — keep color, use dots + tooltip).

7. **Hard-coded thresholds in scoring formula**: 30, 90, 7 days, 5%/20% CoV are chosen by intuition. **Mitigation**: constants in `price_command.rs` with named variables (`MIN_POINTS_FOR_INSIGHT = 30`, `RECENCY_FLOOR_DAYS = 7`, etc.) so they're easy to tune.

---

## Affected Areas

- `src-tauri/src/repository/price_history.rs` — extend SQL + struct
- `src-tauri/src/commands/price_command.rs` — add field, scoring fn, tests
- `src/lib/components/PriceBadge.svelte` — confidence UI
- `src/lib/components/ProductCard.svelte` — pass new prop
- `src/lib/types/price.ts` — new type file (recommended)

**No** changes needed to: SQL migrations (no schema change), Tauri command registration in `main.rs`, `PriceChart.svelte`, scraper, search service, sync service, export service, settings.

---

## Recommendation

This is a **small, contained enhancement** that adds rich meta-information to an already-working feature. The single-PR path fits comfortably under the 400-line budget. The biggest design decision is the **scoring formula** — that belongs in `sdd-propose` / `sdd-design` for the user to confirm.

**Concrete proposal to take to `sdd-propose`**:
- Single PR, no schema migration, additive change to `PriceInsight` payload
- Confidence: 0-100 with 4 weighted factors (quantity 50, recency 25, sources 15, stability 10)
- Tiered display (High/Medium/Low) via 3-dot indicator + enriched tooltip
- Keep current color-coded badge; add confidence as meta-signal (do not hide or recolor)
- New `src/lib/types/price.ts` to mirror the Rust struct

---

## Ready for Proposal

**Yes.** The orchestrator can launch `sdd-propose` next. The user should be asked to confirm:
1. The 4 factors and their weights (quantity 50 / recency 25 / sources 15 / stability 10).
2. The tier cutoffs (80+ High, 50-79 Medium, <50 Low).
3. The display choice (Option B: keep badge color, add confidence dots + enriched tooltip).
4. Whether to add `src/lib/types/price.ts` mirror file.

If the user pushes back on weights, the constants are easy to retune.
