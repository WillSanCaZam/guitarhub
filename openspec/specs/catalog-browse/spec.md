# Capability: catalog-browse

> **Status**: New capability  
> **Change**: product-display-pipeline

## Purpose

Provide a `/catalog` SvelteKit route that mounts the existing `SearchPanel` component with FTS5-powered search and a responsive GearCard grid, enabling users to browse all products with search and filters.

## Requirements

### Requirement: /catalog route MUST mount SearchPanel with full catalog

The system MUST provide `src/routes/catalog/+page.svelte` that imports `<SearchPanel />` wired to the `filterState` and `collectionStore` reactive stores. The catalog SHALL include both public-scraped and user-connected products in results.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Route renders | App running | Navigate to `/catalog` | Page renders with no 404 |
| SearchPanel mounts | Route loaded | Page renders | Search bar visible, filter bar visible, GearCard grid placeholder shown |
| Stores connected | `filterState` + `collectionStore` | SearchPanel mounts | Filters reflect current store state, cards show collection status |
| User products visible | User connected Reverb with 5 listings | Navigate to `/catalog` | User's listings appear in results, SourceBadge shown per card |
| Min-height layout | Empty state | Route loads at 100% viewport | Page body fills at least the viewport height (no collapsed layout) |

### Requirement: FTS5 search MUST work end-to-end from /catalog including user products

Typing a query in the search bar SHALL invoke `search_products` which searches both public-scraped and user-connected products. Results SHALL include a `source` field per product and the SourceBadge component SHALL render accordingly.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Results found | FTS5 index has 10 products matching "guitar" | Type "guitar", press Enter | GearCard grid renders with matched products |
| No results | No matches for "xyznonexistent" | Type and search | `EmptyState` "No products found" shown |
| Too short | 1-char query | Type "a" | Search button disabled; hint "min. 3 characters" visible |
| With filters | Category filter active | Search with filters applied | Results narrowed to selected category |
| Load more | 50 total, 20 per page | Click "Load More" | Next 20 products appended to grid |
| Recent searches | Previous searches exist | Route loads | Recent search chips rendered above filter bar |
| User product found | User listing matches "Stratocaster" | Type "Stratocaster" | User's listing appears in results with source badge |
| Public-only search | No user_id filter active | Type "guitar" | All matching public + (any) user products returned |

### Requirement: SourceBadge MUST appear on user-connected products

Each product card in the catalog SHALL display a `<SourceBadge>` component indicating the product's source store. Connected (user-owned) products SHALL show "via Reverb — Your listing" or similar user-facing label. Public products SHALL show "via {source}" as before.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| User product badge | User connected, 1 listing | SearchPanel renders cards | SourceBadge shows "via Reverb — Your listing" with store icon |
| Public product badge | Public scraped product | Card renders | SourceBadge shows "via Reverb" |
| Mixed grid | 3 public + 2 user products | Catalog loads | Each card has correct badge variant |
