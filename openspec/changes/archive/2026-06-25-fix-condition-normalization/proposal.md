# Proposal: Fix Condition Normalization — Move Vocabulary Mapping to Rust Sanitize

## Intent

CHECK constraint violations on `products_meta.condition` because `sanitize()` only lowercases instead of mapping to the 4-value vocabulary. Reverb passes raw `condition_slug` values ("brand_new", "excellent"), GC has its own Python-side map — every source is a data integrity risk.

## Scope

### In Scope
- Add `normalize_condition()` in Rust `sanitize()` mapping arbitrary strings to 4-value vocabulary
- Simplify GC adapter: remove `_normalize_condition()` call, pass raw condition (keep sticker extraction)
- Update `guitarcenter-adapter/spec.md` with changed normalization contract
- Tests covering Reverb slug patterns, GC raw values, and edge cases

### Out of Scope
- Collection item condition vocabulary (`mint/excellent/good/fair/poor`)
- GC sticker extraction (open_box, blemished, restock) — stays in adapter
- Schema migration — CHECK constraint is correct

## Capabilities

### New Capabilities
- `condition-normalization`: maps arbitrary condition strings to `new/used/refurbished/unknown` in Rust's product sanitize pipeline, applying to all sources

### Modified Capabilities
- `guitarcenter-adapter`: normalization requirement shifts — adapter passes raw GC condition values; Rust sanitize handles vocabulary mapping. Sticker extraction retained.

## Approach

Add `normalize_condition(input: &str) -> String` as a pure function in `product.rs`, called from `sanitize()`. Heuristics:

| Input pattern | Mapped to |
|---------------|-----------|
| Exact match: "new", "used", "refurbished" | Direct |
| Reverb slugs: "brand_new" | new |
| Reverb slugs: "excellent", "great", "good", "fair", "poor" | used |
| GC raw: "Open Box", "Blemished" | new |
| GC raw: "Used > Excellent/Great/Good/Fair/Poor" | used |
| GC raw: "Restock" | refurbished |
| Other / empty | unknown |

GC adapter: replace `_normalize_condition()` with pass-through of raw `condition` field. Keep `_SKU_CONDITION_MAP` for sticker extraction only.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src-tauri/src/domain/product.rs` | Modified | Add `normalize_condition()`, update `sanitize()` |
| `scraper/adapters/guitarcenter.py` | Modified | Remove `_normalize_condition()`, pass raw condition |
| `openspec/specs/guitarcenter-adapter/spec.md` | Modified | Normalization req shifts to Rust side |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Over-mapping unknown words | Low | Unknown → "unknown" safe default |
| Broken GC stickers | Low | Sticker extraction stays in adapter |

## Rollback Plan

Revert GC adapter changes (restore `_normalize_condition()`). Rust mapping is additive-safe — `git revert <merge-commit>`.

## Dependencies

None

## Success Criteria

- [ ] Reverb "brand_new" → "new", passes CHECK constraint
- [ ] Reverb "excellent" → "used"
- [ ] GC raw "Used > Excellent" → "used"
- [ ] GC raw "Restock" → "refurbished", sticker "restock" still present
- [ ] All existing `sanitize()` tests pass unchanged
