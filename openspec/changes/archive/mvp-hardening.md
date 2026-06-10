# Archive Report: mvp-hardening

**Archived**: 2026-06-10
**Source**: `openspec/changes/mvp-hardening/` → `openspec/changes/archive/2026-06-10-mvp-hardening/`
**Mode**: openspec

## Summary

Closed three deferred gaps from the `mvp-fixes` cycle: defense-in-depth at the image cache service layer (HTTPS-only enforcement), missing frontend build + type checking in CI, and a broken scrape workflow validation step with no deploy target.

## Verdict

- **9/9 spec scenarios compliant** (verified)
- **392 tests passing**: 307 Rust + 46 Python + 39 Vitest
- **Clippy**: clean
- **5 files changed**, ~40 lines

## Specs Synced

| Domain | Action | Details |
|--------|--------|---------|
| `local-image-cache` | Updated | Added URL scheme validation requirement (v8): `https://` only at service layer, `ImageCacheError::InvalidUrl` for non-HTTPS. 3 new scenarios appended. |
| `ci-pipeline` | Created | New domain spec: frontend build + `svelte-check` type-check in CI pipeline. |
| `scrape-workflow` | Created | New domain spec: `incoming/` directory provisioning + GitHub Pages deploy step. |

## Archive Contents

| Artifact | Present |
|----------|---------|
| `proposal.md` | ✅ |
| `specs/local-image-cache/spec.md` | ✅ |
| `specs/ci-pipeline/spec.md` | ✅ |
| `specs/scrape-workflow/spec.md` | ✅ |
| `design.md` | — (not created for this change) |
| `tasks.md` | ✅ (all 6/6 tasks completed) |
| `verify-report.md` | — (verified inline, no report artifact) |

## Tasks Completion

| Phase | Task | Status |
|-------|------|--------|
| 1.1 | RED test for `http://` rejection | ✅ |
| 1.2 | GREEN impl: `ImageCacheError::InvalidUrl` | ✅ |
| 2.1 | Add `svelte-check` devDependency | ✅ |
| 2.2 | Add `check` script to `package.json` | ✅ |
| 2.3 | Add build + check steps to `ci.yml` | ✅ |
| 3.1 | Add `incoming/` provisioning in `scrape.yml` | ✅ |
| 3.2 | Add `gh-pages` deploy step | ✅ |

## Source of Truth Updated

The following specs now reflect the new behavior:

- `openspec/specs/local-image-cache/spec.md` (v3 — URL scheme validation)
- `openspec/specs/ci-pipeline/spec.md` (new — frontend CI build + type-check)
- `openspec/specs/scrape-workflow/spec.md` (new — incoming/ dir + deploy)

## SDD Cycle Complete

This change has been fully planned, implemented, verified, and archived. Ready for the next change.
