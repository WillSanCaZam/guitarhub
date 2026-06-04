# Archive Report: fix-v4-findings — CI/CD Hardening

**Archived on**: 2026-06-03
**Archive location**: `openspec/changes/archive/2026-06-03-fix-v4-findings/`

---

## Summary

The v4 plan review identified 4 CI/CD gaps: missing job timeouts causing hung builds, unpinned Python dev deps causing silent drift, unsafe bare `git push` in the release workflow, and no concurrency cancellation on stale PR runs. This change addressed all 4 gaps across 5 files with zero functional impact.

---

## What Was Done

| # | Task | Files | Status |
|---|------|-------|--------|
| 1.1 | Create `requirements-dev.txt` with pinned dev deps | `requirements-dev.txt` | ✅ |
| 1.2 | Update `ci.yml` to consume `requirements-dev.txt` | `.github/workflows/ci.yml` | ✅ |
| 2.1 | Add concurrency group + timeouts to `ci.yml` | `.github/workflows/ci.yml` | ✅ |
| 2.2 | Add timeouts to `scrape.yml` | `.github/workflows/scrape.yml` | ✅ |
| 2.3 | Add timeouts + git push retry to `release.yml` | `.github/workflows/release.yml` | ✅ |
| 2.4 | Create `e2e.yml` with timeout-minutes | `.github/workflows/e2e.yml` | ✅ |

**Total**: 6/6 tasks completed and verified.

---

## Files Changed

| File | Action | Description |
|------|--------|-------------|
| `.github/workflows/ci.yml` | Modified | Concurrency group + cancel-in-progress; `timeout-minutes` on Python (10) and Rust (15) jobs; `pip install -r requirements-dev.txt` |
| `.github/workflows/scrape.yml` | Modified | `timeout-minutes` on scrape (15) and publish (10) jobs |
| `.github/workflows/release.yml` | Modified | `timeout-minutes` on build (30) and publish-update-endpoint (10) jobs; bare `git push` replaced with 3-attempt retry loop |
| `.github/workflows/e2e.yml` | **New** | Weekly scheduled workflow (cron `0 4 * * 1`), `timeout-minutes: 30`, build + integration tests + tauri-driver E2E |
| `requirements-dev.txt` | **New** | Pinned versions for ruff, mypy, pytest, pytest-asyncio, jsonschema, pip-audit |

---

## Verification Result

**Status**: ✅ PASS — all 6 tasks verified successfully

- **CRITICAL findings**: 0
- **WARNING findings**: 0

Verification covered:
- `yamllint` passing on all 4 workflow files
- `pip install -r requirements-dev.txt --dry-run --break-system-packages` succeeding
- All 7 `timeout-minutes` fields present with correct values
- Concurrency group with `cancel-in-progress: true` in `ci.yml`
- Git retry loop compiling as valid YAML string
- `e2e.yml` rendering correctly as a valid GitHub Actions workflow

---

## Deferred Items

None. All in-scope items from the proposal were completed. The following remain out of scope (per the original proposal):

- Rust/Python code-level fixes from v4 review — deferred to component builds
- Functional workflow changes beyond the 4 CI/CD items
- Composite actions, reusable workflows, or matrix builds

---

## Artifacts

| Artifact | Original Path | Archived Path |
|----------|---------------|---------------|
| Proposal | `openspec/changes/fix-v4-findings/proposal.md` | `openspec/changes/archive/2026-06-03-fix-v4-findings/proposal.md` |
| Tasks | `openspec/changes/fix-v4-findings/tasks.md` | `openspec/changes/archive/2026-06-03-fix-v4-findings/tasks.md` |

No spec-level artifacts were created (pure CI/CD config hardening — no spec changes).
