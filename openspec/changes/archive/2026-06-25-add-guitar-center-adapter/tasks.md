# Tasks: Guitar Center Scraper Adapter

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~900–1,000 |
| 400-line budget risk | High |
| Chained PRs recommended | Yes |
| Suggested split | Adapter + fixture (PR 1) → Tests + integration (PR 2) |
| Delivery strategy | single-pr-default |
| Chain strategy | size-exception — adapter + tests are a tight TDD unit |

Decision needed before apply: Yes
Chained PRs recommended: Yes
Chain strategy: size-exception
400-line budget risk: High

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | `GuitarCenterAdapter` + field mapping + condition normalization + credentials | PR 1 | ~380 lines, standalone — all non-trivial adapter logic |
| 2 | Tests (fixture + unit + protocol) + CLI wiring + env vars + Makefile + docs | PR 2 | Depends on PR 1; CLI/contract tests verify end-to-end |

## Phase 1: Core Adapter

- [x] 1.1 Create `scraper/adapters/guitarcenter.py` with Algolia POST auth, session setup, pagination loop
- [x] 1.2 Implement `_map_hit()`: field mapping per design table (name, brand, sku, price, url, image_url, category)
- [x] 1.3 Implement condition normalization: map GC 9-value vocabulary to `new`/`used`/`refurbished`/`unknown`, preserve original in `specs_json.condition_original`
- [x] 1.4 Implement sticker/badge tracking: `specs_json.stickers` for Open Box, Blemished, Restock
- [x] 1.5 Implement availability logic: `inventoryStatus` in (1000, 1003) AND `stores` non-empty → `in_stock`
- [x] 1.6 Implement credential resolution: constructor args with env var fallback, raise `ValueError` if missing

## Phase 2: Testing

- [x] 2.1 Create `scraper/tests/fixtures/guitarcenter-sample.json` with all 10 condition variants + stickers + multi-store
- [x] 2.2 Write unit tests for every condition normalization case (10 variants across 5 test cases)
- [x] 2.3 Write unit tests for pagination: empty hits, max_pages, rate-limiting delay
- [x] 2.4 Write unit tests for field mapping: complete hit, missing brand, missing image, SKU prefix
- [x] 2.5 Write unit tests for availability: in_stock, out_of_stock, no stores, no inventoryStatus
- [x] 2.6 Write unit tests for error handling: 4xx, timeout, invalid JSON → typed FetchError/ParseError
- [x] 2.7 Write unit test for missing credentials → ValueError
- [x] 2.8 Add `GuitarCenterAdapter` conformance test in `scraper/tests/contract/test_protocol.py`

## Phase 3: Integration

- [x] 3.1 Add `"guitarcenter"` to `--adapter` choices in `scraper/cli.py`, wire import/instantiation
- [x] 3.2 Add `GC_ALGOLIA_APP_ID` and `GC_ALGOLIA_API_KEY` to `.env.example` with Algolia re-extraction docs
- [x] 3.3 Add `scrape-guitarcenter` target to `Makefile`
- [x] 3.4 Update `docs/CONTRIBUTING.md` "Adding a new source adapter" section with current file paths

## Phase 4: Verification

- [x] 4.1 Run `make test-scraper` — all existing + new tests pass (109 tests, 0 failed)
- [x] 4.2 Run `make lint-py` — ruff passes, mypy 7 false positives (same pattern as test_reverb.py)
- [x] 4.3 Verify adapter loads: adapter created and CLI wired per spec; env vars required at runtime
