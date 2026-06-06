# Proposal: Setup Frontend Testing

## Intent

Enable test-driven development for all future UI changes by adding a frontend test runner and the first component tests. Currently `make test` only covers Rust and Python; any Svelte 5 component change is untestable before merge. This closes the TDD gap.

## Scope

### In Scope
- Install Vitest + @testing-library/svelte + jsdom
- Add `vitest.config.ts`, `src/setupTests.ts`, and tsconfig types
- Write first tests for 3 components: `DashboardCell`, `PriceBadge`, `CollectionView`
- Add `test-frontend` to Makefile and a `frontend` job to CI

### Out of Scope
- Full test suite for remaining components (`ProductCard`, `ProductDetail`, etc.)
- E2E or visual regression tests
- Coverage enforcement thresholds

## Capabilities

### New Capabilities
- `frontend-testing`: Vitest-based component test harness for Svelte 5 with Tauri mocking.

### Modified Capabilities
- `frontend-scaffolding`: Add `test`, `test:watch`, and `test:coverage` scripts to `package.json`.

## Approach

Install exact packages: `vitest@^3.0.0`, `@vitest/coverage-v8@^3.0.0`, `@testing-library/svelte@^5.2.0`, `@testing-library/jest-dom@^6.6.0`, `@testing-library/user-event@^14.6.0`, `jsdom@^26.0.0`. Create a separate `vitest.config.ts` (cleaner than inlining into `vite.config.ts`). `setupTests.ts` mocks `@tauri-apps/api/core` `invoke` and initialises jest-dom matchers. Target pure/presentation components first to prove the harness before touching Tauri-dependent ones.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `package.json` | Modified | Add devDeps + `test` / `test:watch` / `test:coverage` scripts |
| `vitest.config.ts` | New | Test runner config: jsdom, setupFiles, coverage v8 |
| `tsconfig.json` | Modified | Add `vitest/globals` and `@testing-library/jest-dom` types |
| `Makefile` | Modified | Add `test-frontend`; include in `test` |
| `.github/workflows/ci.yml` | Modified | Add `frontend` job (Node 22, `npm ci`, `npm run test`) |
| `src/setupTests.ts` | New | Tauri invoke mock + jest-dom + userEvent setup |
| `src/lib/components/__tests__/` | New | `DashboardCell.test.ts`, `PriceBadge.test.ts`, `CollectionView.test.ts` |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Svelte 5 snippet rendering in tests | Med | Use `@testing-library/svelte` v5.2+; pass snippet props explicitly if needed |
| Tauri `invoke` mock leaks across tests | Med | Mock in `setupTests.ts`; reset per-test with `vi.clearAllMocks` |
| Store mocking pattern not obvious | Low | Mock `collectionStore` as a writable in `CollectionView` test; document pattern |
| Vite 6 + Vitest 3 compatibility | Low | Both are latest stable; verify with `npm run test` immediately after install |

## Rollback Plan

1. Revert `package.json` and `package-lock.json` to pre-change state.
2. Delete `vitest.config.ts`, `src/setupTests.ts`, and `src/lib/components/__tests__/`.
3. Revert `tsconfig.json`, `Makefile`, and `.github/workflows/ci.yml`.
4. Run `make test` to confirm baseline still passes.

## Dependencies

None. All tooling is npm-based and zero-cost.

## Success Criteria

- [ ] `npm run test` passes locally (Vitest run mode)
- [ ] `make test` passes (includes new `test-frontend` target)
- [ ] CI `frontend` job is green on PR
- [ ] 3 component tests (`DashboardCell`, `PriceBadge`, `CollectionView`) pass
