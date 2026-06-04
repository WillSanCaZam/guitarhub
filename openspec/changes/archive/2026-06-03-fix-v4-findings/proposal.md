# Proposal: fix-v4-findings — CI/CD Hardening

## Intent

The v4 plan review identified 4 CI/CD gaps: missing job timeouts causing hung builds, unpinned Python dev deps causing silent drift, unsafe bare `git push` in release workflow, and no concurrency cancellation on stale PR runs. Fixing these prevents wasted GH Actions minutes and reduces flaky releases.

## Scope

### In Scope
1. **timeout-minutes** — Add to 7 jobs across 3 workflows
2. **requirements-dev.txt** — Create pinned dev dependency file; update CI to install via file
3. **release.yml git push retry** — Retry loop with `git pull --rebase && git push`
4. **concurrency group** — Add `cancel-in-progress: true` to ci.yml

### Out of Scope
- Rust/Python code-level fixes from v4 review — deferred to component builds
- Functional workflow changes beyond the 4 items above
- Composite actions, reusable workflows, or matrix builds

## Capabilities

No spec-level changes — pure CI/CD config hardening.

### New Capabilities

None.

### Modified Capabilities

None.

## Approach

Four independent changes:
1. **timeout-minutes**: Add job-level `timeout-minutes:` — python=10, rust=15, scrape=15, publish=10, build=30, update-endpoint=10.
2. **requirements-dev.txt**: Pin versions from current pip-resolved deps. CI switches from bare `pip install` to `pip install -r requirements-dev.txt`.
3. **git push retry**: Wrap `git commit && git push` in `for i in {1..3}; do ... git pull --rebase && git push && break; done`.
4. **concurrency group**: Add `concurrency: ci-${{ github.ref }}` with `cancel-in-progress: true` at workflow level in ci.yml.

All reversible — revert the commit.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `.github/workflows/ci.yml` | Modified | Concurrency + timeout (2 jobs) |
| `.github/workflows/scrape.yml` | Modified | timeout (2 jobs) |
| `.github/workflows/release.yml` | Modified | timeout (2 jobs) + git push retry |
| `requirements-dev.txt` | **New** | Pinned Python dev deps |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| timeout too tight for slow builds | Low | Set at p99 × 1.5; easy to adjust |
| git retry masks real failures | Low | Loop exits with last error code |
| Concurrency cancels desired in-flight run | Low — intended | Saves minutes, re-runs on latest push |

## Rollback Plan

Revert the commit. `requirements-dev.txt` is inert if CI stops referencing it.

## Dependencies

None. All features (`timeout-minutes`, `concurrency`) are built-in GitHub Actions.

## Success Criteria

- [ ] All 7 job timeouts present with correct values
- [ ] `pip install -r requirements-dev.txt` passes in CI
- [ ] Concurrency cancels stale PR runs on new push
- [ ] git retry loop compiles and falls back to failure on 3rd retry
