# Proposal: Enhance Price Insight with Confidence Scoring

## Intent

The `PriceBadge` shows "Good price" (green) or "Above average" (amber) with no signal of how much to trust the claim. A badge backed by 100 points from 3 sources is materially different from one at the 30-point minimum from a single scraper. Surface that distinction so users can weigh the price claim against its reliability.

## Scope

### In Scope
- `confidence: u8` (0-100) on `PriceInsight`, computed server-side
- 4-factor scoring: quantity (50) + recency (25) + source diversity (15) + stability (10)
- 3-tier UI (High ≥80 / Medium 50-79 / Low <50) via 3-dot scale + enriched tooltip
- New `src/lib/types/price.ts` mirroring the Rust struct

### Out of Scope
- Recomputing historical confidence (live only)
- Custom popover / new tooltip library (native `title=` only)
- User-tunable thresholds (hard-coded constants for now)
- SQL schema changes (all data already in `price_history`)
- Scraper, search, sync, export, settings, `main.rs` registration

## Capabilities

### New Capabilities
- `price-insight`: governs the `PriceInsight` IPC payload, confidence scoring algorithm, tier cutoffs, and badge UI. No top-level spec exists today; the `price-intelligence-phase-3` archive has loose specs but nothing under `openspec/specs/price-insight/`.

### Modified Capabilities
- None — this is the first top-level spec for the domain; nothing to delta against.

## Approach

**Backend (~110 LoC):** Extend `get_insight` SQL with 4 additive aggregates (`max_30d`, `last_recorded_at`, `source_count_30d`, `avg_30d` for stability CoV). Add a pure `compute_confidence()` fn. Thresholds in named `pub const`s. ~5 new unit tests cover each tier, 0-data, all-stable, single-source, and 30-day-stale cases. All 8 existing tests must stay green.

**Frontend (~60 LoC):** `PriceBadge.svelte` gains a `confidence` prop, renders a 3-dot scale (••• / •●○ / •○○) tinted by tier, exposes a 3-line tooltip (points, sources, days-since-last, min/avg/current). `ProductCard.svelte` passes the new prop. New `src/lib/types/price.ts` mirrors the Rust struct (matches existing `search.ts` pattern) to prevent TS drift.

**UX principle:** Color keeps its current meaning (the price claim). Confidence is meta-signal carried by dots + tooltip. We do NOT dim or hide the badge at low confidence — the user wants the claim, plus the warning.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src-tauri/src/repository/price_history.rs` | Modified | +4 SQL aggregates, +4 struct fields |
| `src-tauri/src/commands/price_command.rs` | Modified | +1 field, +1 fn, +5 tests |
| `src/lib/types/price.ts` | New | TS mirror of `PriceInsight` |
| `src/lib/components/PriceBadge.svelte` | Modified | Dots + tooltip + aria |
| `src/lib/components/ProductCard.svelte` | Modified | Pass `confidence` prop |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| 4 new aggregates slow `get_insight` | Low | Same single-row aggregate; existing spec requires < 50ms on 10k rows |
| Old rows with `source_id = ''` penalize diversity | Low | Treat empty source as 1 source (correct) |
| Low-confidence green/amber confuses users | Med | Tooltip explains; tuning knob is future work |
| Hard-coded thresholds may be wrong | Med | Named constants (`MIN_POINTS_FOR_INSIGHT = 30`, etc.) |
| Tooltip overflow on narrow cards | Low | Cap to 3 lines; custom popover is future work |

## Rollback Plan

`confidence` is **additive** on `PriceInsight`. Revert the 5 files; serde tolerates the missing field in any cached state. No schema migration to revert. No destructive operations. **Zero infra cost**: no new deps, no schema change.

## Dependencies

None new. Uses existing `serde`, `rusqlite`, Svelte 5 runes.

## Success Criteria

- [ ] `make test` passes (existing 181+ Rust tests + new tests all green)
- [ ] `PriceInsight.confidence ∈ [0, 100]` for all valid inputs
- [ ] 3-tier UI renders correctly (••• / •●○ / •○○) for fixture inputs
- [ ] Tooltip shows data points, sources, days-since-last, min/avg/current
- [ ] `aria-label` embeds confidence % and tier
- [ ] All 4 thresholds exported as `pub const` in `price_command.rs`
- [ ] No schema migration; no `main.rs` changes
