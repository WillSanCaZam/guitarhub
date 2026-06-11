# Delta for wu3-ci-cd-hardening

> **Change**: ci-pipeline-fix-all-issues — Fix pip-audit virtualenv, release caching, retry exit code

## MODIFIED Requirements

### Requirement: pip-audit gates scraper execution

`scrape.yml` MUST run `pip-audit --desc on` immediately after `pip install` and BEFORE `run_all.py` execution. If `pip-audit` fails, the scraper MUST NOT run. The virtualenv MUST be activated before `pip-audit` runs so it audits project dependencies, not system Python packages.

(Previously: `pip-audit` ran outside the virtualenv, auditing system Python packages instead of project deps.)

#### Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Vulnerable dependency | `.venv` activated, vulnerable package installed | `pip-audit` step runs | Workflow FAILS, scraper never executes |
| Clean dependencies | `.venv` activated, all deps current | `pip-audit` step runs | Workflow continues to scraper step |
| Wrong scope without venv | `.venv` NOT activated | `pip-audit` runs | Audits system packages — INCORRECT behavior |

### Requirement: pip-audit gates test execution

`ci.yml` MUST run `pip-audit` before `pytest`. If `pip-audit` fails, the test suite MUST NOT run. The Python job MUST create and activate `.venv` before `pip install` and `pip-audit`.

(Previously: `ci.yml` Python job ran `pip install` and `pip-audit` without a virtualenv.)

#### Scenario: Vulnerable dependency in CI

- GIVEN `ci.yml` creates `.venv` and installs Python dependencies
- WHEN `pip-audit` finds a vulnerability
- THEN the workflow fails before `pytest` executes

#### Scenario: Python tests run in virtualenv

- GIVEN the `ci.yml` Python job starts
- WHEN the virtualenv setup step runs
- THEN `.venv` is created and activated
- AND `pip install` installs into `.venv`
- AND `pytest` runs using `.venv` Python

### Requirement: Update endpoint pushed to gh-pages with retry

A `publish-update-endpoint` job (needs: `create-release`) MUST checkout `gh-pages`, run `python scripts/generate_latest.json.py ${{ github.ref_name }}`, commit, and push. On push failure, it MUST retry with `git pull --rebase` up to 3 times with 5s delay. After exhausting all retries, the step MUST run `exit 1` to fail the job. Timeout MUST be 10 minutes.

(Previously: retry loop did not exit non-zero after exhausting attempts, allowing the job to succeed silently despite push failures.)

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Clean push | gh-pages is current | Generate → commit → push | latest.json updated |
| Push race | Concurrent push to gh-pages | Push fails → pull --rebase → retry | Push succeeds within 3 retries |
| Max retries exceeded | 3 consecutive push failures | Loop exhausts → `exit 1` | Job FAILS with explicit error |
