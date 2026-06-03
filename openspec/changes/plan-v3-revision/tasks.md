# Tasks: Plan V3 Revision

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~920 (23 tasks across 5 phases) |
| 400-line budget risk | **High** |
| Chained PRs recommended | Yes |
| Suggested split | 5 autonomous work units → 5 PRs |
| Delivery strategy | ask-always |
| Chain strategy | stacked-to-main |

Decision needed before apply: Yes
Chained PRs recommended: Yes
Chain strategy: stacked-to-main
400-line budget risk: High

### Suggested Work Units

| Unit | Tasks | Goal | PR | Base | Est. Lines |
|------|-------|------|-----|------|-----------|
| 1 | T1–T5 | Foundation: CI hardening + Migration Runner | PR 1 | main | ~200 |
| 2 | T6–T11 | Capability: Local Image Cache | PR 2 | main | ~350 |
| 3 | T12–T16 | Dev Experience: container, Makefile, pre-commit | PR 3 | main | ~140 |
| 4 | T17–T20 | Data/Logging: tracing, delta fallback, rename | PR 4 | main | ~95 |
| 5 | T21–T23 | Packaging: F-Droid, AppStream, desktop file | PR 5 | main | ~120 |

Each unit is autonomous. Merge order 1→2 matters (migration runner enables image cache).
Units 3–5 are independent and can merge in any order after 1.

---

## Phase 1: Foundation — Critical + CI/Security

- [x] **T1** — Add macOS CI matrix to release.yml — **Area**: ci — **Effort**: S — **Depends**: — — **Files**: `.github/workflows/release.yml` — **AC**: macOS x86_64 + aarch64 targets build and pass `cargo test` in CI
- [x] **T2** — Add `concurrency` group to scrape.yml publish job — **Area**: ci — **Effort**: S — **Depends**: — — **Files**: `.github/workflows/scrape.yml` — **AC**: concurrent gh-pages pushes queue instead of conflicting
- [x] **T3** — Wire `cargo audit` + `pip-audit` into CI workflows — **Area**: ci — **Effort**: S — **Depends**: — — **Files**: `.github/workflows/ci.yml`, `.github/workflows/scrape.yml` — **AC**: audit commands run on PR and scrape, report findings instead of skipping
- [x] **T4** — Create `.github/dependabot.yml` with weekly schedule — **Area**: ci — **Effort**: S — **Depends**: — — **Files**: `.github/dependabot.yml` — **AC**: Dependabot opens weekly PRs for pip + cargo deps
- [x] **T5** — Implement `MigrationRunner` in `migrations/mod.rs` — **Area**: app-backend — **Effort**: M — **Depends**: — — **Files**: `src-tauri/src/repository/sqlite/migrations/mod.rs`, `sqlite/mod.rs`, `lib.rs` — **AC**: fresh DB applies all `.sql`; up-to-date DB is no-op; gap in sequence returns error; corrupt `db_version` errors; each migration in its own transaction

## Phase 2: Core Capability — Local Image Cache

- [x] **T6** — Create `003_add_image_cache.sql` migration — **Area**: app-backend — **Effort**: S — **Depends**: T5 — **Files**: `src-tauri/src/repository/sqlite/migrations/003_add_image_cache.sql` — **AC**: adds `image_cache` table with `url_hash PK`, `blob`, `size_bytes`, `last_accessed`, `ttl_seconds` + index on `last_accessed`
- [x] **T7** — Create `repository/sqlite/image_cache.rs` (SQL CRUD + eviction queries) — **Area**: app-backend — **Effort**: M — **Depends**: T6 — **Files**: `src-tauri/src/repository/sqlite/image_cache.rs`, `sqlite/mod.rs` — **AC**: insert, fetch, delete, LRU eviction (`ORDER BY last_accessed ASC LIMIT N`) and total-size queries work against `:memory:` DB
- [x] **T8** — Create `services/image_cache.rs` (`ImageCacheService`) — **Area**: app-backend — **Effort**: M — **Depends**: T7 — **Files**: `src-tauri/src/services/image_cache.rs`, `services/mod.rs` — **AC**: `get(url)` caches on miss, hits cache, coalesces concurrent requests via `DashMap`+`oneshot`, enforces LRU eviction and TTL re-fetch with stale fallback
- [x] **T9** — Create `commands/image_command.rs` for Tauri IPC — **Area**: app-backend — **Effort**: S — **Depends**: T8 — **Files**: `src-tauri/src/commands/image_command.rs`, `commands/mod.rs` — **AC**: `invoke("get_product_image", { imageUrl })` returns `data:image/<mime>;base64,...` string
- [x] **T10** — Wire `ImageCacheService` into `AppState` + update `ProductCard.svelte` — **Area**: app-backend, frontend — **Effort**: S — **Depends**: T9 — **Files**: `src-tauri/src/lib.rs`, `src/lib/components/ProductCard.svelte` — **AC**: ProductCard calls IPC instead of direct `<img src>`, shows cached images offline
- [x] **T11** — Write image cache tests (unit + integration) — **Area**: app-backend — **Effort**: M — **Depends**: T8 — **Files**: test modules in `image_cache.rs` and `services/image_cache.rs` — **AC**: cache hit returns bytes; miss fetches+stores; LRU evicts oldest; concurrent coalesce = 1 HTTP call; stale+offline returns stale blob; oversized entry skips cache

## Phase 3: Dev Experience

- [x] **T12** — Create `.devcontainer/devcontainer.json` + Dockerfile — **Area**: docs — **Effort**: S — **Depends**: — — **Files**: `.devcontainer/devcontainer.json`, `.devcontainer/Dockerfile` — **AC**: VS Code "Reopen in Container" installs Rust, Python 3.12, Node, and system deps
- [x] **T13** — Create `.env.example` documenting all scraper env vars — **Area**: docs — **Effort**: S — **Depends**: — — **Files**: `.env.example` — **AC**: every env var referenced in `scraper/` code has a documented entry with description
- [x] **T14** — Create `Makefile` with dev/build/test/clean targets — **Area**: docs — **Effort**: S — **Depends**: — — **Files**: `Makefile` — **AC**: `make dev`, `make build`, `make test`, `make clean` all succeed on clean checkout
- [x] **T15** — Create `.pre-commit-config.yaml` with ruff, mypy, clippy hooks — **Area**: docs — **Effort**: S — **Depends**: — — **Files**: `.pre-commit-config.yaml` — **AC**: `pre-commit run --all-files` passes lint and type checks
- [x] **T16** — Deduplicate `CONTRIBUTING.md` — **Area**: docs — **Effort**: S — **Depends**: — — **Files**: `docs/CONTRIBUTING.md` — **AC**: single `CONTRIBUTING.md` living in `docs/` (Phase 0 owns it, remove any Phase 2 duplicate)

## Phase 4: Data/Logging

- [x] **T17** — Add `tracing` + `tracing-subscriber` to Cargo.toml, init in lib.rs — **Area**: app-backend — **Effort**: S — **Depends**: — — **Files**: `src-tauri/src/lib.rs` — **AC**: structured JSON logs emitted at startup
- [ ] **T18** — Add delta sync fallback heuristic in `sync_service.rs` — **Depends**: sync_service.rs (Phase 1 implementation) — **Deferred**: implement when SyncService is built
- [ ] **T19** — Add Reverb pagination `is_last_page` warning — **Depends**: scraper/ directory (Phase 0 implementation) — **Deferred**: implement when ReverbAdapter is built
- [ ] **T20** — Rename `missing_price_pct` → `missing_price_ratio` — **Depends**: scraper/ directory (Phase 0 implementation) — **Deferred**: implement when scraper domain models are built

## Phase 5: Packaging

- [x] **T21** — Create F-Droid reproducible build strategy doc — **Area**: docs, packaging — **Effort**: S — **Depends**: — — **Files**: `scripts/packaging/fdroid-reproducible-build.md` — **AC**: documents build environment, dependency pinning, and reproducibility verification steps for F-Droid submission
- [x] **T22** — Create AppStream `metainfo.xml` — **Area**: packaging — **Effort**: S — **Depends**: — — **Files**: `scripts/packaging/com.guitarhub.metainfo.xml` — **AC**: passes `appstreamcli validate` with no errors; includes screenshots URL, OARS content ratings, donation URL
- [x] **T23** — Create `.desktop` file + placeholder app icon — **Area**: packaging — **Effort**: S — **Depends**: — — **Files**: `scripts/packaging/com.guitarhub.app.desktop`, `scripts/packaging/icons/com.guitarhub.app.svg` — **AC**: `desktop-file-validate` passes; icon is a valid SVG with guitar silhouette
