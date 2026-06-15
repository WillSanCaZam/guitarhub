# Proposal: MVP Cleanup Before Release

## Intent

The MVP is substantially complete but has medium technical debt blocking v0.4.0: failing tests, unverified E2E, Svelte 4 store remnants, and a 323-line `+page.svelte` mixing concerns. This cleanup makes the codebase release-ready.

## Scope

### In Scope
- Run `make test` and fix all failures (Rust, Python, frontend)
- Verify E2E tests pass against debug build (`make test-e2e`)
- Migrate 1-2 stores from Svelte 4 `writable` to Svelte 5 runes as proof of concept
- Decompose `src/routes/+page.svelte` into smaller components if feasible

### Out of Scope
- New features or new source adapters
- Full Svelte 5 migration (only PoC stores)
- Performance optimization
- New E2E scenarios beyond existing ones

## Capabilities

### New Capabilities
None — this is cleanup/refactor, not new functionality.

### Modified Capabilities
None — no spec-level behavior changes. Existing specs (`frontend-testing`, `e2e-testing`, `ui`) already cover the expected behavior; this change makes the implementation match.

## Approach

1. **Test triage**: Run `make test`, categorize failures (flaky vs. real), fix real failures first
2. **E2E verification**: Run `make test-e2e` against debug build, fix environment or config issues
3. **Store migration PoC**: Pick `collectionStore` or `settingsStore` — convert from `writable()` to `$state` runes, verify no regressions
4. **Page decomposition**: Extract dashboard cells or logical sections from `+page.svelte` into child components

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src/routes/+page.svelte` | Modified | Decompose into smaller components |
| `src/lib/stores/` | Modified | Migrate 1-2 stores to Svelte 5 runes |
| `src/lib/components/` | New/Modified | New child components extracted from page |
| Test files | Modified | Fix failing tests, update mocks for rune stores |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Store migration breaks reactive subscriptions | Med | Migrate one store at a time, run full test suite after each |
| E2E fails due to env differences | Med | Document required env vars, add skip logic for missing deps |
| Page decomposition introduces render bugs | Low | Extract one section at a time, visual regression check |

## Rollback Plan

- **Test fixes**: Revert individual test file changes via git
- **Store migration**: Keep original `writable` stores as fallback; rune migration is additive
- **Page decomposition**: Git revert the component extraction commits; `+page.svelte` is preserved in history

## Dependencies

- `tauri-driver` installed for E2E verification
- Debug build compiles successfully (`cargo tauri build --debug --no-bundle`)

## Success Criteria

- [ ] `make test` passes with zero failures
- [ ] `make test-e2e` passes against debug build
- [ ] At least 1 store migrated to Svelte 5 runes with all tests passing
- [ ] `+page.svelte` reduced to under 200 lines (or clearly justified if not feasible)
- [ ] No regressions in existing component behavior
