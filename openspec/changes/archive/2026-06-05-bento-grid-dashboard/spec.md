# Delta Specs: bento-grid-dashboard

> **Change**: bento-grid-dashboard
> **Artifact**: spec
> **Mode**: openspec + engram

---

# Capability: bento-grid-ui (NEW)

## Purpose

Define the CSS Grid bento layout and 9-cell dashboard surface for the GuitarHub landing page (`/`). Replaces the existing vertical-stack layout with a glanceable grid composition while preserving all existing search, sync, and settings functionality.

## Requirements

### Requirement: The grid MUST use a 4-column CSS Grid on desktop

The main page container MUST declare `display: grid` with `grid-template-columns: repeat(4, 1fr)` on viewports >= 768px. The grid MUST have a `gap` of `16px` (or `1rem`). The page wrapper MUST constrain max-width to `1200px` and center horizontally.

#### Scenario: Desktop viewport renders 4 columns

- GIVEN a viewport width of 1024px
- WHEN the dashboard page loads
- THEN the CSS Grid container has exactly 4 equal columns
- AND all 9 cells are positioned within the grid

#### Scenario: Grid gap is consistent

- GIVEN the dashboard is rendered
- WHEN inspecting the grid container
- THEN the gap between cells is 16px

### Requirement: Cell sizes MUST follow the bento pattern

Each cell MUST be assigned a grid span class that maps to one of four size variants:

| Cell | Size | grid-column | grid-row |
|------|------|-------------|----------|
| Cell 1 (Search Results) | Hero (2x2) | span 2 | span 2 |
| Cell 2 (Sync Status + Drops) | Wide (2x1) | span 2 | span 1 |
| Cell 3 (Total Products) | Standard (1x1) | span 1 | span 1 |
| Cell 4 (Wishlist Count) | Standard (1x1) | span 1 | span 1 |
| Cell 5 (Recent Searches) | Standard (1x1) | span 1 | span 1 |
| Cell 6 (Featured Deal) | Wide (2x1) | span 2 | span 1 |
| Cell 7 (Quick Settings) | Standard (1x1) | span 1 | span 1 |
| Cell 8 (Price Trend) | Standard (1x1) | span 1 | span 1 |
| Cell 9 (App Info) | Standard (1x1) | span 1 | span 1 |

#### Scenario: Hero cell occupies 2x2 space

- GIVEN the dashboard is rendered on desktop
- WHEN inspecting Cell 1 (Search Results)
- THEN it occupies exactly 2 columns and 2 rows
- AND it is the largest cell in the grid

#### Scenario: Wide cells span 2 columns

- GIVEN the dashboard is rendered on desktop
- WHEN inspecting Cell 2 (Sync Status) and Cell 6 (Featured Deal)
- THEN each spans exactly 2 columns and 1 row

### Requirement: The grid MUST be mobile-stackable

On viewports < 768px, the grid MUST collapse to a single column via `@media (max-width: 768px)` with `grid-template-columns: 1fr`. All cell span classes MUST reset to `grid-column: span 1` and `grid-row: span 1` in the mobile breakpoint. The mobile layout MUST maintain the original cell order (1-9 top-to-bottom).

#### Scenario: Mobile viewport stacks to single column

- GIVEN a viewport width of 375px
- WHEN the dashboard page loads
- THEN the grid renders as a single column
- AND each cell is full-width

#### Scenario: Cell order is preserved on mobile

- GIVEN the dashboard is rendered on mobile
- WHEN scrolling through the cells
- THEN the cells appear in order 1 through 9

### Requirement: Each cell MUST have an independent empty state

Every cell MUST render its own empty state when no data is available. The empty state MUST NOT block the rendering of other cells. Empty states MUST display a contextual message and MAY include an icon or placeholder graphic.

#### Scenario: Empty product catalog shows empty state in Cell 3

- GIVEN the products_meta table has zero rows
- WHEN the dashboard loads
- THEN Cell 3 (Total Products) displays "No products yet" with a placeholder icon
- AND all other cells continue to render independently

#### Scenario: No wishlist items show empty state in Cell 4

- GIVEN the wishlist table has zero rows
- WHEN the dashboard loads
- THEN Cell 4 (Wishlist) displays "Wishlist is empty" with a link to search

### Requirement: Cells MUST use glassmorphism styling

Each cell MUST have a semi-transparent background (`rgba(255,255,255,0.05)` or equivalent), a `backdrop-filter: blur(12px)` (or `backdrop-filter: blur(8px)` as fallback), a subtle border (`1px solid rgba(255,255,255,0.1)`), and a border-radius of `12px` or `16px`. The glassmorphism effect MUST be visible over the app's dark background.

#### Scenario: Glassmorphism renders on desktop

- GIVEN the dashboard is rendered
- WHEN inspecting any cell
- THEN the background is semi-transparent
- AND `backdrop-filter: blur(12px)` is applied

#### Scenario: Fallback for unsupported browsers

- GIVEN a browser that does not support `backdrop-filter`
- WHEN the dashboard renders
- THEN the cell still displays with a solid semi-transparent background
- AND the content remains readable

### Requirement: The grid MUST be keyboard-navigable

The dashboard MUST support Tab navigation in cell order (1-9). Each cell container MUST be focusable or contain focusable elements. The tab order MUST follow the visual DOM order (Cell 1 → Cell 2 → ... → Cell 9). Focus indicators MUST be visible.

#### Scenario: Tab navigates through cells in order

- GIVEN the dashboard is rendered
- WHEN the user presses Tab repeatedly
- THEN focus moves sequentially from Cell 1 to Cell 9
- AND each cell shows a visible focus ring

#### Scenario: Focus is trapped inside interactive elements

- GIVEN a cell contains interactive elements (e.g., buttons, links)
- WHEN the user Tabs inside that cell
- THEN focus cycles through the cell's interactive elements before moving to the next cell

## Cell Specifications

| Cell | Name | Size | Content | Data Source |
|------|------|------|---------|-------------|
| 1 | Search Results | 2x2 | Existing product grid + search input | `search_products` IPC + local state |
| 2 | Sync Status + Latest Drops | 2x1 | Sync toast, last sync time, price drop count | `syncResult` store |
| 3 | Total Products | 1x1 | Count + label | `get_total_products` IPC |
| 4 | Wishlist Count | 1x1 | Count + label | `get_wishlist_count` IPC |
| 5 | Recent Searches | 1x1 | List of last 5 searches | `get_recent_searches` IPC |
| 6 | Featured Deal | 2x1 | Random product card from catalog | `search_products` IPC (random offset) |
| 7 | Quick Settings | 1x1 | Shortcut to Settings section | Local navigation |
| 8 | Price Trend Mini-Chart | 1x1 | Placeholder chart area | None (future capability) |
| 9 | App Info | 1x1 | App name, version, logo | `package.json` / Tauri metadata |

## Out of Scope

- Per-cell customization (drag-to-rearrange, hide/show)
- Background scheduler for auto-refresh
- Collection value calculation (requires new table)
- Real-time WebSocket updates

---

# Capability: dashboard-data (NEW backend)

## Purpose

Provide lightweight Rust IPC commands that supply aggregate metrics for the bento-grid dashboard cells. All commands are read-only, query existing tables, and MUST return within 50ms.

## Requirements

### Requirement: get_total_products command MUST return product count

The `get_total_products` Tauri command MUST execute `SELECT COUNT(*) FROM products_meta` and return the count as a `u64`. The command MUST be registered in `main.rs` under the Tauri `generate_handler!` macro.

#### Scenario: Returns exact count for populated catalog

- GIVEN the products_meta table contains 1,247 rows
- WHEN the frontend invokes `get_total_products`
- THEN the command returns `1247` within 50ms

#### Scenario: Returns zero for empty catalog

- GIVEN the products_meta table has zero rows
- WHEN the frontend invokes `get_total_products`
- THEN the command returns `0` within 50ms

#### Scenario: Frontend displays count in Cell 3

- GIVEN `get_total_products` returns `1247`
- WHEN the dashboard renders Cell 3
- THEN the count `1247` is displayed with the label "Products"

### Requirement: get_wishlist_count command MUST return wishlist count

The `get_wishlist_count` Tauri command MUST execute `SELECT COUNT(*) FROM wishlist` and return the count as a `u64`. The command MUST be registered in `main.rs` under the Tauri `generate_handler!` macro.

#### Scenario: Returns exact count for populated wishlist

- GIVEN the wishlist table contains 23 rows
- WHEN the frontend invokes `get_wishlist_count`
- THEN the command returns `23` within 50ms

#### Scenario: Returns zero for empty wishlist

- GIVEN the wishlist table has zero rows
- WHEN the frontend invokes `get_wishlist_count`
- THEN the command returns `0` within 50ms

#### Scenario: Frontend displays count in Cell 4

- GIVEN `get_wishlist_count` returns `23`
- WHEN the dashboard renders Cell 4
- THEN the count `23` is displayed with the label "Wishlist"

### Requirement: get_recent_searches command MUST return last 5 unique searches

The `get_recent_searches` Tauri command MUST read the last 5 unique search queries from the frontend's localStorage (key: `guitarhub_recent_searches`), deduplicated by query string (case-insensitive). The return type MUST be `Vec<String>`. The command MUST be registered in `main.rs` under the Tauri `generate_handler!` macro.

#### Scenario: Returns last 5 unique searches

- GIVEN localStorage contains `["fender", "gibson", "fender", "prs", "martin", "taylor"]`
- WHEN the frontend invokes `get_recent_searches`
- THEN the command returns `["taylor", "martin", "prs", "gibson", "fender"]` (most recent first, deduplicated)

#### Scenario: Returns empty list when no searches exist

- GIVEN localStorage has no `guitarhub_recent_searches` key
- WHEN the frontend invokes `get_recent_searches`
- THEN the command returns `[]`

#### Scenario: Frontend displays searches in Cell 5

- GIVEN `get_recent_searches` returns `["fender", "gibson", "prs"]`
- WHEN the dashboard renders Cell 5
- THEN the list displays as clickable links that trigger a search

### Requirement: All dashboard commands MUST return within 50ms

The `get_total_products`, `get_wishlist_count`, and `get_recent_searches` commands MUST complete and return within 50ms under normal conditions (local SQLite, warm cache). The 50ms budget includes deserialization, query execution, and serialization.

#### Scenario: All commands complete under 50ms

- GIVEN the database has 10,000 products and 50 wishlist items
- WHEN all three commands are invoked sequentially
- THEN each command completes in under 50ms
- AND the total sequential time is under 150ms

#### Scenario: Commands fail gracefully on database error

- GIVEN the database connection is unavailable
- WHEN any dashboard command is invoked
- THEN it returns an `AppError::Database` with a user-friendly message
- AND the frontend cell displays the error in its empty state

## Out of Scope

- Background scheduler for auto-refresh
- Real-time WebSocket updates
- Collection value calculation (requires new table)
- Per-cell customization (drag-to-rearrange)

---

# Modified Capability: ui (DELTA)

## Purpose

The existing `ui` spec (openspec/specs/ui/spec.md) governs `PriceBadge` and `ProductCard` only. The `+page.svelte` rewrite to bento grid is an additive layout change that does not modify `PriceBadge` or `ProductCard` behavior. Therefore, the existing `ui` spec requires **no delta** — the bento grid is a new capability (`bento-grid-ui`) that wraps existing components.

(Note: if future changes modify `ProductCard` or `PriceBadge` props, those will be captured as delta specs under the `ui` domain.)
