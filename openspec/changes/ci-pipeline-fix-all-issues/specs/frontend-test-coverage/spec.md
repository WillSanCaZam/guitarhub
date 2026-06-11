# Delta for frontend-test-coverage

> **Change**: ci-pipeline-fix-all-issues — Add 4 missing component tests + 3 missing store tests

## ADDED Requirements

### Requirement: Missing Svelte component tests MUST exist

Four component test files MUST be created in `src/lib/components/__tests__/`: `ProductCard.test.ts`, `Settings.test.ts`, `PriceChart.test.ts`, and `+page.test.ts` (dashboard). Each test MUST cover at least one happy path and one edge case.

#### Scenario: Component tests discovered by vitest

- GIVEN `npm run test` is executed
- WHEN vitest discovers test files
- THEN all 4 new component test files are included in the test run
- AND all tests pass

#### Scenario: Component tests cover rendering

- GIVEN a component test file
- WHEN the test mounts the component with valid props
- THEN the component renders expected DOM elements
- AND assertions pass without errors

### Requirement: Missing Svelte store tests MUST exist

Three store test files MUST be created in `src/lib/stores/__tests__/` covering the collection store, settings store, and the shared store module. Each test MUST verify state transitions and persistence.

#### Scenario: Store tests discovered by vitest

- GIVEN `npm run test` is executed
- WHEN vitest discovers test files
- THEN all 3 new store test files are included in the test run
- AND all tests pass

#### Scenario: Store tests verify state management

- GIVEN a store test file
- WHEN the test updates the store state
- THEN the store reflects the new state
- AND derived values update accordingly

## MODIFIED Requirements

### Requirement: All new tests run under `npm run test`

The new component and store test files MUST be discovered by vitest and MUST pass when `npm run test` is executed. The total frontend test count MUST be at least 13 (existing + 4 component + 3 store tests).

(Previously: only 4 component tests existed; 7 new tests are added for a total of at least 13.)

#### Scenario: Test suite passes

- GIVEN a clean working tree with `node_modules` installed
- WHEN `npm run test` is executed
- THEN all existing tests pass
- AND all 4 new component tests pass
- AND all 3 new store tests pass
- AND the total test count is at least 13
