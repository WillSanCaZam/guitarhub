# Search Filters UI Specification

> **Status**: New capability
> **Change**: mvp-ui

## Purpose

Provide interactive filter controls on the search page that let users narrow product results by category, price range, source, sort order, condition, and listing currency. Filters sync with URL search params for shareable URLs and persist in a Svelte store.

## Requirements

### Requirement: Filter controls MUST render below the search bar

The search page (`+page.svelte`) MUST render collapsible filter controls for: category (dropdown), price_min and price_max (number inputs), source (select), sort_by/sort_order (radio or select), condition (select), listing_currency (select). Controls MUST appear between the search bar and the results area.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| All controls present | Page loaded, filters visible | Inspect DOM | Controls for each filter dimension rendered |
| Collapsible | Filters expanded | Toggle collapse | Filter bar hides; toggle shows |
| Default collapsed | Fresh page load | Render | Filter bar collapsed, search bar prominent |

### Requirement: Filter state MUST live in a Svelte store

All filter values MUST persist in a dedicated `filterStore` (Svelte writable store), not isolated in local component state. The store SHALL hold every filter field as a nullable value.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Cross-render persistence | User selects "Guitar" category | Component re-renders | Store value unchanged |
| Store shape | `filterStore` initialised | Observe store | All filter fields present, all `null` |
| Store-driven search | User changes filter | Search triggered | `invoke` reads from store |

### Requirement: Filters MUST sync with URL search params

Applying a filter MUST update `window.location.search` with corresponding params (`category`, `price_min`, `price_max`, `source`, `sort`, `condition`, `listing_currency`). Loading a page with these URL params MUST restore the filter values and trigger a search automatically.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| URL updated | User selects "Guitar" category | Observe URL | `?category=Guitar` appended |
| URL restored on load | URL has `?category=Guitar&sort=price_asc` | Page loads | Filters populated; search executes |
| All-null omitted | All filters null | Observe URL | No query params |
| Partial restore | URL has only `?price_min=100` | Page loads | Only price_min restored; others null |
| Shareable URL | URL copied and reopened | New tab | Same results rendered |

### Requirement: Filters MUST be clearable individually and collectively

Each filter control MUST have a clear button (×) that resets that field to `null`. A "Reset all filters" button MUST clear every filter at once. Both actions MUST trigger `search(true)`.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Clear single | Category set to "Guitar" | Click clear on category | Category null; search re-runs |
| Reset all | Multiple filters active | Click "Reset all" | All filters null; search re-runs |
| After reset URL clean | All filters reset | Observe URL | No filter params in URL |

### Requirement: Filter changes MUST re-trigger search

Any filter change (apply, clear, reset) MUST call `search(true)` with the updated filter values. The `invoke` payload MUST contain the store's current filter object and sort order.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Change triggers search | Category selected | Change event | `search(true)` invoked with new filters |
| All null = unfiltered | All filters null | Search | Filters sent as `{ category: null, price_min: null, ... }` |
| Sort order applied | User selects "Price: Low to High" | Sort changed | Sort param sent as `sort: "price_asc"` |

## Out of Scope

- Client-side post-filtering (all filtering MUST go through the Rust backend)
- Autocomplete or typeahead for category/condition/currency dropdowns
- Filter presets or saved filter combinations
- Animations on filter bar expand/collapse
- Dark mode styling for filter controls (follows existing page theme)

## Risks

| Risk | Mitigation |
|------|-----------|
| Backend `SearchFilters` struct lacks `condition` / `listing_currency` fields | Extend Rust struct (currently out of proposal scope — design phase MUST confirm) |
| URL param ↔ store sync causes infinite loop | Debounce store→URL writes; URL→store runs only once at init |
| Too many filters clutter the mobile layout | Collapse by default; stack controls vertically under 768 px |
