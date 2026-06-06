## Exploration: setup-frontend-testing

### Current State
- **package.json**: No test frameworks. Scripts: `dev`, `build`, `preview` only.
- **package-lock.json**: Confirms zero presence of vitest, jest, mocha, playwright.
- **vite.config.ts**: No `test` configuration block.
- **tsconfig.json**: No test types or include paths.
- **Existing tests**: Only Python scraper tests (`scraper/tests/unit/`, `scraper/tests/contract/`). No `.test.ts`, `.spec.ts`, or `tests/` directory in the frontend.
- **Makefile**: `test` runs `test-app` (cargo) + `test-scraper` (pytest). No frontend target.
- **CI** (`.github/workflows/ci.yml`): Only a `rust` job. Python job is commented out. No Node/vitest step.

### Affected Areas
- `package.json` — add devDependencies + scripts
- `vite.config.ts` — optionally merge vitest config
- `tsconfig.json` — add test types / include
- `Makefile` — add `test-frontend` target, include in `test`
- `.github/workflows/ci.yml` — add `frontend` job
- New: `vitest.config.ts` — test runner config
- New: `src/setupTests.ts` — jest-dom matchers + Tauri invoke mock + user-event setup
- New: `src/lib/components/__tests__/` — first component tests

### Approaches
1. **Separate `vitest.config.ts`** (recommended)
   - Pros: Clean separation from Vite build config; standard pattern; easy to extend.
   - Cons: One more file.
   - Effort: Low

2. **Inline `test` block in `vite.config.ts`**
   - Pros: One less file.
   - Cons: Mixes build and test concerns; harder to conditionally load test-only plugins.
   - Effort: Low

### Recommendation
Use **Approach 1** (separate `vitest.config.ts`). It keeps build and test configs decoupled, which is the idiomatic pattern for SvelteKit + Vitest and makes it easier to add test-only plugins (e.g., `vitest-dom` or `msw`) later without touching the production Vite config.

### Tooling Recommendations (exact packages)
```json
"devDependencies": {
  "vitest": "^3.0.0",
  "@vitest/coverage-v8": "^3.0.0",
  "@testing-library/svelte": "^5.2.0",
  "@testing-library/jest-dom": "^6.6.0",
  "@testing-library/user-event": "^14.6.0",
  "jsdom": "^26.0.0"
}
```
- **vitest 3.x**: Aligned with Vite 6 already in use.
- **jsdom**: Stable, well-known. happy-dom is faster but less battle-tested for complex DOM; stick with jsdom for now.
- **@testing-library/svelte 5.2+**: Explicitly supports Svelte 5 runes.
- **@testing-library/jest-dom**: Provides `.toBeInTheDocument()`, `.toHaveClass()`, etc.

### Configuration Plan
1. **vitest.config.ts**
   - `plugins: [sveltekit()]` (reuse vite plugin)
   - `test.environment: 'jsdom'`
   - `test.globals: true` (optional, but helps jest-dom)
   - `test.setupFiles: ['./src/setupTests.ts']`
   - `coverage.provider: 'v8'`, `coverage.include: ['src/lib']`

2. **tsconfig.json**
   - Add `"types": ["vitest/globals", "@testing-library/jest-dom"]` to `compilerOptions`
   - Ensure `include` covers `src/**/*.test.ts`

3. **package.json scripts**
   - `"test": "vitest run"`
   - `"test:watch": "vitest"`
   - `"test:coverage": "vitest run --coverage"`

4. **src/setupTests.ts**
   - Import `@testing-library/jest-dom/vitest`
   - Mock `window.__TAURI_INTERNALS__` or `@tauri-apps/api/core` `invoke` for component tests that call Tauri commands.
   - Configure `userEvent` setup.

### First Tests (priority order)
1. **DashboardCell.svelte** (`src/lib/components/__tests__/DashboardCell.test.ts`)
   - Renders title and icon
   - Shows loading spinner when `loading=true`
   - Shows empty state with custom message/icon when `empty=true`
   - Renders children snippet when not loading/empty

2. **PriceBadge.svelte** (`src/lib/components/__tests__/PriceBadge.test.ts`)
   - Renders green/amber variants
   - Computes correct confidence tier (high/medium/low) and dot glyphs
   - Correct `aria-label` and `role="status"`

3. **CollectionView.svelte** (`src/lib/components/__tests__/CollectionView.test.ts`)
   - Mock `collectionStore` with `loading=true`, `items=[]`, and populated items
   - Empty state rendering
   - Gain/loss calculation and CSS class (`gain` vs `loss`)

4. **ProductCard.svelte** (later — not in first batch)
   - Requires mocking Tauri `invoke` and image blobs; more complex. Better as an integration test once the setup is proven.

### Integration Plan
- **Makefile**:
  - Add `test-frontend: npm run test` target
  - Update `test: test-app test-scraper test-frontend`
- **CI**:
  - Add `frontend` job (ubuntu-latest, node 22, `npm ci`, `npm run test`, `npm run test:coverage` optional)
  - Keep `rust` job unchanged
- **package.json**:
  - Add scripts as listed above

### Scope Estimate
- Config files: ~60 lines (vitest.config.ts, setupTests.ts, tsconfig/package.json edits)
- Component tests (3 files): ~200–280 lines
- Makefile + CI edits: ~30 lines
- **Total: ~300–400 lines**

### Risks
- **Svelte 5 snippet rendering**: `@testing-library/svelte` v5 supports snippets, but `{@render children?.()}` in tests requires passing a snippet prop. May need a small wrapper.
- **Tauri mocking**: `invoke` is imported from `@tauri-apps/api/core`. Vitest needs a module mock or `vi.mock`. If done wrong, all component tests that touch Tauri will fail.
- **Store mocking**: `collectionStore` is a `writable`. Easy to mock with `set`/`update`, but needs a pattern established in `setupTests.ts` or per-test.
- **Vite 6 + vitest 3 compatibility**: Should be fine (both are latest stable), but need to verify after install.
- **CI time**: Adding Node setup + npm install increases CI duration by ~1–2 min. Acceptable.

### Ready for Proposal
**Yes.** The path is clear: install tooling, configure vitest, write setup file with mocks, then add tests for DashboardCell, PriceBadge, and CollectionView. No architectural blockers.
