# Delta for e2e-testing

> **Change**: ci-pipeline-fix-all-issues — Add missing system deps to E2E workflow

## MODIFIED Requirements

#### REQ-E2E-3: CI workflow runs E2E tests on Ubuntu headless

The `.github/workflows/e2e.yml` MUST run E2E tests on `ubuntu-latest` inside `xvfb-run`, triggered on push to `main` and on pull requests. The workflow MUST install all required system dependencies: `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `libsoup-3.0-dev`, `libjavascriptcoregtk-4.1-dev`, and `webkit2gtk-driver`.

(Previously: the workflow was missing 5 system deps — `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `libsoup-3.0-dev`, `libjavascriptcoregtk-4.1-dev` — causing build failures.)

#### Scenario: S3 — CI gate passes

- GIVEN a pull request is opened
- WHEN the `e2e` job runs in GitHub Actions
- THEN the job installs system deps (webkit2gtk-driver, xvfb, libgtk-3-dev, libayatana-appindicator3-dev, librsvg2-dev, libsoup-3.0-dev, libjavascriptcoregtk-4.1-dev)
- AND installs `tauri-driver`
- AND runs `xvfb-run --auto-servernum npm run test:e2e`
- AND the job completes within 15 minutes

#### Scenario: Missing system dep causes build failure

- GIVEN `libgtk-3-dev` is not in the `apt-get install` list
- WHEN `cargo tauri build --debug` runs in the E2E job
- THEN the build fails with a pkg-config error for `gtk+-3.0`
