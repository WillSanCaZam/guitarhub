# Design: WU3 — CI/CD Hardening

## Technical Approach

Reorder and augment three GitHub Actions workflows with security gates. Two reordering fixes (pip-audit before execution), one new validation step (--validate-input), and one concurrency guard. Each change is a surgical YAML edit — no restructuring, no new jobs.

## Architecture Decisions

### Decision: pip-audit as pre-execution gate (not post-hoc report)
- **Choice**: Run `pip-audit` between `pip install` and the actual execution step (scraper or tests)
- **Rationale**: If dependencies are vulnerable, running the code against them is a security risk. Pre-execution gating means the workflow fails fast before any processing. The existing order (execute first, audit after) defeats the purpose of auditing — the vulnerable code already ran

### Decision: `--validate-input` as a dedicated step, not merged into `--publish-index`
- **Choice**: Separate `python scraper/run_all.py --validate-input --input-dir incoming/` step
- **Alternatives**: Add validation inside the existing `--publish-index` command; use a separate validation script
- **Rationale**: A dedicated step makes it explicit in the workflow: "validation must pass before publish." If validation and publish were merged, a bug in the publish path could execute on invalid data. Separation means the validation step MUST succeed for publish to run (GitHub Actions stops on non-zero exit by default)

### Decision: `cancel-in-progress: false` for release
- **Choice**: Concurrency group `gh-pages-publish` with `cancel-in-progress: false`
- **Rationale**: Release publishing to `gh-pages` is not idempotent — two concurrent writes to the same branch produce a race. `cancel-in-progress: false` means a fast-follow release waits for the current one to finish, rather than cancelling it mid-publish (which would leave gh-pages in a broken state)

## File Changes

### `.github/workflows/scrape.yml` — Modify

**Current state**: pip-audit runs AFTER `run_all.py`, no input validation before publish.

```yaml
      - run: python scraper/run_all.py --source ${{ matrix.source }} --output-dir artifacts/
        env:
          GITHUB_RUN_ID: ${{ github.run_id }}
      - run: pip install pip-audit --break-system-packages
      - run: pip-audit -r scraper/requirements.txt
```

**Target change**: Move pip-audit between `pip install` and scraper execution.

```yaml
      - run: pip install -r scraper/requirements.txt --break-system-packages
      - run: pip install pip-audit --break-system-packages
      - run: pip-audit -r scraper/requirements.txt
      - run: python scraper/run_all.py --source ${{ matrix.source }} --output-dir artifacts/
        env:
          GITHUB_RUN_ID: ${{ github.run_id }}
```

**Exact diff**:
1. Move `pip install pip-audit` and `pip-audit -r scraper/requirements.txt` BEFORE the `run_all.py` step
2. Keep the `upload-artifact` step in the same position (after execution)
3. The `if: always()` on upload-artifact means we still get partial artifacts even if audit or scraper fails (not affected by this change)

**Publish job**: Add validation step before `--publish-index`.

```yaml
      - uses: actions/download-artifact@v4
        with:
          path: incoming/
      - run: python scraper/run_all.py --validate-input --input-dir incoming/
      - run: python scraper/run_all.py --publish-index --input-dir incoming/
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          GITHUB_REPOSITORY: ${{ github.repository }}
```

If `--validate-input` exits non-zero, the workflow stops and `--publish-index` never runs (GitHub Actions default behavior: fail on error).

### `.github/workflows/ci.yml` — Modify

**Current state**: pip-audit runs AFTER pytest.

```yaml
      - run: pip install -r requirements-dev.txt --break-system-packages
      - run: ruff check scraper/
      - run: mypy scraper/ --strict
      - run: pytest scraper/tests/unit scraper/tests/contract -v
      - run: pip-audit -r scraper/requirements.txt
```

**Target change**: Move pip-audit before pytest (and after the linters — linters are safe to run on any deps).

```yaml
      - run: pip install -r requirements-dev.txt --break-system-packages
      - run: ruff check scraper/
      - run: mypy scraper/ --strict
      - run: pip-audit -r scraper/requirements.txt
      - run: pytest scraper/tests/unit scraper/tests/contract -v
```

### `.github/workflows/release.yml` — Modify

**Current state**: No concurrency group on build or publish-update-endpoint jobs.

Add concurrency block at the top level (after `on:` block):

```yaml
concurrency:
  group: gh-pages-publish
  cancel-in-progress: false
```

**Exact insertion point** (after line 5):
```yaml
on:
  push:
    tags: ['v*']

concurrency:                          # ◀ INSERT
  group: gh-pages-publish             # ◀
  cancel-in-progress: false           # ◀

jobs:
```

This applies to ALL jobs in the workflow — the build matrix and the publish step share the same concurrency group. Two release workflows with different tags will queue, not race.

## Sequence

All three files are independent YAML edits — apply in any order:

1. `scrape.yml` — two edits (pip-audit reorder + --validate-input step)
2. `ci.yml` — one edit (pip-audit reorder)
3. `release.yml` — one edit (concurrency block)

## Risks

| Risk | Likelihood | Mitigation |
|------|-----------|------------|
| pip-audit false positive blocks CI | Low | `pip-audit` only fails on known advisories; threshold set by the advisory database, not by us |
| --validate-input fails on format mismatch | Low | The validation step uses the same codebase as publish — if it fails, data IS malformed and should not be published |
| concurrency blocks legitimate parallel releases | Low | `cancel-in-progress: false` means queuing, not cancelling. Two releases on different tags both eventually publish |
| Merge conflicts across workflow branches | Low | YAML edits on independent lines; no structural conflicts expected |

## Testing Approach

| Layer | What | How |
|-------|------|-----|
| Review | Step order in `scrape.yml` | Confirm: pip install → pip-audit → run_all.py |
| Review | Step order in `ci.yml` | Confirm: pip install → linters → pip-audit → pytest |
| Review | Validation in `scrape.yml` | Confirm: download-artifact → --validate-input → --publish-index |
| Review | Concurrency in `release.yml` | Confirm: `concurrency` block present, `cancel-in-progress: false` |
| Dry-run | All workflows | Push to a branch, trigger `workflow_dispatch` on scrape.yml, push a PR to trigger ci.yml, push a tag to dry-run release.yml |
