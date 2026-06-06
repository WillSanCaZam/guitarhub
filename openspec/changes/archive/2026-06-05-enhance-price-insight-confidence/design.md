# Design: Enhance Price Insight with Confidence Scoring

**Change**: `enhance-price-insight-confidence`
**Date**: 2026-06-05
**Phase**: sdd-design

## Technical Approach

Add a server-computed `confidence: u8` (0-100) to the `PriceInsight` IPC payload via an extended SQL aggregate in `get_insight` and a pure `compute_confidence()` function. Surface the value in `PriceBadge.svelte` via a 3-dot scale and enriched tooltip. Fully additive — no schema migration, no behavior change for SKUs with <30 points. ~170 LoC across 5 files; single PR (well under 400-line review budget).

## Architecture Decisions

| # | Decision | Choice & Why |
|---|----------|--------------|
| 1 | Where to compute | **Rust** — IPC single-sourced; matches `PriceInsight` boundary |
| 2 | CoV proxy | **`range/mean`** — SQLite has no SQRT; ≈ stddev/mean for small samples |
| 3 | Tier UX | **Dots+tooltip** (Option B) — color = price claim, dots = trust warning |
| 4 | TS mirror | **Add `price.ts`** — matches `search.ts`; catches field drift |
| 5 | CoV guard `avg=0` | **`cov=0` branch** — defensive; unreachable in practice |
| 6 | Dot glyphs | **`•••`/`••○`/`•○○`** — no emoji font dependency |

## Data Flow

```
price_history (SQLite)
  └── PriceHistoryRepo.get_insight(sku)   [7 aggregates, 1 round-trip]
        └── PriceInsightRow { +max_30d, +avg_30d, +last_recorded_at, +source_count_30d }
              └── price_command::get_price_insight_cmd
                    ├── gates unchanged
                    └── compute_confidence(5 args) → u8   (pure, deterministic)
                          └── PriceInsight { ..., confidence: u8 }
                                └── IPC → ProductCard → <PriceBadge confidence={...}>
```

`compute_confidence` is **pure** — caller passes `days_since_last = (now - last_recorded_at) / 86_400` from existing `now`. No `epoch_seconds` inside the fn → trivially unit-testable.

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `src-tauri/src/repository/price_history.rs` | Modify | +4 SQL aggregates, +4 struct fields; extend `no_data_*` test |
| `src-tauri/src/commands/price_command.rs` | Modify | +`confidence: u8`; +6 consts; +`compute_confidence()`; wire 4 sites; +5 tests |
| `src/lib/types/price.ts` | **Create** | `PriceInsight` interface + `ConfidenceTier` type |
| `src/lib/components/PriceBadge.svelte` | Modify | `confidence` prop; `getTier()`; 3-dot scale; enriched tooltip; aria |
| `src/lib/components/ProductCard.svelte` | Modify | Pass `confidence={priceInsight.confidence}` (one line) |

## Contracts

**Rust `PriceInsight`**: add `pub confidence: u8` after `level`. 6 thresholds as `pub const` in `price_command.rs`:

```rust
pub const CONFIDENCE_TIER_HIGH: u8 = 80;
pub const CONFIDENCE_TIER_MEDIUM: u8 = 50;
pub const RECENT_DATA_DAYS: i64 = 7;
pub const MIN_DATA_POINTS: i64 = 30;
pub const STABLE_COV_THRESHOLD: f64 = 0.05;
pub const VOLATILE_COV_THRESHOLD: f64 = 0.20;
```

**SQL** (`get_insight`, additive — no schema change):

```sql
SELECT
    MIN(CASE WHEN recorded_at >= ?2 THEN price END)  AS min_30d,
    AVG(CASE WHEN recorded_at >= ?2 THEN price END)  AS avg_30d,       -- NEW (30d for CoV)
    COUNT(CASE WHEN recorded_at >= ?2 THEN 1 END)    AS cnt_30d,
    MAX(CASE WHEN recorded_at >= ?2 THEN price END)  AS max_30d,       -- NEW
    MAX(recorded_at)                                  AS last_recorded_at,         -- NEW
    COUNT(DISTINCT CASE WHEN recorded_at >= ?2 THEN source_id END) AS source_count_30d, -- NEW
    (SELECT price FROM price_history WHERE sku = ?1 ORDER BY recorded_at DESC LIMIT 1) AS current_price
FROM price_history WHERE sku = ?1
```

Existing `idx_price_history_sku_recorded (sku, recorded_at)` covers all new aggregates — no new index.

**TS `src/lib/types/price.ts`** (new): mirror of `PriceInsight` with `confidence: number`, plus `export type ConfidenceTier = 'high' | 'medium' | 'low'`.

## Testing Strategy

| Layer | Scope | Approach |
|-------|-------|----------|
| Rust unit | `compute_confidence` per factor | quantity bounds (0/full); recency (0 at ≥7d); sources (15 for 3+); stability (10/0 at CoV 5%/20%); cov=0 guard; clamp negative days |
| Rust integration | cmd returns confidence | Extend green/amber tests to assert `confidence > 80`; new `stale_data_yields_low_confidence` |
| Regression | All 8 existing tests stay green | Only 3 non-`None` return sites gain a field |
| Frontend | None | No vitest; TS strict catches prop mismatch; manual visual review |

**TDD order**: RED `compute_confidence_*` → GREEN impl+consts → RED `get_insight_*` aggregates → GREEN SQL+struct → RED `get_price_insight_cmd_*` → GREEN wire 4 sites.

## Migration / Rollout

No migration. `confidence` is additive; serde includes it in every response. Revert = revert 5 files. No DB migration, no new deps, no feature flag — frontend sees dots from the next page load. Existing 8 tests gate regression.

## Edge Cases (defensive guards)

- `current_price=None` → `Ok(None)` upstream; `compute_confidence` never called
- `cnt_30d < 30` → `level="hidden"` upstream; `confidence: 0` (field ignored)
- `source_count_30d == 0` → `match 0` arm returns 5
- `source_id = ''` (pre-migration-004) → `COUNT(DISTINCT '')=1` → 5 source pts (correct)
- `avg_30d == 0` → `if avg_30d > 0.0` guard → CoV=0 → stab=10
- `days_since_last < 0` (clock skew) → `clamp(0, 7)` → full recency
- All prices equal → `range=0` → CoV=0 → stab=10

## Open Questions

None. All decisions resolved in `sdd-propose` and `sdd-explore`.
