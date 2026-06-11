# Design: docs-gaps — Dashboard Data Spec Alignment

## Technical Approach

No architecture changes. This is a documentation-only alignment: the spec at `openspec/changes/docs-gaps/specs/dashboard-data/spec.md` is corrected to match existing code. No code is created, modified, or removed. Each delta was verified against the actual Rust implementation in `src-tauri/src/`.

## Architecture Decisions

None. This is a pure documentation change with zero code impact. CLI entry points, IPC contracts, database schema, and component hierarchy are unaffected.

## Data Flow

Unaffected. All four commands already operate correctly in production — the spec was simply stale.

| Command | Actual Type (Code) | Previous Spec | Correction |
|---------|-------------------|---------------|------------|
| `get_total_products` | `Result<u32, AppError>` | `u64` | `u32` |
| `get_categories` | `Result<Vec<String>, AppError>` | Not documented | Added |
| `record_search` | `Result<()>` with ON CONFLICT upsert | Mentioned in passing | Full spec |
| `get_collection_stats` | `(u32, f64, Option<String>, f64)` incl. `top_item_value` | Missing `top_item_value` | Added |

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `openspec/changes/docs-gaps/specs/dashboard-data/spec.md` | Already modified by sdd-spec | Contains the 4 corrections — no further changes needed |

## Interfaces / Contracts

No interfaces change. The existing Rust types are the source of truth:

```rust
// src-tauri/src/repository/dashboard.rs
pub async fn get_total_products(&self) -> Result<u32, AppError>          // was: u64
pub async fn get_categories(&self) -> Result<Vec<String>, AppError>      // was: undocumented
pub async fn record_search(&self, query: &str) -> Result<(), AppError>   // was: only mentioned

// src-tauri/src/repository/collection.rs
pub struct CollectionStats {
    pub total_items: u32,
    pub total_value: f64,
    pub top_item_name: Option<String>,
    pub top_item_value: f64,       // was: missing from spec
}
```

## Verification Strategy

1. **Read the final spec** — compare each command's documented return type vs the actual Rust signature
2. **Scenario audit** — verify each Given/When/Then scenario matches actual test cases in `dashboard.rs` and `collection_command.rs`
3. **Regression check** — confirm pre-existing correct content (e.g. `get_wishlist_count`, `get_recent_searches`) was not altered
4. **No tests to run** — zero code changed

## Open Questions

None. All deltas have been cross-checked against the running codebase.
