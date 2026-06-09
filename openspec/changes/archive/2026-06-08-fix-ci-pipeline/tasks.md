# Tasks: Fix CI Pipeline Failures

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~25-30 |
| 400-line budget risk | Low |
| Chained PRs recommended | No |
| Suggested split | Single PR |
| Delivery strategy | ask-on-risk |
| Chain strategy | pending |

```
Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: pending
400-line budget risk: Low
```

## Phase 1: CI Configuration Changes

- [x] 1.1 Add `[tool.mypy]` section to `scraper/pyproject.toml` — `plugins = ["pydantic.v2.mypy_plugin"]`, `strict = true`, `warn_unused_ignores = true`, with per-module override for `scraper.tests.*` allowing untyped defs/decorators/calls
- [x] 1.2 Replace `--private-key` with `--write-keys` on line 44 of `.github/workflows/ci.yml` in the `tauri signer generate` dry-run step
- [x] 1.3 Insert apt-get step (`Install system dependencies (Tauri)` with `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`) before `cargo clippy` in the `rust` job of `.github/workflows/ci.yml`

## Phase 2: Verification

- [x] 2.1 Run `make test` (test-app, test-scraper, test-frontend) — confirm zero regressions
- [x] 2.2 Run `cd scraper && mypy . --strict` — verify mypy config exits 0
