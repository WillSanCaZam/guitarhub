# Capability: collection-ui

## Purpose

Frontend integration for collection management: dashboard cell, product card action, and collection view.

## Requirements

### Requirement: Cell 8 collection stats

Cell 8 in the bento grid MUST display collection stats: total items count, total estimated value, and top item by estimated value. Cell 8 MUST also display the aggregate gain/loss summary, computed as `SUM(estimated_value - purchase_price)` across all collection items.

(Previously: Cell 8 displayed total items, total value, and top item name. Gain/loss summary was not displayed.)

#### Scenario: Cell 8 renders stats with gain/loss

- GIVEN the dashboard loads with a collection containing 3 items
- WHEN Cell 8 is rendered
- THEN it shows total items count
- AND it shows total estimated value
- AND it shows the top item name
- AND it shows the aggregate gain/loss as a signed number (e.g., "+450" or "-120")

#### Scenario: Cell 8 shows positive gain

- GIVEN the collection has items with `purchase_price = [1000, 2000]` and `estimated_value = [1200, 2200]`
- WHEN Cell 8 is rendered
- THEN the gain/loss shows "+400"
- AND the gain/loss is displayed in green or with a positive indicator

#### Scenario: Cell 8 shows negative loss

- GIVEN the collection has items with `purchase_price = [1500, 3000]` and `estimated_value = [1400, 2800]`
- WHEN Cell 8 is rendered
- THEN the gain/loss shows "-300"
- AND the gain/loss is displayed in red or with a negative indicator

#### Scenario: Cell 8 shows zero gain/loss

- GIVEN the collection has items with `purchase_price = estimated_value` for all items
- WHEN Cell 8 is rendered
- THEN the gain/loss shows "0" or "±0"
- AND the indicator is neutral (grey or no color)

#### Scenario: Cell 8 gain/loss hidden when collection is empty

- GIVEN the collection has zero items
- WHEN Cell 8 is rendered
- THEN the gain/loss line is hidden
- AND the empty state message is shown

---

### Requirement: Gain/loss summary MUST be reactive

The gain/loss summary in Cell 8 MUST update automatically when the collection store changes (e.g., when an item is added, removed, or its price is updated). The component MUST use the existing Svelte store reactivity and MUST NOT require a page refresh.

#### Scenario: Gain/loss updates on item removal

- GIVEN the collection store initially has 2 items with a gain of +200
- WHEN one item is removed from the collection
- THEN the gain/loss in Cell 8 updates to reflect the new total within 1 second

#### Scenario: Gain/loss updates on price sync

- GIVEN the collection store has items with `estimated_value = 1000`
- WHEN a price sync updates `estimated_value` to `1100`
- THEN the gain/loss in Cell 8 updates to reflect the new difference

---

### Requirement: Gain/loss formatting MUST be locale-aware

The gain/loss value MUST be formatted with the user's selected currency and locale. The sign (+/-) MUST be preserved regardless of locale. The value MUST use the same number formatting as other price values in the app.

#### Scenario: USD formatting

- GIVEN the user currency is "USD"
- WHEN the gain/loss is +500
- THEN the display shows "+500" or "+$500" (consistent with app-wide currency formatting)

#### Scenario: EUR formatting

- GIVEN the user currency is "EUR"
- WHEN the gain/loss is -250
- THEN the display shows "-250" or "-€250" (consistent with app-wide currency formatting)

---

### Requirement: Collection view grid MUST also show gain/loss per item

The collection view grid (`/collection`) MUST display the gain/loss per item alongside the aggregate. This is existing behavior per the main spec, and MUST be preserved.

#### Scenario: Per-item gain/loss in collection view

- GIVEN an item with `purchase_price = 1000` and `estimated_value = 1200`
- WHEN the collection grid renders
- THEN the item row shows a gain of +200

---

### Requirement: Collection remove action is unchanged

The collection view MUST allow removing items. The remove action MUST update both the collection store and the Cell 8 gain/loss summary reactively.

#### Scenario: Remove updates dashboard

- GIVEN the collection view shows item `id = 5` with a gain of +100
- WHEN the user clicks remove and confirms
- THEN the item is removed from the grid
- AND Cell 8 gain/loss decreases by 100

### Requirement: ProductCard add button

`ProductCard` MUST show an "Add to collection" button when the SKU is not already in `collection_items`.

#### Scenario: Add button visible

- GIVEN a product with SKU `NEW-001` not in collection
- WHEN the card renders
- THEN an "Add to collection" button is visible

### Requirement: Collection view accessible

A collection view MUST be reachable via `/collection` route.

#### Scenario: Navigate to collection

- GIVEN the user clicks a collection link
- WHEN navigation completes
- THEN the `/collection` route renders

### Requirement: Collection view grid

The collection view MUST display items in a grid showing `estimated_value`, `purchase_price`, and gain/loss (`estimated_value - purchase_price`).

#### Scenario: Grid shows gain/loss

- GIVEN an item with `purchase_price = 1000` and `estimated_value = 1200`
- WHEN the collection grid renders
- THEN it displays a gain of `+200`

### Requirement: Collection remove action

The collection view MUST allow removing items.

#### Scenario: Remove from collection view

- GIVEN the collection view shows item `id = 5`
- WHEN the user clicks remove and confirms
- THEN the item is removed
- AND the grid updates

## Out of Scope

- Drag-to-rearrange collection items
- Photos upload (image_url only, no file upload)
- Collection value chart over time (deferred)
