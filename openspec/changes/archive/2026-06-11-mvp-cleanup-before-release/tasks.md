# Tasks: MVP Cleanup Before Release

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 150–250 |
| 400-line budget risk | Low |
| Chained PRs recommended | No |
| Suggested split | Single PR |
| Delivery strategy | ask-on-risk |
| Chain strategy | size-exception |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: size-exception
400-line budget risk: Low

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | Full cleanup | PR 1 | Tests pass, stores migrated, styles extracted — all in one |

## Phase 1: Test Triage & E2E Verification

- [x] 1.1 Run `make test` and confirm all 124 tests pass (49 Python + 75 frontend). If any fail, categorize: flaky vs. real. Fix real failures.
- [x] 1.2 Run `make test-e2e`. If `tauri-driver` not installed, document the env gap in `docs/CONTRIBUTING.md` under E2E prerequisites. Verify debug build compiles: `cargo tauri build --debug --no-bundle`.
- [x] 1.3 Run `make lint` — fix any clippy/ruff/svelte-check warnings surfaced during triage.

## Phase 2: Store Migration PoC (Svelte 4 → Svelte 5 Runes)

- [x] 2.1 Migrate `src/lib/stores/collection.ts` from `writable()` to Svelte 5 `$state` runes. Rename file to `collection.svelte.ts`. Export reactive state object + async action functions. Keep the same public API shape.
- [x] 2.2 Update all consumers of `collectionStore`: `src/routes/+page.svelte`, `src/lib/components/CollectionStatsCell.svelte`, `src/lib/components/SearchPanel.svelte`. Replace `$collectionStore` subscription syntax with direct state access.
- [x] 2.3 Update `src/lib/stores/__tests__/collection.test.ts` to work with the new rune-based store. Run `make test-frontend` — verify zero regressions.
- [x] 2.4 Migrate `src/lib/stores/filter.ts` from `writable()` to `$state` runes. Rename to `filter.svelte.ts`. Update consumers: `+page.svelte`, `FilterBar.svelte`, `SearchPanel.svelte`. Update `filter.test.ts`.

## Phase 3: Page Decomposition — Extract Styles

- [x] 3.1 Extract the 162-line `<style>` block from `src/routes/+page.svelte` into `src/lib/styles/page.css`. Import it in the page. This alone cuts `+page.svelte` from 323 → ~160 lines.
- [x] 3.2 Extract the `loadDashboard()` async function from `+page.svelte` script into a new `src/lib/stores/dashboard.ts` helper (or merge into existing `dashboard.ts`). Page script drops to ~30 lines.
- [x] 3.3 Run `make test` and `make lint`. Verify visual rendering unchanged. Update `src/routes/__tests__/page.test.ts` if imports changed.

## Phase 4: Final Verification & Documentation

- [x] 4.1 Run full `make test && make lint && make audit`. All must pass.
- [x] 4.2 Update `docs/CONTRIBUTING.md` if E2E env requirements changed (step 1.2).
- [x] 4.3 Verify `+page.svelte` is under 200 lines (target from success criteria). If not, document why.
