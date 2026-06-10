# Design: Clippy Fix

## Technical Approach

Three isolated Clippy errors in test-only code, each with a mechanical fix:

1. **`src-tauri/src/repository/sqlite/migrations/mod.rs`** — Remove `.to_string()` from two `&str` literals at lines 1660 and 1664. `std::fs::write` accepts `impl AsRef<[u8]>`; both `&str` and `String` implement it, so `.to_string()` is redundant.
2. **`src-tauri/src/services/search.rs`** — Extract the 9-parameter test helper into a `ProductTestParams<'a>` struct with `#[derive(Default)]` to eliminate the `too_many_arguments` lint.

No production code changes. Validate with `cargo clippy --all-targets -- -D warnings && cargo test`.

## Architecture Decisions

### Remove `.to_string()` vs. suppress lint

| Option | Tradeoff | Decision |
|--------|----------|----------|
| Remove `.to_string()` | One-line change, semantically correct | **Adopted** — minimal diff, no behavioral change |
| `#[allow()]` | Suppresses future similar lints | Rejected — mask rather than fix |
| Clippy `--allow` flag | Weakens global lint discipline | Rejected — `-D warnings` is project policy |

### Struct vs. positional builder for test params

| Option | Tradeoff | Decision |
|--------|----------|----------|
| `ProductTestParams` struct | Named fields at every call site. `#[derive(Default)]` for test ergonomics. | **Adopted** — idiomatic Rust test pattern. 10 call sites become self-documenting. |
| Builder pattern | More code (`ParamsBuilder` + setters), no benefit for flat creation | Rejected — over-engineered for test helpers with all-required semantics |
| `#[allow(clippy::too_many_arguments)]` | Zero code change, but lint reappears on any new param | Rejected — structural fix is better than perma-allow |

Struct uses `#[derive(Default)]` with sensible defaults (sku: `"TEST-SKU-001"`, name: `"Test Product"`, price: `100.0`) so future tests can create minimal params. All current call sites override every field, so no behavioral change.

## Data Flow

No data flow change — both fixes are syntactic. The helper struct is destructured inside the same function body, and `.bind()` parameter order is unchanged.

```
   Call site ──→ ProductTestParams{ pool, sku, name, ... }
                        │
                        ▼
            insert_product_with_condition_currency(params)
                        │
                        ▼
              INSERT INTO products_meta ... (same query, same bind order)
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `src-tauri/src/repository/sqlite/migrations/mod.rs` | Modify | Remove `.to_string()` from lines 1660, 1664 |
| `src-tauri/src/services/search.rs` | Modify | Add `ProductTestParams` struct (~15 lines), refactor function signature, update 10 call sites |

## Interfaces / Contracts

```rust
// New struct — placed above insert_product_with_condition_currency in the tests module
#[derive(Default)]
struct ProductTestParams<'a> {
    pool: &'a SqlitePool,
    sku: &'a str,       // Default: "TEST-SKU-001"
    name: &'a str,      // Default: "Test Product"
    brand: &'a str,
    category: &'a str,
    price: f64,         // Default: 100.0
    source_id: &'a str,
    condition: &'a str,
    currency: &'a str,
}
```

Function signature shrinks from 9 positional params to 1 struct param:

```rust
// Before
async fn insert_product_with_condition_currency(
    pool: &SqlitePool, sku: &str, name: &str, brand: &str,
    category: &str, price: f64, source_id: &str,
    condition: &str, currency: &str,
)

// After
async fn insert_product_with_condition_currency(
    params: ProductTestParams<'_>,
)
```

## Testing Strategy

No new tests. Coverage is proven by existing tests passing:

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit | Clippy compliance | `cargo clippy --all-targets -- -D warnings` → exit 0 |
| Integration | Test helper still works | `cargo test` — all `search::*` tests pass |

## Migration / Rollout

No migration required. Both files are test-only code. Single commit.

## Open Questions

None. The approach is fully specified by the proposal and confirmed by codebase inspection.
