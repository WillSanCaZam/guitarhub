# Proposal: MVP Polish and Release (v0.1.0)

## Intent

Close the remaining UI, testing, CI/CD, and documentation gaps identified in the pre-release audit so the MVP can ship as v0.1.0. No schema or architecture changes.

## Scope

### In Scope
- Fix Settings save button no-op bug and add saved feedback
- Apply glassmorphism styling to bento grid cells per `bento-grid-ui` spec
- Expand PriceBadge tooltip to full 3-line confidence context per `ui` spec
- Add frontend tests for ProductCard, Settings, PriceChart, and +page
- Write proper v0.1.0 CHANGELOG entry summarizing MVP features
- Add `--validate-input` gate to `scrape.yml` before publishing
- Add collection gain/loss summary to Dashboard Cell 8

### Out of Scope
- New features or user-facing functionality beyond polish
- Backend schema changes, migrations, or IPC contract changes
- Performance optimization or bundle-size work

## Capabilities

### New Capabilities
- `frontend-test-coverage`: Unit tests for ProductCard, Settings, PriceChart, +page

### Modified Capabilities
- `bento-grid-ui`: Glassmorphism CSS for grid cells (REQ currently unmet)
- `ui`: Full 3-line PriceBadge tooltip (currently v1 partial)
- `frontend-testing`: Expand test suite with four new component tests
- `wu3-ci-cd-hardening`: Add `--validate-input` gate in `scrape.yml`
- `repo-presentable`: Update CHANGELOG with v0.1.0 release notes
- `collection-ui`: Add gain/loss summary to Dashboard Cell 8

## Approach

Four sequential phases, each independently verifiable:

1. **UI Polish** — Remove/fix Settings save button, add glassmorphism cell styles to `+page.svelte`, and expand PriceBadge `title` attribute to 3 lines using existing IPC payload fields.
2. **Frontend Tests** — Write vitest component tests following the existing `DashboardCell.test.ts` pattern. Mock Tauri invoke and stores.
3. **CI/CD & Docs** — Insert `--validate-input` step in `scrape.yml` between download-artifact and publish. Rewrite CHANGELOG `[Unreleased]` into a v0.1.0 release entry.
4. **Collection Dashboard** — Compute aggregate gain/loss in Cell 8 using `collectionStore.stats` plus `purchase_price` totals from items.

Estimated changed lines: ~400 (additions + deletions).

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src/lib/components/Settings.svelte` | Modified | Fix save button no-op; add saved-feedback state |
| `src/routes/+page.svelte` | Modified | Add glassmorphism cell backgrounds |
| `src/lib/components/PriceBadge.svelte` | Modified | Expand tooltip to 3 lines |
| `src/lib/components/__tests__/` | New | ProductCard, Settings, PriceChart, +page tests |
| `CHANGELOG.md` | Modified | v0.1.0 release entry |
| `.github/workflows/scrape.yml` | Modified | `--validate-input` gate before publish |
| `src/lib/components/CollectionView.svelte` | Modified | Add gain/loss summary to Cell 8 |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Glassmorphism breaks on older WebKit | Low | Fallback to solid semi-transparent background already in spec |
| New frontend tests flake due to async invoke mocks | Low | Use `vi.waitFor` and stable test IDs |

## Rollback Plan

Each phase is a standalone commit. If any phase fails verification, revert that commit only. No schema changes means rollback is always `git revert <commit>`. The CHANGELOG edit can be reverted independently without affecting app code.

## Dependencies

- None. All work is frontend, docs, and CI YAML. Zero infrastructure cost.

## Success Criteria

- [ ] Settings save button removed or shows "Saved" feedback on click
- [ ] Bento grid cells render `backdrop-filter: blur(12px)` with semi-transparent backgrounds
- [ ] PriceBadge tooltip shows 3 lines: confidence tier, data points/sources/days, and price summary
- [ ] `npm run test` passes with 4+ new frontend tests
- [ ] CHANGELOG contains a dated v0.1.0 section with MVP feature summary
- [ ] `scrape.yml` has `--validate-input` step that gates `--publish-index`
- [ ] Dashboard Cell 8 displays collection gain/loss alongside total value
