# Frontend Testing Specification

> **Status**: Stable
> **Change**: setup-frontend-testing

## Purpose

Provide a frontend unit-testing layer for the SvelteKit 5 + Tauri 2 application, enabling component-level TDD and protecting against regressions in shared UI components.

## Requirements

### Requirement: REQ-TEST-1 — npm run test executes vitest

The `npm run test` script MUST execute vitest in run mode and discover and execute all `*.test.ts` and `*.spec.ts` files under `src/`.

#### Scenario: S1 — Discover tests automatically

- GIVEN a fresh clone with `node_modules` installed
- WHEN `npm run test` is executed
- THEN vitest discovers all `.test.ts` files in `src/`
- AND runs them inside the jsdom environment
- AND exits with code 0 when all tests pass

#### Scenario: S2 — Watch mode available

- GIVEN a developer running local changes
- WHEN `npm run test:watch` is executed
- THEN vitest re-runs affected tests on file save

---

### Requirement: REQ-TEST-2 — make test runs all test suites

The `make test` target MUST execute the Rust test suite (`cargo test`), the Python scraper test suite (`pytest`), and the frontend test suite (`npm run test`), stopping on the first failure.

#### Scenario: S3 — Full project validation

- GIVEN a clean working tree
- WHEN `make test` is executed
- THEN `test-app` (cargo test) runs first
- AND `test-scraper` (pytest) runs second
- AND `test-frontend` (vitest) runs third
- AND the overall command exits non-zero if any suite fails

---

### Requirement: REQ-TEST-3 — CI workflow includes frontend job

The `.github/workflows/ci.yml` MUST include a `frontend` job that installs Node 22, runs `npm ci`, and then runs `npm run test`.

#### Scenario: S4 — Pull-request gate

- GIVEN a pull request is opened
- WHEN the CI workflow runs
- THEN the `frontend` job executes in parallel with the `rust` job
- AND the job installs Node 22 and runs `npm ci`
- AND the job runs `npm run test`
- AND the PR is blocked if any test fails

---

### Requirement: REQ-TEST-4 — DashboardCell renders all states

The `DashboardCell` component MUST render its title, optional icon, children snippet, empty state, and loading state correctly under the respective prop conditions.

#### Scenario: S1 — Render title and icon

- GIVEN a `DashboardCell` rendered with `title="Search"` and `icon="🔍"`
- WHEN the component mounts
- THEN the heading "Search" is present in the document
- AND the icon "🔍" is visible

#### Scenario: S2 — Loading state

- GIVEN a `DashboardCell` rendered with `loading={true}`
- WHEN the component mounts
- THEN the loading spinner element is present
- AND the text "Loading..." is visible

#### Scenario: S3 — Empty state

- GIVEN a `DashboardCell` rendered with `empty={true}` and `emptyMessage="No results"`
- WHEN the component mounts
- THEN the empty icon and the text "No results" are visible

#### Scenario: S6 — Children snippet

- GIVEN a `DashboardCell` rendered with default props (not loading, not empty) and a child snippet containing `<p>Content</p>`
- WHEN the component mounts
- THEN the child paragraph "Content" is visible

---

### Requirement: REQ-TEST-5 — PriceBadge renders green/amber with correct dot patterns

The `PriceBadge` component MUST render the green or amber variant based on the `level` prop and display the correct three-dot confidence glyph derived from the `confidence` prop.

#### Scenario: S4 — High confidence green badge

- GIVEN a `PriceBadge` rendered with `level="green"` and `confidence={85}`
- WHEN the component mounts
- THEN the badge text contains "Good price"
- AND the dot pattern "•••" is visible

#### Scenario: S5 — Medium confidence amber badge

- GIVEN a `PriceBadge` rendered with `level="amber"` and `confidence={60}`
- WHEN the component mounts
- THEN the badge text contains "Above average"
- AND the dot pattern "••○" is visible

#### Scenario: S7 — Low confidence badge

- GIVEN a `PriceBadge` rendered with `level="green"` and `confidence={30}`
- WHEN the component mounts
- THEN the dot pattern "•○○" is visible

---

### Requirement: REQ-TEST-6 — CollectionView renders empty state and item list

The `CollectionView` component MUST render an empty-state message when the collection store has zero items, and MUST render one card per item when the store contains items.

#### Scenario: S6 — Empty collection

- GIVEN the `collectionStore` is set to `{ items: [], loading: false }`
- WHEN `CollectionView` mounts
- THEN the text "Your collection is empty." is visible
- AND the hint "Search for gear to add!" is visible

#### Scenario: S7 — Populated collection

- GIVEN the `collectionStore` is set to `{ items: [itemA, itemB], loading: false }`
- WHEN `CollectionView` mounts
- THEN exactly two `.collection-card` elements are present
- AND each card displays the correct item name and price values

---

### Requirement: REQ-TEST-7 — ProductCard component tests

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

### Requirement: REQ-TEST-8 — Settings component tests

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

### Requirement: REQ-TEST-9 — PriceChart component tests

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

### Requirement: REQ-TEST-10 — Dashboard page tests

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

### Requirement: REQ-TEST-11 — All tests use consistent mocking patterns

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

### Requirement: REQ-TEST-12 — Test count increases by at least 4 new files

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
