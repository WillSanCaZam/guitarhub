# Proposal: MVP Hardening

## Intent

Close three deferred gaps from the `mvp-fixes` cycle: defense-in-depth at the image cache service layer, missing frontend build + type checking in CI, and a broken scrape workflow validation step with no deploy target.

## Scope

### In Scope
- Reject `http://` URLs at the `ImageCacheService` level with an error (defense-in-depth)
- Add `npm run build` and `svelte-check` to `ci.yml` frontend job; add `svelte-check` dep + `check` script to `package.json`
- Create `incoming/` directory in scrape workflow, wire it to receive catalog output, add GitHub Pages deploy step

### Out of Scope
- Adding svelte-check to the signing dry-run or any non-CI context
- Refactoring the scraper output format or adapter logic
- Adding new scraper sources or changing the scrape schedule

## Capabilities

### New Capabilities
None — all changes modify existing capabilities.

### Modified Capabilities
- `local-image-cache`: Service-layer URL validation changes from warn-only to reject for `http://` URLs. Currently the service accepts `http://` with a `tracing::warn!`; this makes rejection consistent with the command-layer behavior already specified in `wu2-security-hardening`.
- `wu3-ci-cd-hardening`: CI frontend job adds `npm run build` + `svelte-check` steps. Scrape workflow adds `incoming/` directory provisioning and a GitHub Pages deploy step for the catalog JSON artifact.

## Approach

Batch in order of risk (security → developer experience → pipeline):

1. **http:// rejection** — Change `image_cache.rs` lines 114-121: replace the `http://` warn branch with a return of `ImageCacheError::InvalidUrl`. Update or add test.
2. **CI frontend** — Install `svelte-check` as dev dep, add `"check"` script to `package.json`, add `npm run build` and `npm run check` steps to the `frontend` job in `ci.yml`.
3. **Scrape workflow** — Add `mkdir -p incoming/ && cp catalog-${{ matrix.source }}.json incoming/` before the `validate-input` step. Add a `deploy` job that uploads catalog JSON to `gh-pages` via `peaceiris/actions-gh-pages`.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src-tauri/src/services/image_cache.rs` | Modified | `http://` URL rejection at service layer |
| `package.json` | Modified | Add `svelte-check` dep + `check` script |
| `.github/workflows/ci.yml` | Modified | Add frontend build + type-check steps |
| `.github/workflows/scrape.yml` | Modified | Add `incoming/` dir + deploy to GitHub Pages |
| (none) | New | `incoming/` dir created ephemerally in CI runner |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| `http://` rejection breaks local dev that uses HTTP test fixtures | Low | Tests use `httpmock` (ephemeral HTTP servers) which already use URLs parsed fine; verify existing tests pass |
| `svelte-check` adds CI time | Low | Runs in <30s for this project size |
| GitHub Pages deploy requires a `GITHUB_TOKEN` with `contents: write` | Med | Use `permissions:` block in workflow; verify token scope |

## Rollback Plan

Revert all three commits in reverse order (easiest isolated revert per file). The `http://` change is the riskiest — if a user workflow depends on HTTP image URLs, revert that commit first. The CI and workflow changes are additive and safe to revert at any time.

## Dependencies

- `svelte-check` must be compatible with Svelte 5 (already is — we use `svelte-check@4`)

## Success Criteria

- [ ] `ImageCacheService::get("http://...")` returns `Err(ImageCacheError::InvalidUrl)` instead of proceeding
- [ ] `npm run check` runs `svelte-check` and passes
- [ ] CI frontend job includes both `npm run build` and `npm run check` steps
- [ ] Scrape workflow creates `incoming/`, copies catalog JSON there, validates, then deploys to `gh-pages`
- [ ] All existing tests pass (`cargo test`, `pytest`, `npm run test`)
