# Spec + Design: setup-frontend-testing

## Purpose

Introduce a frontend unit-testing layer for the SvelteKit 5 + Tauri 2 application, enabling component-level TDD and protecting against regressions in shared UI components.

---

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

## Design

### 1. Package Installation

```bash
npm install -D vitest@^3.0.0 @vitest/coverage-v8@^3.0.0 @testing-library/svelte@^5.2.0 @testing-library/jest-dom@^6.6.0 @testing-library/user-event@^14.6.0 jsdom@^26.0.0
```

**Rationale:** vitest 3 aligns with Vite 6 already in use. `@testing-library/svelte` 5.2+ explicitly supports Svelte 5 runes. jsdom is chosen over happy-dom for broader DOM compatibility during early adoption.

### 2. vitest.config.ts (new)

```typescript
import { defineConfig } from 'vitest/config';
import { sveltekit } from '@sveltejs/kit/vite';

export default defineConfig({
  plugins: [sveltekit()],
  test: {
    include: ['src/**/*.{test,spec}.{js,ts}'],
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./src/setupTests.ts'],
    coverage: {
      provider: 'v8',
      include: ['src/lib'],
    },
  },
});
```

**Rationale:** Separate file keeps test concerns decoupled from production Vite build config. `globals: true` simplifies jest-dom matcher usage without per-file imports.

### 3. src/setupTests.ts (new)

```typescript
import '@testing-library/jest-dom/vitest';
import { vi } from 'vitest';

// Mock Tauri invoke globally so component tests never attempt real IPC.
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));
```

**Rationale:** Centralising the Tauri mock prevents every component test from re-implementing the same `vi.mock` block and reduces boilerplate.

### 4. tsconfig.json update

Add to `compilerOptions`:
```json
"types": ["vitest/globals", "@testing-library/jest-dom"]
```

**Rationale:** Provides TypeScript autocomplete for `expect`, `describe`, `it`, and custom matchers like `toBeInTheDocument()`.

### 5. package.json scripts

```json
{
  "test": "vitest run",
  "test:watch": "vitest",
  "test:coverage": "vitest run --coverage"
}
```

### 6. Makefile update

```makefile
## Run only frontend tests
## TODO: Add coverage target once thresholds are defined.
test-frontend:
	npm run test

## Run all tests (Rust + Python + Frontend)
test: test-app test-scraper test-frontend
```

**Rationale:** Preserves existing `test-app` and `test-scraper` behaviour while adding the frontend gate. The comment flags a future threshold decision.

### 7. CI workflow update

Add to `.github/workflows/ci.yml`:

```yaml
frontend:
  runs-on: ubuntu-latest
  timeout-minutes: 10
  steps:
    - uses: actions/checkout@v4
    - uses: actions/setup-node@v4
      with:
        node-version: '22'
    - run: npm ci
    - run: npm run test
```

**Rationale:** 10-minute timeout is conservative for a first-run npm install + vitest suite. Node 22 matches the current LTS and the project toolchain.

### 8. First Test Files

- `src/lib/components/__tests__/DashboardCell.test.ts`
- `src/lib/components/__tests__/PriceBadge.test.ts`
- `src/lib/components/__tests__/CollectionView.test.ts`

Each file uses `render` from `@testing-library/svelte` and asserts with jest-dom matchers.

### 9. Tauri Invoke Mocking Pattern

```typescript
import { invoke } from '@tauri-apps/api/core';
import { vi } from 'vitest';

vi.mocked(invoke).mockResolvedValue({ total_items: 5, total_value: 5000 });
```

Because `setupTests.ts` already registers the mock, per-test code only needs to set the resolved value.

### 10. Store Mocking Pattern

```typescript
import { collectionStore } from '$lib/stores/collection';

beforeEach(() => {
  collectionStore.set({
    stats: null,
    items: [],
    collectedSkus: new Set(),
    loading: false,
    error: null,
  });
});
```

The `collectionStore` is a `writable`, so direct `set` is the simplest and most deterministic reset strategy.

### 11. Svelte 5 Snippet Testing Strategy

`DashboardCell` accepts a `children` snippet prop. In tests, pass children using the `render` helper from `@testing-library/svelte`:

```typescript
import { render } from '@testing-library/svelte';
import DashboardCell from '../DashboardCell.svelte';

render(DashboardCell, {
  props: { title: 'Test', children: someSnippet },
});
```

If `@testing-library/svelte` does not yet expose a clean snippet API, create a thin wrapper component in the test file that renders `DashboardCell` with inline `{#snippet children()}` content. Document the chosen pattern in the first test file so later tests can reuse it.

---

## Out of Scope

- Full test suite for all remaining components (future incremental work).
- E2E tests with `tauri-driver` (separate change).
- Coverage thresholds enforcement (can be added after baseline coverage is measured).

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Svelte 5 snippet rendering in tests | Medium | Medium | Use wrapper component fallback; document pattern |
| Tauri mock misconfiguration | Low | High | Central mock in `setupTests.ts`; verify with `CollectionView` test |
| Vite 6 + vitest 3 compatibility | Low | Medium | Pin exact versions; run `npm run test` immediately after install |
| CI time increase | High | Low | 10-minute timeout; parallel job execution |
