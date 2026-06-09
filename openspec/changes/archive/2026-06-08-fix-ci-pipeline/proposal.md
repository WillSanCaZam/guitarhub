# Proposal: Fix CI Pipeline Failures

## Intent

Three CI jobs (Python mypy, frontend signing dry-run, Rust clippy) fail systematically on every PR, blocking velocity and masking real issues. Fix all three so CI is a reliable gating signal.

## Scope

### In Scope
1. Add `[tool.mypy]` config to `scraper/pyproject.toml` — pydantic plugin, per-module relaxed rules for test files
2. Fix Tauri 2 CLI flag: `--private-key` → `--write-keys` in `.github/workflows/ci.yml`
3. Install webkit2gtk-dev + GTK system deps before `cargo clippy` in CI

### Out of Scope
- Fixing non-CI mypy issues or expanding type coverage
- Updating `release.yml` or `scrape.yml`
- Changing test logic or production code
- Dependency upgrades beyond what's needed for CI to pass

## Capabilities

### New Capabilities
None — no new spec-level capabilities introduced.

### Modified Capabilities
None — `wu3-ci-cd-hardening` already specifies the required behavior. This change aligns CI configuration with existing requirements.

## Approach

Three independent, trivially revertible config-only changes:
1. **mypy**: Add `[tool.mypy]` block to `scraper/pyproject.toml` — `plugins = ["pydantic.mypy"]`, per-file ignores for test modules
2. **Signing**: Replace `--private-key` with `--write-keys` in the `tauri signer generate` invocation — single-line fix in CI YAML
3. **Clippy**: Insert `sudo apt-get install -y -qq libwebkit2gtk-4.1-dev libgtk-3-dev libsoup-3.0-dev libjavascriptcoregtk-4.1-dev` step before `cargo clippy`

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `scraper/pyproject.toml` | Modified | Add `[tool.mypy]` with pydantic plugin |
| `.github/workflows/ci.yml` | Modified | Fix signing flag + add apt-get step |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Mypy config misses edge cases | Low | CI validates on next PR; iterate if needed |
| Wrong apt package names for ubuntu-24.04 | Low | Matches Tauri 2 GTK4 deps; CI proves it |
| Signing dry-run depends on Tauri CLI version | Low | Ephemeral test key; no release impact |

## Rollback Plan

Each fix is a single-line or single-block config change. Revert the commit to undo all three. If one fix fails, revert just that hunk.

## Dependencies

- Tauri 2 CLI v2.x (signer subcommand flags stable across minor versions)
- ubuntu-latest runner image (apt package names may vary)

## Success Criteria

- [ ] `mypy scraper/ --strict` passes in CI on PR trigger (0 errors)
- [ ] Frontend signing dry-run exits 0 and produces valid non-empty signature
- [ ] `cargo clippy --all-targets -- -D warnings` passes in CI
- [ ] All three CI jobs (python, frontend, rust) show green checkmarks
