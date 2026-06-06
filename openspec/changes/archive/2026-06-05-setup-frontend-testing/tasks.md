# Tasks: Setup Frontend Testing

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~350 |
| 400-line budget risk | Low |
| Chained PRs recommended | No |
| Suggested split | Single PR |
| Delivery strategy | single-pr |
| Chain strategy | pending |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: pending
400-line budget risk: Low

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | Install testing harness and write first component tests | PR 1 | Base = main; includes config, CI, Makefile, and 3 tests |

---

## Phase 1: Infrastructure

- [x] 1.1 Install Vitest and testing dependencies (`vitest`, `@vitest/coverage-v8`, `@testing-library/svelte`, `@testing-library/jest-dom`, `@testing-library/user-event`, `jsdom`) into devDependencies
- [x] 1.2 Create `vitest.config.ts` with jsdom environment, `globals: true`, `setupFiles: ['./src/setupTests.ts']`, and v8 coverage scoped to `src/lib`
- [x] 1.3 Create `src/setupTests.ts` that imports `@testing-library/jest-dom/vitest` and globally mocks `@tauri-apps/api/core` `invoke` with `vi.fn()`

## Phase 2: Tooling Integration

- [x] 2.1 Update `tsconfig.json` to include `vitest/globals` and `@testing-library/jest-dom` in `compilerOptions.types`
- [x] 2.2 Add `test`, `test:watch`, and `test:coverage` scripts to `package.json`
- [x] 2.3 Add `test-frontend` target to `Makefile` and include it in the existing `test` target after `test-app` and `test-scraper`
- [x] 2.4 Add `frontend` job to `.github/workflows/ci.yml` (Node 22, `npm ci`, `npm run test`, 10-minute timeout) running in parallel with the Rust job

## Phase 3: First Component Tests

- [x] 3.1 Write `src/lib/components/__tests__/DashboardCell.test.ts` covering title/icon render, loading state, empty state, and children snippet (REQ-TEST-4)
- [x] 3.2 Write `src/lib/components/__tests__/PriceBadge.test.ts` covering green/amber variants and dot patterns for high, medium, and low confidence (REQ-TEST-5)
- [x] 3.3 Write `src/lib/components/__tests__/CollectionView.test.ts` covering empty collection message and populated collection card list by mocking `collectionStore` (REQ-TEST-6)

## Phase 4: Verification

- [x] 4.1 Run `npm run test` locally and confirm all 3 component tests pass (REQ-TEST-1 S1)
- [x] 4.2 Run `make test` and confirm `test-app`, `test-scraper`, and `test-frontend` all pass in sequence (REQ-TEST-2 S3)
- [x] 4.3 Verify CI `frontend` job configuration by inspecting the workflow file for Node 22 setup and `npm run test` step (REQ-TEST-3 S4)
