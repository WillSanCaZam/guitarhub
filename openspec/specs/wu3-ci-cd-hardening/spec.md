# CI/CD Hardening Specification

## Purpose

Reorder and augment CI/CD pipelines so security scanning gates execution, input validation gates publishing, and concurrent releases do not clobber each other.

## Requirements

### Requirement: pip-audit gates scraper execution

`scrape.yml` MUST run `pip-audit --desc on` immediately after `pip install` and BEFORE `run_all.py` execution. If `pip-audit` fails, the scraper MUST NOT run.

#### Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Vulnerable dependency | `pip install` installs a known-vulnerable package | `pip-audit` step runs | Workflow FAILS, scraper never executes |
| Clean dependencies | All installed deps are current | `pip-audit` step runs | Workflow continues to scraper step |

### Requirement: pip-audit gates test execution

`ci.yml` MUST run `pip-audit` before `pytest`. If `pip-audit` fails, the test suite MUST NOT run.

#### Scenario: Vulnerable dependency in CI

- GIVEN `ci.yml` installs Python dependencies
- WHEN `pip-audit` finds a vulnerability
- THEN the workflow fails before `pytest` executes

### Requirement: Input validation gates publishing

`scrape.yml` MUST add a validation step between `download-artifact` and `--publish-index` that runs `python scraper/run_all.py --validate-input --input-dir incoming/`. If validation fails, the publish step MUST be skipped.

The validation step MUST be named `validate-input` and MUST appear as a distinct job step in the workflow YAML. The step MUST use the same Python environment and dependencies as the scraper step. The `--validate-input` command MUST exit with code 0 when the input data is valid and code 1 when the input data is malformed.

(Previously: The validation requirement existed but was not explicitly tied to the `--validate-input` CLI flag and the exact step naming.)

#### Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Valid catalog data | Artifact has all required fields per schema | `--validate-input` step runs | Passes, `--publish-index` proceeds |
| Malformed data | Artifact is missing required fields | `--validate-input` step runs | Fails, publish step skipped entirely |
| Step order is correct | `scrape.yml` is inspected | Read job steps | `validate-input` step appears after `download-artifact` and before `--publish-index` |

---

### Requirement: `--validate-input` MUST be idempotent

The `--validate-input` command MUST be safe to run multiple times on the same input directory without side effects. It MUST only read and validate the input files; it MUST NOT modify the input directory or create output artifacts.

#### Scenario: Idempotent validation

- GIVEN `incoming/` contains valid data
- WHEN `--validate-input` is run twice in succession
- THEN both runs exit with code 0
- AND the `incoming/` directory contents are unchanged

---

### Requirement: Validation failure MUST produce actionable logs

When `--validate-input` fails, the step output MUST include the specific file path and the validation error (e.g., missing required field, invalid schema, malformed JSON). The logs MUST be visible in the GitHub Actions UI.

#### Scenario: Validation error details

- GIVEN `incoming/products.json` is missing the `sku` field
- WHEN `--validate-input` runs
- THEN the console output contains "incoming/products.json"
- AND the output contains the specific error "missing required field: sku"

### Requirement: Concurrency guard for release publishing

`release.yml` MUST define a `concurrency` block with `group: ${{ github.ref_name }}` and `cancel-in-progress: false`.

#### Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Different tags | Release v0.1.0 runs, v0.1.1 triggered | Both run | Both build in parallel — no cancellation |
| Same tag re-push | v0.1.0 pushed twice | First runs, second queues | First completes, second waits |

### Requirement: Build matrix covers Linux x86_64 only

`release.yml` MUST define a build matrix with 1 entry: `x86_64-unknown-linux-gnu` on `ubuntu-latest`. `fail-fast` MUST be `false`. Timeout MUST be 30 minutes.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Linux build succeeds | Tag push triggers workflow | 1 matrix job runs | Linux bundle produced |
| Linux build fails | System dep missing | Job fails | No release created, job reports error |

### Requirement: Bundle config MUST declare explicit targets

`tauri.conf.json` MUST include a `bundle` section with `targets: ["deb"]`, `identifier: "com.guitarhub.app"`, and valid `icon` paths.

#### Scenario: Bundle config present

- GIVEN `tauri.conf.json` is inspected
- THEN `bundle.active` is `true`
- AND `bundle.targets` includes `deb`
- AND `bundle.icon` references existing files

### Requirement: httpmock dev-dependency upgraded to 0.8.3

`Cargo.toml` MUST upgrade `httpmock` from `"0.7"` to `"0.8.3"`. After upgrade, `cargo test` MUST pass. Other Dependabot PRs are observed but not merged in this change.

#### Scenario: httpmock upgrade validated

- GIVEN `httpmock` is changed to `"0.8.3"` in `[dev-dependencies]`
- WHEN `cargo test` runs
- THEN all tests pass

### Requirement: Linux system deps installed conditionally

Only on `runner.os == 'Linux'`, the workflow MUST install `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libgirepository1.0-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `libsoup-3.0-dev`, `libjavascriptcoregtk-4.1-dev` via `sudo apt-get install -y -qq`.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Linux build | ubuntu-latest runner | apt-get step executes | 7 deps installed |
| macOS build | macos-13 or macos-latest | Skipped by `if: runner.os` | No apt operation |

### Requirement: Build pipeline npm ci → cargo test → cargo tauri build

Every build job MUST run `actions/setup-node@v4` (Node 22, cache: npm), then `npm ci`, then `cargo test` in `src-tauri/`, then `cargo tauri build --target ${{ matrix.target }}`. `TAURI_SKIP_SIGNING: true` MUST be set at the workflow `env` level.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Happy path | All deps installed | npm ci → cargo test → build | Bundles in `target/${{ matrix.target }}/release/bundle/` |
| Test failure | Rust test fails | cargo test runs | Build skipped, job fails |

### Requirement: Artifacts uploaded per target with empty-bundle guard

After build, each job MUST upload `src-tauri/target/${{ matrix.target }}/release/bundle/` via `actions/upload-artifact@v4` with `if-no-files-found: error`. Artifact name MUST be `guitarhub-${{ matrix.target }}`.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Bundles exist | Build succeeded | Upload step runs | Artifact stored |
| No bundles | Build produced no output | Upload step runs | Job errors — "No files found" |

### Requirement: Release creation from all bundle artifacts

A `create-release` job (needs: `build`) MUST download all `guitarhub-*` artifacts with `merge-multiple: true`, discover bundle files by extension (`.deb`, `.AppImage`, `.dmg`, `.msi`, `.zip`, `.tar.gz`), and run `gh release create` with `--generate-notes`. If no bundle files are found, the job MUST `exit 1` after printing a debug file listing.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| All 4 builds succeeded | 4 artifacts uploaded | Download → find → gh release create | Release with 4+ bundle files |
| Empty artifact set | Builds produced no bundles | `find` returns empty | Job fails with debug output |

### Requirement: Update endpoint pushed to gh-pages with retry

A `publish-update-endpoint` job (needs: `create-release`) MUST checkout `gh-pages`, run `python scripts/generate_latest.json.py ${{ github.ref_name }}`, commit, and push. On push failure, it MUST retry with `git pull --rebase` up to 3 times with 5s delay. Timeout MUST be 10 minutes.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Clean push | gh-pages is current | Generate → commit → push | latest.json updated |
| Push race | Concurrent push to gh-pages | Push fails → pull --rebase → retry | Push succeeds within 3 retries |
| Max retries exceeded | 3 consecutive push failures | Loop exhausts | Job fails with push error |

## Acceptance Criteria

| Criterion | How to verify |
|-----------|---------------|
| `pip-audit` precedes scraper | Inspect `scrape.yml` step order — audit before `run_all.py` |
| `pip-audit` precedes tests | Inspect `ci.yml` step order — audit before `pytest` |
| Validation precedes publish | Inspect `scrape.yml` — `--validate-input` between `download-artifact` and `--publish-index` |
| Concurrency configured | Inspect `release.yml` — `concurrency` block present with `cancel-in-progress: false` |
