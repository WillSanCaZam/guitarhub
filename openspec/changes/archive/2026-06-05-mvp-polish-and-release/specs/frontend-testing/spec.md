# Delta for frontend-testing

## ADDED Requirements

### Requirement: ProductCard component tests

The `ProductCard` component MUST be covered by unit tests in `src/lib/components/__tests__/ProductCard.test.ts`. The tests MUST verify rendering of product info, price badge, and add-to-collection action.

#### Scenario: ProductCard renders product info

- GIVEN a product with `name = "Fender Stratocaster"`, `price = 1299`, `image_url = "https://example.com/img.jpg"`
- WHEN the component is rendered in vitest
- THEN the product name is present in the document
- AND the price is present
- AND the image has the correct `src` attribute

#### Scenario: ProductCard renders price badge

- GIVEN a product with `priceInsight = { level: "green", confidence: 85, pct: 12 }`
- WHEN the component is rendered
- THEN a `PriceBadge` element is present
- AND the badge shows the correct confidence dots

#### Scenario: ProductCard add-to-collection action

- GIVEN a product with `sku = "NEW-001"` not in the collection
- WHEN the component is rendered
- THEN the "Add to collection" button is visible
- AND clicking it invokes the `addToCollection` IPC call with the correct SKU

#### Scenario: ProductCard hides add button when in collection

- GIVEN a product with `sku = "EXIST-001"` already in the collection store
- WHEN the component is rendered
- THEN the "Add to collection" button is not visible

---

### Requirement: Settings component tests

The `Settings` component MUST be covered by unit tests in `src/lib/components/__tests__/Settings.test.ts`. The tests MUST verify form rendering, store binding, and save feedback.

#### Scenario: Settings renders form fields

- GIVEN the `settingsStore` contains `{ currency: "USD", threshold: 50, notifications: true }`
- WHEN the component is rendered
- THEN the currency select shows "USD"
- AND the threshold input shows "50"
- AND the notifications toggle is checked

#### Scenario: Settings save button provides feedback

- GIVEN the Settings component is rendered with modified values
- WHEN the save button is clicked
- THEN the settings store is updated
- AND a "Saved" text is visible within 500ms

#### Scenario: Settings save button is disabled during save

- GIVEN the user clicks the save button
- WHEN the save operation is in progress
- THEN the save button has the `disabled` attribute

---

### Requirement: PriceChart component tests

The `PriceChart` component MUST be covered by unit tests in `src/lib/components/__tests__/PriceChart.test.ts`. The tests MUST verify chart rendering with data and empty state.

#### Scenario: PriceChart renders with data

- GIVEN `priceHistory` with 5 data points
- WHEN the component is rendered
- THEN a chart element (canvas or SVG) is present
- AND 5 data points are represented

#### Scenario: PriceChart empty state

- GIVEN an empty `priceHistory` array
- WHEN the component is rendered
- THEN the text "No price history available" is visible

#### Scenario: PriceChart loading state

- GIVEN `priceHistory` is null or undefined
- WHEN the component is rendered
- THEN a loading spinner or placeholder is visible

---

### Requirement: Dashboard page tests

The `+page.svelte` dashboard route MUST be covered by unit tests in `src/routes/__tests__/+page.test.ts`. The tests MUST verify all 9 cells render and async stores are mocked.

#### Scenario: Dashboard renders all 9 cells

- GIVEN mocked stores for search, collection, sync, and products
- WHEN the dashboard page is rendered
- THEN exactly 9 `.dashboard-cell` elements are present
- AND Cell 1 (Search Results) is visible
- AND Cell 8 (Collection Stats) is visible

#### Scenario: Dashboard shows loading states

- GIVEN the collection store is in `loading` state
- WHEN the dashboard page is rendered
- THEN Cell 8 shows a loading indicator
- AND other cells continue to render independently

#### Scenario: Dashboard handles empty states

- GIVEN the products store is empty
- WHEN the dashboard page is rendered
- THEN Cell 3 (Total Products) shows the empty state
- AND Cell 4 (Wishlist Count) shows the empty state

---

### Requirement: All tests use consistent mocking patterns

All new component tests MUST follow the existing `DashboardCell.test.ts` pattern: mock `tauriInvoke` with `vi.fn()`, mock Svelte stores with writable mocks, and use `vi.waitFor` for async assertions.

#### Scenario: Tauri invoke mocked

- GIVEN a test file for any component that calls `tauriInvoke`
- WHEN the test runs
- THEN `tauriInvoke` is replaced with `vi.fn()` that returns a resolved Promise
- AND no actual Tauri backend calls are made

#### Scenario: Store mocks are stable

- GIVEN a test that depends on a Svelte store
- WHEN the test runs
- THEN the store is replaced with a writable mock that emits immediately
- AND `vi.waitFor` is used to assert on reactive updates

---

### Requirement: Test count increases by at least 4 new files

The frontend test suite MUST contain at least 4 new test files corresponding to the components: `ProductCard`, `Settings`, `PriceChart`, and `+page`.

#### Scenario: New test files discovered

- GIVEN `npm run test` is executed
- WHEN vitest scans `src/` for test files
- THEN `ProductCard.test.ts`, `Settings.test.ts`, `PriceChart.test.ts`, and `+page.test.ts` are discovered
- AND all 4 files pass without errors

#### Scenario: Total test count

- GIVEN the existing test suite has N tests
- WHEN the new tests are added
- THEN the total test count is at least N + 12 (3 scenarios per new component)
- AND `npm run test` exits with code 0
