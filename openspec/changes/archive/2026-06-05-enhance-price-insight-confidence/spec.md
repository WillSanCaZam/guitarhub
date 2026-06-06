# Spec Index — enhance-price-insight-confidence

> **Change**: enhance-price-insight-confidence
> **Date**: 2026-06-05
> **Phase**: sdd-spec

This change is split into three capability specs. All three are NEW — no prior top-level spec exists for any of these domains.

## Capabilities

| Capability | File | Status | Requirements | Scenarios |
|------------|------|--------|-------------|-----------|
| `price-insight` | [specs/price-insight/spec.md](specs/price-insight/spec.md) | New | 7 | 10 (B1–B10) + per-requirement tables |
| `ui` | [specs/ui/spec.md](specs/ui/spec.md) | New | 6 | 13 (prop, dots, tooltip, aria, hidden, ProductCard) |
| `frontend-types` | [specs/frontend-types/spec.md](specs/frontend-types/spec.md) | New | 4 | 10 (file, confidence type, level union, mirror completeness) |

## Coverage Summary

- **Happy paths**: covered in all three specs (B1/B2/B3, dots render, type mirror exists)
- **Edge cases**: zero data (B4), under threshold (B5), stale data (B6), single source (B7), volatility (B9/B10)
- **Error states**: `confidence = 0` when no data; hidden level suppresses badge; `aria-label` carries tier for AT users
- **Contract stability**: `confidence` is additive on `PriceInsight` — serde tolerates missing field in any cached state

## Out of Scope (per proposal)

- New SQL migrations
- Confidence threshold tuning UI
- Custom popovers for tooltips (native `title=` only)
- Price prediction / AI / ML features
- Components other than `PriceBadge` and `ProductCard`

## Next Phase

`make test` must pass with the existing 8 `price_command.rs` tests still green. The 5 new confidence tests (one per tier + zero-data + 30-day-stale) are the new contract surface.
