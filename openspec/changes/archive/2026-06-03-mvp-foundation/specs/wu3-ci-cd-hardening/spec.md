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

#### Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Valid catalog data | Artifact has all required fields per schema | `--validate-input` step runs | Passes, `--publish-index` proceeds |
| Malformed data | Artifact is missing required fields | `--validate-input` step runs | Fails, publish step skipped entirely |

### Requirement: Concurrency guard for release publishing

`release.yml` MUST define a `concurrency` block with `group: gh-pages-publish` and `cancel-in-progress: false`.

#### Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Two releases queued | Release A starts, Release B is triggered | A runs, B waits | A publishes to gh-pages, B runs after A finishes |
| Fast follow | Release B pushed while A is building | A still running | B waits for A to finish — no race on gh-pages |

## Acceptance Criteria

| Criterion | How to verify |
|-----------|---------------|
| `pip-audit` precedes scraper | Inspect `scrape.yml` step order — audit before `run_all.py` |
| `pip-audit` precedes tests | Inspect `ci.yml` step order — audit before `pytest` |
| Validation precedes publish | Inspect `scrape.yml` — `--validate-input` between `download-artifact` and `--publish-index` |
| Concurrency configured | Inspect `release.yml` — `concurrency` block present with `cancel-in-progress: false` |
