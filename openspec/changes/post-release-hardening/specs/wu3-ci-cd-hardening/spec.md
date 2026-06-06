# Delta for CI/CD Hardening

## ADDED Requirements

### Requirement: Bundle config MUST declare explicit targets

`tauri.conf.json` MUST include a `bundle` section with `targets: ["deb", "dmg", "msi"]`, `identifier: "com.guitarhub.app"`, and valid `icon` paths.

#### Scenario: Bundle config present

- GIVEN `tauri.conf.json` is inspected
- THEN `bundle.active` is `true`
- AND `bundle.targets` includes `deb`, `dmg`, `msi`
- AND `bundle.icon` references existing files

### Requirement: httpmock dev-dependency upgraded to 0.8.3

`Cargo.toml` MUST upgrade `httpmock` from `"0.7"` to `"0.8.3"`. After upgrade, `cargo test` MUST pass. Other Dependabot PRs are observed but not merged in this change.

#### Scenario: httpmock upgrade validated

- GIVEN `httpmock` is changed to `"0.8.3"` in `[dev-dependencies]`
- WHEN `cargo test` runs
- THEN all tests pass

## MODIFIED Requirements

### Requirement: Concurrency guard for release publishing

`release.yml` MUST define a `concurrency` block with `group: ${{ github.ref_name }}` and `cancel-in-progress: false`.
(Previously: fixed group name `gh-pages-publish`)

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Different tags | Release v0.1.0 runs, v0.1.1 triggered | Both run | Both build in parallel — no cancellation |
| Same tag re-push | v0.1.0 pushed twice | First runs, second queues | First completes, second waits |

### Requirement: Build matrix covers Linux x86_64 only

`release.yml` MUST define a build matrix with 1 entry: `x86_64-unknown-linux-gnu` on `ubuntu-latest`. `fail-fast` MUST be `false`. Timeout MUST be 30 minutes.
(Previously: 4 entries covering Windows, macOS, Linux)

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Linux build succeeds | Tag push triggers workflow | 1 matrix job runs | Linux bundle produced |
| Linux build fails | System dep missing | Job fails | No release created, job reports error |
