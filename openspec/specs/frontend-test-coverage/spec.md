# Frontend Test Coverage Specification

## Purpose

Provide unit-test coverage for the four critical frontend components that currently lack automated tests: ProductCard, Settings, PriceChart, and the dashboard +page.

## Requirements

### Requirement: ProductCard renders product info and actions

The `ProductCard` component MUST render product name, price, image, and a price badge when `priceInsight` is provided. It MUST render an "Add to collection" button when the SKU is not in the collection.

#### Scenario: ProductCard with price insight

- GIVEN a product with `name = "Fender Stratocaster"`, `price = 1299`, and `priceInsight = { level: "green", confidence: 85, pct: 12 }`
- WHEN the component mounts
- THEN the product name is visible
- AND the price is visible
- AND a `PriceBadge` with the correct props is rendered

#### Scenario: ProductCard add-to-collection button

- GIVEN a product with `sku = "NEW-001"` not in the collection store
- WHEN the component mounts
- THEN an "Add to collection" button is visible
- AND clicking it calls `addToCollection` with the correct SKU

#### Scenario: ProductCard already in collection

- GIVEN a product with `sku = "EXIST-001"` already in the collection store
- WHEN the component mounts
- THEN the "Add to collection" button is hidden
- AND a "In collection" indicator is visible

---

### Requirement: Settings renders form fields and persists on save

The `Settings` component MUST render form fields for currency, price alert threshold, and notification preferences. It MUST persist changes to the settings store and show a saved-feedback state.

#### Scenario: Settings form renders

- GIVEN the `settingsStore` contains `{ currency: "USD", threshold: 50, notifications: true }`
- WHEN the component mounts
- THEN the currency select shows "USD"
- AND the threshold input shows "50"
- AND the notifications toggle is checked

#### Scenario: Settings save button provides feedback

- GIVEN the `Settings` component is rendered with modified form values
- WHEN the user clicks the save button
- THEN the settings store is updated with the new values
- AND a "Saved" feedback message is visible within 500ms

---

### Requirement: PriceChart renders chart with price history

The `PriceChart` component MUST render a chart element when `priceHistory` data is provided. It MUST show an empty state when no data is available.

#### Scenario: PriceChart with data

- GIVEN `priceHistory` with 5 data points
- WHEN the component mounts
- THEN a chart canvas or SVG element is present
- AND 5 data points are represented

#### Scenario: PriceChart empty state

- GIVEN an empty `priceHistory` array
- WHEN the component mounts
- THEN the text "No price history available" is visible

---

### Requirement: Dashboard page renders all 9 cells

The `+page.svelte` dashboard route MUST render all 9 bento grid cells when the dashboard loads. It MUST show loading states for cells that depend on async IPC calls.

#### Scenario: Dashboard page renders grid

- GIVEN the dashboard page route
- WHEN the page mounts
- THEN exactly 9 `.dashboard-cell` elements are present
- AND Cell 1 (Search Results) is visible
- AND Cell 8 (Collection Stats) is visible

#### Scenario: Dashboard page loading states

- GIVEN the dashboard page with `collectionStore.loading = true`
- WHEN the page mounts
- THEN Cell 8 shows a loading spinner
- AND other cells continue to render independently

---

### Requirement: All new tests run under `npm run test`

The four new test files MUST be discovered by vitest and MUST pass when `npm run test` is executed.

#### Scenario: Test suite passes

- GIVEN a clean working tree with `node_modules` installed
- WHEN `npm run test` is executed
- THEN all tests in `ProductCard.test.ts`, `Settings.test.ts`, `PriceChart.test.ts`, and `+page.test.ts` pass
- AND the total frontend test count is at least 10 (existing + new)
