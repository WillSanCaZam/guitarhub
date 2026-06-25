# Capability: catalog-browse

> **Status**: New capability  
> **Change**: product-display-pipeline

## Purpose

Provide a `/catalog` SvelteKit route that mounts the existing `SearchPanel` component with FTS5-powered search and a responsive GearCard grid, enabling users to browse all products with search and filters.

## Requirements

### Requirement: /catalog route MUST mount SearchPanel

The system MUST provide `src/routes/catalog/+page.svelte` that imports `<SearchPanel />` wired to the `filterState` and `collectionStore` reactive stores.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Route renders | App running | Navigate to `/catalog` | Page renders with no 404 |
| SearchPanel mounts | Route loaded | Page renders | Search bar visible, filter bar visible, GearCard grid placeholder shown |
| Stores connected | `filterState` + `collectionStore` | SearchPanel mounts | Filters reflect current store state, cards show collection status |
| Min-height layout | Empty state | Route loads at 100% viewport | Page body fills at least the viewport height (no collapsed layout) |

### Requirement: FTS5 search MUST work end-to-end from /catalog

Typing a query in the search bar SHALL invoke `search_products` via `@tauri-apps/api/core` IPC, and the SearchPanel SHALL render results as a responsive GearCard grid.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Results found | FTS5 index has 10 products matching "guitar" | Type "guitar", press Enter | GearCard grid renders with matched products |
| No results | No matches for "xyznonexistent" | Type and search | `EmptyState` "No products found" shown |
| Too short | 1-char query | Type "a" | Search button disabled; hint "min. 3 characters" visible |
| With filters | Category filter active | Search with filters applied | Results narrowed to selected category |
| Load more | 50 total, 20 per page | Click "Load More" | Next 20 products appended to grid |
| Recent searches | Previous searches exist | Route loads | Recent search chips rendered above filter bar |
