# Search Service Specification

> **Status**: New capability  
> **Change**: mvp-completion

## Purpose

Provide FTS5-powered product search with input sanitization, multi-dimensional filtering (category, price range, source), pagination, and sorting — exposed as a Tauri IPC command.

## Requirements

### Requirement: search_products Tauri command MUST exist

The system MUST provide `#[tauri::command] search_products(query: String, state: State<'_, AppState>, filters: Option<SearchFilters>, sort: Option<SortOrder>, limit: Option<u32>, offset: Option<u32>) -> Result<SearchResult, AppError>`.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Basic text search | FTS5 index has 10 guitars | `search_products("guitar")` | Returns matching products from `products_fts` |
| Empty query | Empty string passed | Search with "" | Returns `AppError::InvalidInput` ("query too short") |
| Query too short | 1-char string | Search with "a" | Returns `AppError::InvalidInput` (min 3 chars) |
| Special chars sanitized | Query "Fender!@#" | Search executes | Special chars stripped/escaped before FTS5 MATCH |
| No results | Query "xyznonexistent" | Search | `total: 0`, `products: []` |

### Requirement: Search MUST filter by category, price, and source

The `SearchFilters` struct SHALL support optional `category: Option<String>`, `price_min: Option<f64>`, `price_max: Option<f64>`, and `source: Option<String>`.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Category filter | "Guitar" and "Bass" products exist | Search with `{ category: "Guitar" }` | Only guitars returned |
| Price range filter | Products at $100, $200, $500 | Search with `{ price_min: 150, price_max: 400 }` | Only $200 product returned |
| Source filter | Products from "reverb" and "ebay" | Search with `{ source: "reverb" }` | Only reverb products returned |
| Combined filters | All filters active | Search with category + price + source | Intersection of all filters applied |

### Requirement: Search MUST paginate

Results SHALL support `limit` (default 20, max 100) and `offset` (default 0). The response MUST include `total: u64` for total matching count.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| First page | 50 total matches | `{ limit: 10, offset: 0 }` | Returns 10 products, `total: 50` |
| Second page | 50 total matches | `{ limit: 10, offset: 10 }` | Returns next 10 products |
| Page beyond results | 5 total matches | `{ limit: 10, offset: 10 }` | Returns 0 products, `total: 5` |

### Requirement: Search MUST sort results

The `SortOrder` enum SHALL support `Relevance` (default), `PriceAsc`, `PriceDesc`, `NameAsc`, and `NameDesc`.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Sort by price ascending | Products at $100, $500, $200 | `{ sort: "PriceAsc" }` | Returns $100, $200, $500 |
| Sort by name | Products "A", "C", "B" | `{ sort: "NameAsc" }` | Returns "A", "B", "C" |
| Default relevance | Query "guitar" | No sort specified | Returns by FTS5 rank (best match first) |

### Requirement: Input MUST be sanitized before FTS5 MATCH

The query MUST be sanitized to prevent FTS5 syntax errors or injection: wrap each word in double quotes, strip FTS5 operators (`*`, `"`, `(`, `)`, `NEAR`, `NOT`, `OR`, `AND`).

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Contains quotes | Query: `guitar "electric"` | Sanitize | Becomes `"guitar" "electric"` |
| Contains operators | Query: `guitar NOT bass` | Sanitize | Becomes `"guitar" "bass"` (NOT removed) |
| Chinese/unicode | Query: `吉他` | Sanitize | Becomes `"吉他"` (trigram tokenizer handles it) |
