# Capability: bento-grid-ui

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
| Cell 8 (Collection Stats) | Standard (1x1) | span 1 | span 1 |
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

Each cell MUST apply the glassmorphism styling via the CSS class `.glassmorphism` or an equivalent named class that can be inspected in tests. The style MUST be applied to the cell container element, not merely declared in global CSS.

(Previously: Glassmorphism CSS was declared but not applied to cell containers via a named class.)

#### Scenario: Glassmorphism renders on desktop

- GIVEN the dashboard is rendered
- WHEN inspecting any cell
- THEN the cell element has a `.glassmorphism` class
- AND the background is semi-transparent
- AND `backdrop-filter: blur(12px)` is applied

#### Scenario: Fallback for unsupported browsers

- GIVEN a browser that does not support `backdrop-filter`
- WHEN the dashboard renders
- THEN the cell still displays with a solid semi-transparent background
- AND the content remains readable

---

### Requirement: Glassmorphism MUST degrade gracefully on older WebKit

On browsers that do not support `backdrop-filter`, the cell MUST fall back to a solid semi-transparent background via a `@supports` query or equivalent CSS feature detection.

#### Scenario: WebKit fallback

- GIVEN Safari 12 or another browser without `backdrop-filter` support
- WHEN the dashboard renders
- THEN the `@supports` fallback is active
- AND the cell background is `rgba(255,255,255,0.08)` or similar solid semi-transparent color

---

### Requirement: Cells MUST have focus-visible ring

Each cell container MUST show a visible focus ring when focused via keyboard navigation. The focus ring MUST use `outline` or `box-shadow` with a minimum 2px width and a color that contrasts with the glassmorphism background.

#### Scenario: Keyboard focus ring

- GIVEN the dashboard is rendered
- WHEN a cell receives focus via Tab key
- THEN a visible focus ring is present
- AND the ring color contrasts with the cell background

---

### Requirement: Cell border radius MUST be 12px

All cells MUST use a border-radius of exactly `12px`. The radius MUST be consistent across all cell sizes (Hero, Wide, Standard).

#### Scenario: Consistent border radius

- GIVEN the dashboard is rendered
- WHEN inspecting Cell 1, Cell 2, and Cell 3
- THEN each cell has `border-radius: 12px`
- AND no cell exceeds 12px radius

---

### Requirement: Grid gap MUST be 16px

The CSS Grid container MUST declare a gap of exactly `16px` (or `1rem`). The gap MUST be consistent across all breakpoints.

#### Scenario: Gap consistency

- GIVEN the dashboard is rendered on a 1024px viewport
- WHEN inspecting the grid container
- THEN the gap between cells is 16px
- AND the gap is consistent horizontally and vertically

### Requirement: The grid MUST be keyboard-navigable

The dashboard MUST support Tab navigation in cell order (1-9). Each cell container MUST be focusable or contain focusable elements. The tab order MUST follow the visual DOM order (Cell 1 -> Cell 2 -> ... -> Cell 9). Focus indicators MUST be visible.

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
| 8 | Collection Stats | 1x1 | Total items, total value, top item name | `get_collection_stats` IPC |
| 9 | App Info | 1x1 | App name, version, logo | `package.json` / Tauri metadata |

## Out of Scope

- Per-cell customization (drag-to-rearrange, hide/show)
- Background scheduler for auto-refresh

- Real-time WebSocket updates
