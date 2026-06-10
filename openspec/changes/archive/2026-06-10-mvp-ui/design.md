# Design: MVP UI — TypeScript Strict + Search Filters

## Technical Approach

Two mechanical phases with no dependency between them:

**Phase 1**: Mechanical type enforcement — add `lang="ts"`, `interface Props`, typed `$state<>()`, and typed `invoke<>()` to 5 components. Zero behavioral changes.

**Phase 2**: Collapsible FilterBar component between search bar and results. Filter state in a Svelte writable store, synced one-way to URL search params. Each filter change triggers `search(true)` passing the store's current filter object.

## Architecture Decisions

### Decision: FilterBar as reusable component

| Option | Tradeoff | Decision |
|--------|----------|----------|
| Inline in `+page.svelte` | Avoids new file, keeps filter logic coupled to page | ❌ Rejected |
| **Dedicated `FilterBar.svelte`** | Clean separation, testable in isolation, follows component convention | ✅ **Chosen** |

**Rationale**: 6+ filter controls + collapsible toggle + individual clear buttons + aria labels is too much for a 757-line page. Extracting keeps `+page.svelte` readable and follows the project's component pattern.

### Decision: Store as source of truth, URL as sync channel

| Option | Tradeoff | Decision |
|--------|----------|----------|
| URL as source of truth | Restore is easy, but store subscribers break on direct URL manipulation | ❌ Rejected |
| **Store + URL sync** | Store drives UI; URL updated on change (debounced). On mount: URL → store → search. Clean and observable. | ✅ **Chosen** |

**Rationale**: The spec requires both. Store-first avoids race conditions: components read one source, URL is a serialization side-effect.

### Decision: 300ms debounce for store→URL writes

| Option | Tradeoff | Decision |
|--------|----------|----------|
| No debounce | Every keystroke on price inputs floods history | ❌ Rejected |
| **300ms debounce** | Balances responsiveness with history noise | ✅ **Chosen** |
| onchange-only | Price inputs with spinbuttons need immediate URL | ❌ Rejected |

**Rationale**: URL updates are cosmetic (shareable links), not functional. 300ms keeps URL clean without UX lag. Store updates are immediate — no debounce there.

### Decision: Add `condition` and `listing_currency` to Rust `SearchFilters`

| Option | Tradeoff | Decision |
|--------|----------|----------|
| Client-side filter only | Only filters current page; wrong results server-side | ❌ Rejected |
| **Extend Rust struct** | Consistent with existing filter pattern; backend already has the columns | ✅ **Chosen** |

**Rationale**: The spec requires condition and currency filters, and the `products_meta` table already has `condition` and `currency` columns. Adding two `Option` fields to `SearchFilters` and two `AND m.condition = ?` / `AND m.currency = ?` clauses to `search.rs` is mechanically identical to the existing filter pattern. The proposal's "no new backend fields" was a conservative scope guard — the design confirms it's safe and trivial to extend.

## Data Flow

```
┌─────────────┐     user input      ┌──────────────┐
│  FilterBar   │ ─────────────────→ │ filterStore  │
│  (component) │                    │ (writable)   │
└─────────────┘                    └──────┬───────┘
                                          │
                    ┌─────────────────────┤
                    │                     │
                    ▼                     ▼
            ┌──────────────┐     ┌──────────────┐
            │ URL params   │     │ +page.svelte │
            │ (debounced   │     │ search()     │
            │  300ms)      │     │ reads store  │
            └──────────────┘     └──────┬───────┘
                                        │
                                        ▼
                               ┌────────────────┐
                               │ invoke(        │
                               │  search_products│
                               │  filters={...}) │
                               └───────┬────────┘
                                       │
                                       ▼
                              ┌─────────────────┐
                              │ FtsSearchService │
                              │ (dynamic WHERE)  │
                              └─────────────────┘
```

**Initialization**: On mount, parse `window.location.search` → populate `filterStore` → if any param is non-null, call `search(true)`.

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `src/lib/components/Settings.svelte` | Modify | Add `lang="ts"`, type state + invoke results |
| `src/lib/components/PriceChart.svelte` | Modify | Add `lang="ts"`, type `sku`, `windowDays`, `points`, `error` |
| `src/lib/components/PriceBadge.svelte` | Modify | Add `lang="ts"`, type `Props` for destructured props |
| `src/lib/components/ProductCard.svelte` | Modify | Add `lang="ts"`, type `product`, `priceInsight` |
| `src/lib/components/ProductDetail.svelte` | Modify | Add `lang="ts"`, type `product` prop |
| `src/lib/stores/filter.ts` | **Create** | Writable store with filter state + URL sync helpers |
| `src/lib/components/FilterBar.svelte` | **Create** | Collapsible filter controls component |
| `src/routes/+page.svelte` | Modify | Import FilterBar, read filterStore in `search()`, restore from URL on mount |
| `src/lib/types/search.ts` | Modify | Add `condition`, `listing_currency` to `SearchFilters`; add `get_categories` return type |
| `src-tauri/src/domain/product.rs` | Modify | Add `condition`, `listing_currency` to `SearchFilters` |
| `src-tauri/src/services/search.rs` | Modify | Add dynamic WHERE for condition + currency |
| `src-tauri/src/commands/dashboard_command.rs` | Modify | Add `get_categories` command |

## Interfaces / Contracts

**Extended `SearchFilters`** (Rust + TS mirror):

```rust
// Rust — src-tauri/src/domain/product.rs
pub struct SearchFilters {
    pub category: Option<String>,
    pub price_min: Option<f64>,
    pub price_max: Option<f64>,
    pub source: Option<String>,
    pub condition: Option<String>,        // NEW
    pub listing_currency: Option<String>, // NEW
}
```

```typescript
// TypeScript — src/lib/types/search.ts
export interface SearchFilters {
  category: string | null;
  price_min: number | null;
  price_max: number | null;
  source: string | null;
  condition: string | null;        // NEW
  listing_currency: string | null; // NEW
}
```

**New `filterStore` contract**:

```typescript
// src/lib/stores/filter.ts
export interface FilterState {
  category: string | null;
  price_min: number | null;
  price_max: number | null;
  source: string | null;
  condition: string | null;
  listing_currency: string | null;
  sort: SortOrder;  // always non-null, defaults to 'relevance'
}

export const filterStore: Writable<FilterState>;
export function syncFiltersToUrl(state: FilterState): void;    // debounced 300ms
export function restoreFiltersFromUrl(): FilterState;           // parse location.search
```

**New `get_categories` command**:

```rust
#[tauri::command]
pub async fn get_categories(state: State<'_, AppState>) -> Result<Vec<String>, AppError> {
    let categories: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT category FROM products_meta ORDER BY category"
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(categories)
}
```

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit | `filterStore` URL sync helpers | Parse/serialize round-trips with jest |
| Unit | `sanitize_fts_input` | Already tested — no change |
| Unit | New `SearchFilters` fields (Rust) | Round-trip JSON test + SQL filter binding test |
| Unit | `get_categories` | In-memory pool, insert distinct categories, assert sorted list |
| Integration | FilterBar renders all controls | Component mount test (Vitest + jsdom) |
| Integration | Filter change → store update → invoke called | Spy on `invoke`, change a filter, assert payload |
| E2E | Full flow: set filters → reload page → filters restored | Tauri WebDriver |

## Migration / Rollout

No migration required. The `condition` and `listing_currency` fields are `Option` — existing searches (which send `null`) keep working. Old URLs without params restore cleanly.

## Open Questions

- None — all decisions are resolved above.
