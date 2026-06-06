# Archive Report: MVP Polish and Release (v0.1.0)

**Archived**: 2026-06-05
**Change**: `mvp-polish-and-release`
**Status**: ✅ Completed (PASS WITH WARNINGS — warnings fixed)

---

## 1. What Was Accomplished

This change closed the remaining UI, testing, CI/CD, and documentation gaps identified in the pre-release audit so the MVP could ship as **v0.1.0**.

### Deliverables
- **UI Polish** — Fixed Settings save button no-op and added saved-feedback state; applied glassmorphism styling to bento grid cells via `.glassmorphism` class with `@supports` fallback; expanded PriceBadge tooltip to full 3-line confidence context.
- **Frontend Tests** — Added 4 new vitest component test files covering `ProductCard`, `Settings`, `PriceChart`, and the dashboard `+page`.
- **CI/CD & Docs** — Added `--validate-input` gate to `scraper/cli.py` and inserted the `validate-input` step in `scrape.yml`; rewrote CHANGELOG `[Unreleased]` into a dated `v0.1.0` release entry; added v0.1.0 badge to README; added GPL-3.0 `LICENSE` and `SPDX-License-Identifier` headers to all Rust and Python source files.
- **Collection Dashboard** — Added aggregate gain/loss summary to Dashboard Cell 8, computed reactively from `$collectionStore.items` with locale-aware formatting and color-coded indicators (green/red/neutral).

### Task Summary
All 15 tasks across 4 phases completed:
- Phase 1 (UI Polish): 5/5 ✅
- Phase 2 (Frontend Tests): 4/4 ✅
- Phase 3 (CI/CD & Docs): 6/6 ✅
- Phase 4 (Collection Dashboard): 3/3 ✅

---

## 2. Spec Deltas Summary

| Domain | Action | Details |
|--------|--------|---------|
| `bento-grid-ui` | Updated | 4 added req: glassmorphism class, WebKit fallback, focus-visible ring, border-radius 12px, grid gap 16px |
| `collection-ui` | Updated | Modified Cell 8 stats to include gain/loss; added reactive gain/loss and locale-aware formatting requirements |
| `frontend-test-coverage` | **Created** | New capability spec with 5 requirements covering ProductCard, Settings, PriceChart, +page, and test suite discovery |
| `frontend-testing` | Updated | Added 6 new requirements (REQ-TEST-7 through REQ-TEST-12) for component tests, mocking patterns, and file count |
| `repo-presentable` | Updated | Modified CHANGELOG requirement for v0.1.0 release section; added contributor credit and README badge requirements |
| `ui` | Updated | Modified tooltip to full 3-line context; added Settings save feedback and form-field binding requirements; added full-context aria-label scenario |
| `wu3-ci-cd-hardening` | Updated | Modified input validation requirement with explicit step naming and exit codes; added idempotency and actionable-logs requirements |

---

## 3. Design Decisions Implemented

| Decision | Location | Rationale |
|----------|----------|-----------|
| Glassmorphism via `@supports` with testable `.glassmorphism` class | `DashboardCell.svelte` | Reuses existing styles, satisfies testability, zero breaking change |
| Settings save button — explicit feedback over auto-save | `Settings.svelte` | Auto-save already exists per-field; explicit feedback gives user tangible confirmation without changing persistence model |
| Gain/loss computed in component, not backend | `+page.svelte` | Zero backend touch; reactive; uses existing data; no schema changes |
| CI validation — extend existing `scraper/cli.py` | `scraper/cli.py` | Keeps CLI surface small; reuses existing `CatalogFile` Pydantic model |

---

## 4. Test Results

| Suite | Count | Result |
|-------|-------|--------|
| Rust (`cargo test`) | 293 tests | ✅ PASS |
| Frontend (`npm run test`) | 32 tests | ✅ PASS |
| Python (`pytest`) | — | ✅ PASS |
| `clippy` | — | ✅ Clean (no warnings) |
| `make test` | Full project | ✅ PASS |

**Verification Status**: PASS WITH WARNINGS (warnings were fixed during this phase).

---

## 5. Known Deviations

| Deviation | Status | Note |
|-----------|--------|------|
| Settings `currency` and `threshold` fields are present in the component but the store integration is partial (no persistent `settingsStore` exists) | **Documented as future work** | The design notes that `settingsStore` does not exist in the codebase; the save button re-invokes per-field `invoke('save_setting')` and provides visual feedback only. A follow-up change should introduce a persistent `settingsStore` with schema-backed defaults. |
| `scrape.yml` spec references `download-artifact` and `--publish-index` steps that do not yet exist in the current workflow | **Documented as future work** | The `validate-input` step was added as a gate before the artifact upload boundary. If a true `publish-index` job is added later, the `validate-input` step should be moved there. |

---

## 6. Rollback Notes

- Each phase was delivered as a standalone commit. Rollback of any individual phase is `git revert <commit>`.
- No database schema changes were made — rollback is always non-destructive to user data.
- The CHANGELOG and README edits can be reverted independently without affecting app code.
- The LICENSE and SPDX header additions are legal/documentation only and do not affect runtime behavior.

---

## 7. Artifact Traceability

| Artifact | Location | Status |
|----------|----------|--------|
| Proposal | `archive/2026-06-05-mvp-polish-and-release/proposal.md` | ✅ Archived |
| Design | `archive/2026-06-05-mvp-polish-and-release/design.md` | ✅ Archived |
| Tasks | `archive/2026-06-05-mvp-polish-and-release/tasks.md` | ✅ Archived |
| Specs | `archive/2026-06-05-mvp-polish-and-release/specs/*/spec.md` | ✅ Archived |
| Delta Specs Merged | `openspec/specs/{domain}/spec.md` | ✅ Synced to source of truth |
| New Spec | `openspec/specs/frontend-test-coverage/spec.md` | ✅ Created |

---

## 8. Active Specs Remaining

All specs modified by this change remain active in `openspec/specs/` as they represent ongoing capabilities. The delta files have been archived.

---

*SDD Cycle Complete.*
