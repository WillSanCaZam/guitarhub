# Delta for ci-pipeline-fixes

> **New domain** — CI workflow corrections for `ci.yml`, `scrape.yml`, `e2e.yml`, `release.yml`.

## Purpose

Fix CI-breaking issues: stale lockfile, missing system deps, virtualenv activation, and release reliability.

## Requirements

### Requirement: Single lockfile enforced

The repository MUST contain exactly one lockfile (`package-lock.json`). A stale `pnpm-lock.yaml` MUST be removed to prevent `npm ci` failures from dual-lockfile conflicts.

#### Scenario: npm ci with single lockfile

- GIVEN the working tree contains `package-lock.json` and no `pnpm-lock.yaml`
- WHEN `npm ci` runs in CI
- THEN the install completes successfully without lockfile conflict errors

#### Scenario: pnpm-lock.yaml absent

- GIVEN the repo is checked out
- WHEN `ls pnpm-lock.yaml` runs
- THEN the file does not exist (exit code non-zero)

### Requirement: e2e.yml MUST install all required system dependencies

The `e2e.yml` workflow MUST install `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `libsoup-3.0-dev`, `libjavascriptcoregtk-4.1-dev` via `sudo apt-get install -y` before the E2E build step.

#### Scenario: E2E build succeeds with system deps

- GIVEN the `e2e.yml` workflow runs on `ubuntu-latest`
- WHEN the "Install system dependencies" step executes
- THEN all 5 packages are installed without errors
- AND the subsequent `cargo tauri build --debug` step succeeds

#### Scenario: Missing system dep causes build failure

- GIVEN `libgtk-3-dev` is not installed
- WHEN `cargo tauri build --debug` runs
- THEN the build fails with a missing pkg-config error

### Requirement: scrape.yml MUST activate virtualenv before pip-audit

`scrape.yml` MUST activate `.venv` before running `pip-audit --desc on`. The virtualenv activation MUST occur in the same step or a preceding step so that `pip-audit` audits the project's installed dependencies, not system Python packages.

#### Scenario: pip-audit audits project deps

- GIVEN `.venv` is activated and project deps are installed
- WHEN `pip-audit --desc on` runs
- THEN the audit scans project dependencies, not system packages
- AND the step exits with code 0 for clean deps

#### Scenario: pip-audit without virtualenv audits wrong scope

- GIVEN `.venv` is NOT activated
- WHEN `pip-audit` runs
- THEN it audits system Python packages instead of project deps

### Requirement: ci.yml Python job MUST use virtualenv

The `ci.yml` Python job MUST create and activate `.venv` before `pip install` and `pip-audit`. All Python steps in the job MUST run inside the activated virtualenv.

#### Scenario: Python tests run in virtualenv

- GIVEN the `ci.yml` Python job starts
- WHEN the virtualenv setup step runs
- THEN `.venv` is created and activated
- AND `pip install` installs into `.venv`
- AND `pytest` runs using `.venv` Python

### Requirement: release.yml MUST include Cargo caching

`release.yml` MUST use `Swatinem/rust-cache@v2` in each build job to cache `~/.cargo/registry`, `~/.cargo/git`, and `src-tauri/target/`. The cache key MUST include the target triple and a hash of `Cargo.lock`.

#### Scenario: First build populates cache

- GIVEN no existing Rust cache
- WHEN the release build job runs
- Then `Swatinem/rust-cache@v2` restores empty cache, builds, then saves cache

#### Scenario: Second build restores cache

- GIVEN a cached Rust build from a previous run
- WHEN the release build job runs
- Then `Swatinem/rust-cache@v2` restores the cache
- AND build time is reduced by at least 30%

### Requirement: release.yml push retry MUST exit non-zero on failure

After exhausting all retry attempts for `git push` to `gh-pages`, the step MUST run `exit 1` to fail the job. The retry loop MUST NOT silently succeed after failures.

#### Scenario: Push fails all retries

- GIVEN `git push` fails 3 consecutive times
- WHEN the retry loop exhausts
- THEN the step runs `exit 1`
- AND the job reports failure

#### Scenario: Push succeeds on retry

- GIVEN `git push` fails on first attempt
- WHEN `git pull --rebase` and retry succeed
- THEN the job completes successfully
