# Tasks: npm-audit-fix

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~5-10 |
| 400-line budget risk | Low |
| Chained PRs recommended | No |
| Suggested split | Single PR |
| Delivery strategy | single-pr |
| Chain strategy | pending |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: pending
400-line budget risk: Low

## Phase 1: Dependency Override

- [x] 1.1 Read `package.json` and add `"overrides": { "serialize-javascript": "7.0.5", "cookie": "0.7.2" }` after `devDependencies`
- [x] 1.2 Run `npm install` to regenerate `package-lock.json` with overridden resolutions
- [x] 1.3 Verify `npm ls serialize-javascript` resolves to `7.0.5` and `npm ls cookie` resolves to `0.7.2`

## Phase 2: Verification

- [x] 2.1 Run `npm audit` and confirm output shows `0 vulnerabilities` with exit code 0
- [x] 2.2 Run `npm test` (vitest) — all tests pass
- [ ] 2.3 Run `npm run check` (svelte-check) — exits 0 ❌ Pre-existing failure — jest types missing in `@testing-library/jest-dom`

## Phase 3: Cleanup

- [ ] 3.1 Commit `package.json` and `package-lock.json` together with a conventional commit message
