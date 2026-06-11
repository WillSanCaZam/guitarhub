# Tasks: docs-gaps — Dashboard Data Spec Alignment

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~60-80 |
| 400-line budget risk | Low |
| Chained PRs recommended | No |
| Suggested split | Single PR |
| Delivery strategy | single-pr |
| Chain strategy | pending |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: pending
400-line budget risk: Low

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | Apply spec deltas | Single PR | Docs-only; verify + commit |

## Phase 1: Verify Spec Accuracy

- [x] 1.1 Read `src-tauri/src/commands/dashboard_command.rs` — confirm each command signature matches spec
- [x] 1.2 Read `src-tauri/src/repository/dashboard.rs` — confirm repo return types match spec
- [x] 1.3 Read `src-tauri/src/repository/collection.rs` — confirm `CollectionStats` struct has `top_item_value: f64`
- [x] 1.4 Cross-check both spec copies: `openspec/specs/dashboard-data/spec.md` (canonical) and `openspec/changes/docs-gaps/specs/dashboard-data/spec.md` (delta)

## Phase 2: Ensure Spec Is Complete

- [x] 2.1 Confirm `get_total_products` return type is `u32` in canonical spec (line 11)
- [x] 2.2 Confirm `get_categories` requirement is present with 2 scenarios
- [x] 2.3 Confirm `record_search` requirement is present with 3 scenarios
- [x] 2.4 Confirm `top_item_value: f64` is in `get_collection_stats` return type
- [x] 2.5 Confirm pre-existing specs (`get_wishlist_count`, `get_recent_searches`, 50ms guard) are untouched

## Phase 3: Commit

- [x] 3.1 Commit the canonical spec update with:
      `docs(spec): align dashboard-data spec with implementation`
      
      Adds get_categories + record_search requirements, fixes get_total_products type (u64→u32), adds top_item_value to get_collection_stats.
