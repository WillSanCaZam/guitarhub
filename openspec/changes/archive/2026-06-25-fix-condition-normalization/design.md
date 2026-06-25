# Design: Fix Condition Normalization — Move Vocabulary Mapping to Rust

## Technical Approach

Shift vocabulary mapping from per-adapter Python code into Rust's `sanitize()` pipeline via a pure `normalize_condition()` function. The Python adapters pass raw condition strings; Rust normalizes to the 4-value vocabulary (`new/used/refurbished/unknown`). This eliminates CHECK constraint violations regardless of source.

## Architecture Decisions

### Decision: normalize_condition handles all incoming patterns

| Option | Tradeoff | Decision |
|--------|----------|----------|
| Match exact words only | Misses "Used > Excellent" (GC) and "brand_new" (Reverb) | Guard `starts_with("used >")` + exact word list |
| Keep mapping in adapter too | Duplicate logic, defeats purpose | Single source of truth in Rust |

**Rationale**: The function lowercases input first, then applies a pattern: (1) empty→unknown, (2) `starts_with("used >")`→used (GC hierarchical), (3) exact match against known words, (4) else unknown. A single pure function with no side effects.

### Decision: condition_original preservation in sanitize()

| Option | Tradeoff | Decision |
|--------|----------|----------|
| Adapter always sets it | GC already does; Reverb doesn't | Rust checks if missing, sets from raw condition BEFORE normalize |
| Rust always overwrites | Loses adapter's more precise original (e.g. "Open Box") | Only set if `condition_original` key absent in specs_json |

**Rationale**: `sanitize()` parses `specs_json` as `serde_json::Value`, checks for `condition_original` existence, sets it from pre-normalize condition if absent, and reserializes. GC adapter already writes `condition_original` with meaningful names ("Open Box", "Restock") — Rust preserves those.

### Decision: GC adapter keeps skuCondition-derived condition names

| Option | Tradeoff | Decision |
|--------|----------|----------|
| Pass raw `condition.lvl0/lvl1` unconditionally | Restock item has lvl1="New" → Rust normalizes to "new" instead of "refurbished" | Use skuCondition name for condition field (e.g. "Restock", "Open Box") |
| Pass only `condition.lvl0/lvl1` | Loses semantic condition from skuCondition codes | Keep sticker extraction + derive condition string from sku |

**Rationale**: For skuCondition items, the adapter passes the semantic name ("Restock", "Open Box", "Blemished") as the condition field. Rust then normalizes "restock"→"refurbished", "open box"→"new", etc. For non-skuCondition items, the adapter passes the raw `condition.lvl1`/`condition.lvl0` value. `condition_original` still preserves the human-readable form.

## Data Flow

```
Python Adapter                    Rust sanitize()
┌────────────────┐               ┌──────────────────────┐
│ Extract raw     │               │ Save raw condition    │
│ condition from  │  condition=   │ → condition_original  │
│ Algolia/Reverb  │ "brand_new"   │ (if not in specs)    │
│ → CatalogProduct│ ──────────→   │                       │
│ .condition      │               │ lower_case + trim    │
│                 │               │ → normalize("brand_new")│
│ specs_json has  │               │ → "new"              │
│ condition_orig  │               │                       │
│ (if adapter     │               │ CHECK constraint      │
│  provides it)   │               │ passes: "new" is valid│
└────────────────┘               └──────────────────────┘
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `src-tauri/src/domain/product.rs` | Modify | Add `normalize_condition()`, update `sanitize()` to call it + preserve original |
| `scraper/adapters/guitarcenter.py` | Modify | Remove `_CONDITION_MAP` and vocabulary mapping; keep sticker extraction; pass raw/sku-derived condition |
| `scraper/tests/unit/test_guitarcenter.py` | Modify | `TestConditionNormalization` asserts raw adapter output, not normalized |
| `src-tauri/src/domain/product.rs` (test module) | Modify | Add `#[cfg(test)]` tests for `normalize_condition()` mapping |

## Interfaces / Contracts

```rust
/// Normalize a condition string to the 4-value vocabulary.
/// Input should already be lowercased and trimmed.
fn normalize_condition(condition: &str) -> &str {
    match condition {
        "" => "unknown",
        s if s.starts_with("used >") => "used",
        "new" | "brand_new" | "mint" | "open box" | "blemished" => "new",
        "used" | "excellent" | "great" | "good" | "fair" | "poor" => "used",
        "refurbished" | "restock" => "refurbished",
        _ => "unknown",
    }
}
```

**GC adapter contract**: `_normalize_condition()` now returns `(raw_condition, specs_dict)`. No `_CONDITION_MAP` lookup. For skuCondition items, the condition string is derived from the sku condition code name (Restock/Open Box/Blemished).

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Rust unit | `normalize_condition()` mapping 1:1 with spec table | Table-driven: each (input → expected) pair |
| Rust unit | `sanitize()` preserves `condition_original` in specs_json | Create product with and without pre-existing `condition_original` |
| Python unit | GC adapter passes raw condition (no vocabulary mapping) | Assert `product.condition` equals raw Algolia value (e.g. "Used > Excellent") |
| Python unit | GC adapter still sets stickers from skuCondition | Assert sticker presence unchanged |

## Migration / Rollout

No migration required. Rust `normalize_condition()` is additive-safe — existing DB values that already match the vocabulary pass through unchanged. GC adapter change is the only rollback risk (covered by reverting `_normalize_condition()`).

## Open Questions

None.
