# Design: Collection Value Tracker

## Technical Approach

Add a `collection_items` table via migration 008 and a concrete `CollectionRepo` following the existing `PriceHistoryRepo` pattern (no trait, `SqlitePool` field). Expose five IPC commands with core logic extracted for testability (`_cmd` suffix pattern). Value estimation uses a **batched SQL strategy** to stay under the 50ms spec requirement: one aggregate query for 90-day price-history averages and one fallback query for `products_meta.price`, merged in Rust. Frontend integrates collection stats into the dashboard store, adds an “Add to collection” button on `ProductCard` (conditional via a parent-level SKU `Set`), and introduces a `/collection` route with a grid view. Export service includes `collection_items.json`. Work is split into two review-safe PRs (≤350 LoC each).

## Architecture Decisions

| Decision | Choice | Alternatives | Rationale |
|---|---|---|---|
| Table name | `collection_items` | `collection` | Avoids collision with common SQL keyword and matches capability naming. |
| Value computation | Batch SQL aggregates + Rust merge | Per-item SQL queries | Keeps `get_collection_stats` under 50ms even with 100+ items. |
| Estimated-value fallback | `price_history` 90d avg → `products_meta.price` → `0.0` | `products_meta` first | History average reflects market better than a single list price; `products_meta` is only a backfill. |
| “Add” button conditional | Parent passes a `Set<string>` of collected SKUs | New `is_in_collection` IPC command | Avoids an extra round-trip per card; local collection size is small, so a frontend `Set` is acceptable. |
| Collection view surface | New `/collection` SvelteKit route | Modal in `+page.svelte` | Spec (REQ-CUI-3) explicitly requires a dedicated route. |
| Duplicate SKU policy | Allowed | Unique constraint | User may own two identical guitars; uniqueness would block legitimate use. |
| Partial update | Single `UPDATE … COALESCE(?1, field)` | Dynamic SQL builder | Simpler, no macro needed; MVP does not require NULLing fields. |
| Security boundary | Local SQLite only | Cloud sync | Collection data never leaves the device; no auth or encryption changes required. |

## Data Flow

```
User clicks "Add to collection"
  → ProductCard invokes add_to_collection(item)
  → collection_command.rs → CollectionRepo.add()
  → collection_items row inserted
  → +page.svelte refreshes dashboardStats & skuSet

User opens /collection
  → route loads CollectionView
  → get_collection() → batch SQL for estimated values
  → grid renders with gain/loss badges
```

## File Changes

| File | Action | Description |
|---|---|---|
| `src-tauri/src/repository/sqlite/migrations/008_collection_items.sql` | Create | Table, `CHECK` on `condition`, index on `sku` |
| `src-tauri/src/repository/collection.rs` | Create | `CollectionRepo`, `CollectionItem`, `CollectionItemInput`, `CollectionItemUpdates`, `CollectionStats` |
| `src-tauri/src/repository/mod.rs` | Modify | Add `pub mod collection;` |
| `src-tauri/src/commands/collection_command.rs` | Create | Five IPC commands + core `_cmd` functions |
| `src-tauri/src/commands/mod.rs` | Modify | Add `pub mod collection_command;` |
| `src-tauri/src/services/export_service.rs` | Modify | Add `collection_items.json` to ZIP; bump `file_count` to 4 |
| `src-tauri/src/main.rs` | Modify | Register collection commands in `generate_handler!` |
| `src/lib/stores/dashboard.ts` | Modify | Add `collectionCount`, `collectionValue`, `topItemName` |
| `src/lib/types/collection.ts` | Create | Frontend `CollectionItem` type |
| `src/lib/components/ProductCard.svelte` | Modify | Conditional “Add to collection” button |
| `src/lib/components/CollectionView.svelte` | Create | Grid with estimated value, purchase price, gain/loss |
| `src/routes/+page.svelte` | Modify | Wire Cell 8 to new stats, pass `skuSet` to `ProductCard` |
| `src/routes/collection/+page.svelte` | Create | Route wrapper for `CollectionView` |

## Interfaces / Contracts

```rust
// collection.rs
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct CollectionItemRow { /* raw DB columns */ }

#[derive(Debug, Clone, Serialize)]
pub struct CollectionItem {
    pub id: i64,
    pub sku: Option<String>,
    pub name: String,
    // ... other columns
    pub estimated_value: Option<f64>, // hydrated at query time, not stored
}

#[derive(Debug, Clone, Deserialize)]
pub struct CollectionItemInput { /* mutable fields */ }

#[derive(Debug, Clone, Deserialize)]
pub struct CollectionItemUpdates { /* all Option<T> */ }

#[derive(Debug, Clone, Serialize)]
pub struct CollectionStats {
    pub total_items: u32,
    pub total_value: f64,
    pub top_item_name: Option<String>,
}

// Standalone helper
pub async fn estimated_value(sku: &str, pool: &SqlitePool) -> Result<Option<f64>, sqlx::Error>;
```

IPC commands follow the existing `_cmd` pattern (e.g. `add_to_collection_cmd`) so unit tests avoid the Tauri runtime.

## Testing Strategy

| Layer | What to Test | Approach |
|---|---|---|
| Unit | `CollectionRepo` CRUD, `estimated_value` fallback chain, `get_stats` edge cases | In-memory SQLite pool, seeded `price_history` and `products_meta` rows |
| Unit | IPC command core fns | Call `_cmd` variants directly with in-memory pool |
| Integration | Export ZIP contains `collection_items.json` and `file_count` is 4 | Run `ExportService::export_to` against real migration chain (001→008) |
| E2E | Cell 8 renders stats; `/collection` grid shows gain/loss; `ProductCard` button hidden when collected | Tauri WebDriver (weekly cadence, not blocking PR) |

## Migration / Rollout

No feature flags required. Migration 008 is additive and backward-compatible. Rollback: drop `collection_items`, delete 008 file, unregister commands, revert Svelte changes. Because the app is offline-first and user-local, there is no server coordination.

## Open Questions

- Should `update_collection_item` support setting a field to `NULL` (e.g. clearing `serial_number`)? Current `COALESCE` approach prevents this; revisit if requested.
- Should we cache estimated values in memory for the session to avoid re-querying on every dashboard refresh? Likely unnecessary for MVP but worth profiling after PR 2.
