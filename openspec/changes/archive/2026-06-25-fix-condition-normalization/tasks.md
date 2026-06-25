# Tasks: Fix Condition Normalization — Move Vocabulary Mapping to Rust

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~130 |
| 400-line budget risk | Low |
| Chained PRs recommended | No |
| Suggested split | Single PR |
| Delivery strategy | single-pr |
| Chain strategy | pending |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: pending
400-line budget risk: Low

## Phase 1: Rust — normalize_condition infrastructure

- [x] 1.1 Add `pub fn normalize_condition(condition: &str) -> &str` in `src-tauri/src/domain/product.rs` with mapping per spec table (empty→unknown, starts_with("used >")→used, exact matches→new/used/refurbished, else unknown)
- [x] 1.2 Update `RawProduct::sanitize()` to deserialize `specs_json`, save original condition as `condition_original` if key absent, call `normalize_condition()` on `self.condition`, and reserialize `specs_json`
- [x] 1.3 Remove empty-condition fallback from sanitize (now handled by normalize_condition)

## Phase 2: Python — GC adapter simplification

- [x] 2.1 Remove `_CONDITION_MAP` dict from `scraper/adapters/guitarcenter.py`
- [x] 2.2 Rewrite `_normalize_condition()` to return raw `condition.lvl1`/`condition.lvl0` for non-skuCondition items; for skuCondition (2/3/11) return semantic name (Restock/Open Box/Blemished) and keep sticker extraction
- [x] 2.3 Remove `_SKU_CONDITION_MAP` (normalized values no longer needed); replace with inline sticker mapping

## Phase 3: Rust tests

- [x] 3.1 Add `#[cfg(test)] mod tests` table-driven test for `normalize_condition()` covering all spec table entries (new/brand_new/mint/open box/blemished, used/excellent/great/good/fair/poor/used >, refurbished/restock, empty/unknown/other)
- [x] 3.2 Add test for `sanitize()` preserving `condition_original` in specs_json when absent
- [x] 3.3 Add test for `sanitize()` leaving existing `condition_original` intact

## Phase 4: Python tests — update assertions

- [x] 4.1 Update `TestConditionNormalization` in `scraper/tests/unit/test_guitarcenter.py`: non-skuCondition tests assert raw condition value (e.g. "New", "Used > Excellent") instead of normalized
- [x] 4.2 Update skuCondition tests: assert condition is semantic name ("Open Box", "Blemished", "Restock") instead of normalized value
- [x] 4.3 Verify `condition_original` and sticker assertions remain correct
