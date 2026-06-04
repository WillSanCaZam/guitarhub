# Proposal: Plan V3 Revision

## Intent

Address gaps found in adversarial review of the v3 plan: platform holes (macOS), infra blind spots (DB migration runner), CI risks (security scanning, gh-pages push conflicts), and dev experience friction (dev container, Makefile, env config). All changes are zero-cost infra additions — no service spend, no code deletion.

## Scope

### In Scope
- **Critical**: macOS CI matrix entry, DB migration runner
- **CI/Security**: `cargo audit`, `pip-audit`, Dependabot config, `concurrency` on gh-pages workflows
- **Dev Experience**: dev container, `.env.example`, `Makefile`, pre-commit hooks, deduplicate `CONTRIBUTING.md`
- **Packaging**: F-Droid reproducible build doc, AppStream `metainfo.xml`, `.desktop` file + icon
- **Data/Logging**: delta sync fallback heuristic, Reverb pagination full-page warning, structured logging framework, rename `missing_price_pct` → `missing_price_ratio`
- **Offline**: local image cache strategy

### Out of Scope
UI/UX critique, code correctness, perf beyond delta fallback, API legal/ToS, iOS (Phase 5), monetization ethics

## Capabilities

### New Capabilities
- `db-migration-runner`: schema migration discovery, tracking, and application
- `local-image-cache`: offline-first image caching (strategy doc + SQLite blob cache)

### Modified Capabilities
None — pure infra/ops improvements, no spec-level behavior changes.

## Approach

Fix in priority order, all non-destructive:

1. **Critical blockers** (block all other work): macOS CI matrix, migration runner with `schema_meta` tracking
2. **CI/infra hardening** (parallelizable): security scanning in `scrape.yml`, Dependabot config, add `concurrency` group to gh-pages workflows
3. **Dev experience** (enables contribution): dev container, `.env.example`, `Makefile`, pre-commit config, remove Phase 0 duplicate `CONTRIBUTING.md`
4. **Packaging & docs** (edge polish): F-Droid strategy doc, AppStream XML, `.desktop`+icon in `scripts/packaging/`
5. **Data correctness** (low risk, additive): delta sync fallback check, pagination `is_last_page` warning, structured logging via `tracing` crate, rename metric field

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `.github/workflows/release.yml` | Modified | Add macOS x86_64 + aarch64 matrix |
| `.github/workflows/scrape.yml` | Modified | Add `concurrency` group |
| `.github/dependabot.yml` | New | Weekly dependency scan |
| `scraper/src/db/migrations/` | New | Migration runner + SQL files |
| `scraper/src/domain/` | Modified | `missing_price_pct` → `missing_price_ratio` |
| `Cargo.toml` | Modified | Add `tracing` + `tracing-subscriber` |
| `.devcontainer/` | New | Docker + devcontainer.json |
| `.env.example` | New | Document all scraper env vars |
| `Makefile` | New | Unified dev commands |
| `.pre-commit-config.yaml` | New | ruff, mypy, clippy hooks |
| `scripts/packaging/` | New | F-Droid, AppStream, desktop, icon |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Migration runner breaks existing installs | Low | Tested on fresh DB; idempotent by design (`schema_meta` version check) |
| macOS CI runner cost | Low | GitHub-hosted macOS included in plan minutes |

## Rollback Plan

Every item is additive or non-destructive. Rollback per item: revert CI matrix entries, delete new files/dirs, or revert the field rename. Zero impact on existing users or data.

## Dependencies

- Tauri 2 macOS toolchain (included with `tauri init`)
- `cargo audit` + `pip-audit` CLIs (CI-installed)
- Docker (optional — dev container only)

## Success Criteria

- [ ] macOS targets build and pass `cargo test` in CI
- [ ] Migration runner applies `001_init.sql` on empty DB, skips when `db_version` matches
- [ ] `cargo audit` + `pip-audit` run in CI (fail or report findings, not skipped)
- [ ] Concurrent `scrape.yml` and `release.yml` pushes to `gh-pages` do not conflict
- [ ] `make dev` works on a clean checkout
- [ ] `.env.example` documents every env var referenced in scraper code
- [ ] `missing_price_ratio` used consistently across all scraper modules
