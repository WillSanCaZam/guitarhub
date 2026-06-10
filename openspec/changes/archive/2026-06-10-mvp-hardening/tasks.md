# Tasks: mvp-hardening

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~30-40 |
| 400-line budget risk | Low |
| Chained PRs recommended | No |
| Suggested split | Single PR (3 independent work units) |
| Delivery strategy | single-pr |
| Chain strategy | size-exception |

Decision needed before apply: Yes
Chained PRs recommended: No
Chain strategy: size-exception
400-line budget risk: Low

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | HTTPS defense-in-depth in image cache | Single PR | TDD: RED test then GREEN impl |
| 2 | CI frontend build & type-check | Same PR | npm dep + CI steps |
| 3 | Scrape workflow fix + deploy | Same PR | incoming/ dir + gh-pages deploy |

## Phase 1: HTTPS Defense-in-Depth

- [x] 1.1 RED: Add test asserting `http://` URL returns `ImageCacheError::InvalidUrl` in `image_cache.rs` tests
- [x] 1.2 GREEN: Replace `tracing::warn!` for `http://` URLs with `return Err(ImageCacheError::InvalidUrl(...))` in `ImageCacheService::get()` — remove the `if url.starts_with("http://")` warning path

## Phase 2: CI Frontend Build

- [x] 2.1 Add `"svelte-check": "^4.0.0"` to `devDependencies` in `package.json`
- [x] 2.2 Add `"check": "svelte-check"` to `scripts` in `package.json`
- [x] 2.3 Insert `npm run build` then `npm run check` steps in `.github/workflows/ci.yml` after `npm run test` and before signing

## Phase 3: Scrape Workflow

- [x] 3.1 In `.github/workflows/scrape.yml`, add `mkdir -p incoming/` and `cp catalog-${{ matrix.source }}.json incoming/` before the `validate-input` step
- [x] 3.2 Add `permissions:` block with `contents: write` and deploy step using `peaceiris/actions-gh-pages@v4` with `publish_dir: incoming/` after `validate-input`

## Phase 4: Verify All

- [x] 4.1 Run `cargo test` — all image cache tests pass including new HTTPS rejection test
- [ ] 4.2 Run `npm run check` and `npm run build` pass locally (optional: CI check passes)
