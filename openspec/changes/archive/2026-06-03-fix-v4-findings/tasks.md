# Tasks: fix-v4-findings — CI/CD Hardening

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~40 |
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
| 1 | Dev dependency pinning + ci.yml update | PR 1 (sole) | Foundation — reqs file then consume it |
| 2 | Workflow hardening (timeouts, concurrency, retry) | PR 1 (sole) | 4 independent YAML files, no deps on each other |

**Note**: Tasks 1.1 and 2.1 both touch `ci.yml` — merge into one apply session to avoid conflicts.

## Phase 1: Foundation

- [x] 1.1 **Create `requirements-dev.txt` with pinned dev deps** — file `requirements-dev.txt`. Pin versions for ruff, mypy, pytest, pytest-asyncio, jsonschema, pip-audit (resolved from current environment). Verification: `pip install -r requirements-dev.txt --dry-run --break-system-packages` succeeds.

- [x] 1.2 **Update `ci.yml` to consume requirements-dev.txt** — file `.github/workflows/ci.yml`. Replace bare `pip install ruff mypy...` with `pip install -r requirements-dev.txt --break-system-packages`. Verification: `yamllint` passes, diff shows the line swap.

## Phase 2: Workflow Hardening

- [x] 2.1 **Add concurrency group + timeouts to `ci.yml`** — file `.github/workflows/ci.yml`. Add `concurrency: ci-${{ github.ref }}` + `cancel-in-progress: true` at workflow level. Add `timeout-minutes: 10` to python job, `timeout-minutes: 15` to rust job. Verification: `yamllint` passes, all 3 additions present.

- [x] 2.2 **Add timeouts to `scrape.yml`** — file `.github/workflows/scrape.yml`. Add `timeout-minutes: 15` to scrape job, `timeout-minutes: 10` to publish job. Verification: `yamllint` passes, both fields present.

- [x] 2.3 **Add timeouts + git push retry to `release.yml`** — file `.github/workflows/release.yml`. Add `timeout-minutes: 30` to build job, `timeout-minutes: 10` to publish-update-endpoint job. Replace bare `git push` with 3-attempt retry loop (`for i in 1 2 3; do git pull --rebase && git push && break; sleep 5; done`). Verification: `yamllint` passes, retry loop compiles as valid YAML string.

- [x] 2.4 **Create `e2e.yml` with timeout-minutes** — file `.github/workflows/e2e.yml`. Create weekly scheduled workflow per plan spec: schedule cron `0 4 * * 1`, `timeout-minutes: 30`, steps for build, integration tests, tauri-driver E2E. Verification: `yamllint` passes, workflow renders correctly in GitHub Actions preview.
