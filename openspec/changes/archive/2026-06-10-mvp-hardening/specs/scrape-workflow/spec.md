# Delta for scrape-workflow

Relates to: `wu3-ci-cd-hardening` — scrape pipeline.

## Purpose

Fix the scrape workflow by ensuring the `incoming/` directory exists before the validation step, and deploying the catalog JSON artifact to GitHub Pages for consumption by the app.

## ADDED Requirements

### Requirement: `incoming/` directory MUST exist before scrape validation

The `scrape.yml` workflow MUST ensure the `incoming/` directory exists and contains the catalog JSON artifact before the `validate-input` step runs. The workflow MUST create `incoming/` with `mkdir -p incoming/` and copy the catalog artifact(s) into it using `cp catalog-${{ matrix.source }}.json incoming/`.

This provisioning MUST happen AFTER the `download-artifact` step and BEFORE the `validate-input` step.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Normal scrape | Catalog artifact downloaded | Workflow runs | `incoming/` created, catalog JSON copied, validation passes |
| `incoming/` missing | Fresh CI runner, no prior directory | Workflow runs | `mkdir -p incoming/` creates it, no error |

#### Scenario: Missing incoming/ directory is created

- GIVEN the CI runner has no `incoming/` directory
- WHEN the `mkdir -p incoming/` step executes
- THEN the directory is created without error
- AND the catalog JSON file is copied into it

### Requirement: Catalog JSON MUST deploy to GitHub Pages

After validation passes, the workflow MUST deploy the catalog JSON artifact to the `gh-pages` branch using `peaceiris/actions-gh-pages@v4` (or equivalent). The deploy step MUST:

- Publish the `incoming/` directory contents to `gh-pages`
- Use `GITHUB_TOKEN` with `contents: write` permission
- Specify `publish_dir: incoming/`
- Run ONLY if the `validate-input` step succeeds

The workflow MUST include a `permissions:` block at the job or workflow level granting `contents: write`.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Validation passes | `--validate-input` exits 0 | Deploy step runs | Catalog JSON published to `gh-pages` |
| Validation fails | `--validate-input` exits 1 | Deploy step runs | Deploy is skipped (due condition), job fails |
| Deploy succeeds | Valid catalog, gh-pages branch exists | `peaceiris/actions-gh-pages` runs | Catalog JSON accessible at Pages URL |

#### Scenario: Catalog published after valid scrape

- GIVEN the scrape completes and validates successfully
- WHEN the deploy action runs with `publish_dir: incoming/`
- THEN the catalog JSON is committed to the `gh-pages` branch
- AND the catalog is accessible at the GitHub Pages URL
