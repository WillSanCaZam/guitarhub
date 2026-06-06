# Delta for wu3-ci-cd-hardening

## MODIFIED Requirements

### Requirement: Input validation gates publishing

`scrape.yml` MUST add a validation step between `download-artifact` and `--publish-index` that runs `python scraper/run_all.py --validate-input --input-dir incoming/`. If validation fails, the publish step MUST be skipped.

The validation step MUST be named `validate-input` and MUST appear as a distinct job step in the workflow YAML. The step MUST use the same Python environment and dependencies as the scraper step. The `--validate-input` command MUST exit with code 0 when the input data is valid and code 1 when the input data is malformed.

(Previously: The validation requirement existed but was not explicitly tied to the `--validate-input` CLI flag and the exact step naming.)

#### Scenario: Valid catalog data passes validation

- GIVEN the `download-artifact` step has populated `incoming/` with valid JSON files
- WHEN the `validate-input` step runs `python scraper/run_all.py --validate-input --input-dir incoming/`
- THEN the command exits with code 0
- AND the `--publish-index` step proceeds

#### Scenario: Malformed data fails validation

- GIVEN the `incoming/` directory contains a JSON file missing required fields
- WHEN the `validate-input` step runs
- THEN the command exits with code 1
- AND the `--publish-index` step is skipped entirely

#### Scenario: Step order is correct

- GIVEN `scrape.yml` is inspected
- WHEN reading the job steps
- THEN the `validate-input` step appears after `download-artifact`
- AND the `validate-input` step appears before `--publish-index`

## ADDED Requirements

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
