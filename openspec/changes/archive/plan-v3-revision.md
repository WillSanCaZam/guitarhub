# Archive Report: plan-v3-revision

**Archived**: 2026-06-03
**Archive Path**: `openspec/changes/archive/2026-06-03-plan-v3-revision/`

## Change Title and Intent

**Title**: Plan V3 Revision

**Intent**: Address gaps found in adversarial review of the v3 plan — platform holes (macOS), infra blind spots (DB migration runner), CI risks (security scanning, gh-pages push conflicts), and dev experience friction (dev container, Makefile, env config). All changes are zero-cost infra additions with no service spend or code deletion.

## Summary of What Was Implemented

**20 of 23 tasks completed** across 5 work units:

### Phase 1: Foundation — Critical + CI/Security (T1–T5) ✅
- **T1** — Add macOS CI matrix to release.yml (x86_64 + aarch64 targets)
- **T2** — Add `concurrency` group to scrape.yml publish job
- **T3** — Wire `cargo audit` + `pip-audit` into CI workflows
- **T4** — Create `.github/dependabot.yml` with weekly schedule
- **T5** — Implement `MigrationRunner` in `migrations/mod.rs`

### Phase 2: Core Capability — Local Image Cache (T6–T11) ✅
- **T6** — Create `003_add_image_cache.sql` migration
- **T7** — Create `repository/sqlite/image_cache.rs` (SQL CRUD + eviction)
- **T8** — Create `services/image_cache.rs` (`ImageCacheService`)
- **T9** — Create `commands/image_command.rs` for Tauri IPC
- **T10** — Wire `ImageCacheService` into `AppState` + update `ProductCard.svelte`
- **T11** — Write image cache tests (unit + integration)

### Phase 3: Dev Experience (T12–T16) ✅
- **T12** — Create `.devcontainer/devcontainer.json` + Dockerfile
- **T13** — Create `.env.example` documenting all scraper env vars
- **T14** — Create `Makefile` with dev/build/test/clean targets
- **T15** — Create `.pre-commit-config.yaml` with ruff, mypy, clippy hooks
- **T16** — Deduplicate `CONTRIBUTING.md`

### Phase 4: Data/Logging (T17–T20) ⚠️ Partial
- **T17** ✅ — Add `tracing` + `tracing-subscriber` to Cargo.toml, init in lib.rs
- **T18–T20** ❌ Deferred (see below)

### Phase 5: Packaging (T21–T23) ✅
- **T21** — Create F-Droid reproducible build strategy doc
- **T22** — Create AppStream `metainfo.xml`
- **T23** — Create `.desktop` file + placeholder app icon

## Delta Specs Merged into Main Specs

Both delta specs are new capabilities (no prior main spec existed), so they were **copied directly** to the main specs directory:

| Domain | Action | Details |
|--------|--------|---------|
| `db-migration-runner` | **Created** | `openspec/specs/db-migration-runner/spec.md` — 6 requirements with scenarios |
| `local-image-cache` | **Created** | `openspec/specs/local-image-cache/spec.md` — 6 requirements with scenarios |

## What Was Deferred

| Task | Description | Reason |
|------|-------------|--------|
| **T18** | Delta sync fallback heuristic in `sync_service.rs` | Depends on `sync_service.rs` (Phase 1 implementation — not yet built at time of planning) |
| **T19** | Reverb pagination `is_last_page` warning | Depends on `scraper/` directory (ReverbAdapter — Phase 0 implementation) |
| **T20** | Rename `missing_price_pct` → `missing_price_ratio` | Depends on scraper domain models (Phase 0 implementation) |

All three are additive changes that can be implemented independently when their dependency code exists.

## Files Created / Modified Summary

### CI & GitHub
| File | Action |
|------|--------|
| `.github/workflows/release.yml` | Modified — added macOS x86_64 + aarch64 matrix |
| `.github/workflows/scrape.yml` | Modified — added `concurrency` group |
| `.github/workflows/ci.yml` | Modified — wired `cargo audit` + `pip-audit` |
| `.github/dependabot.yml` | Created — weekly dependency scan |

### App Backend (Rust)
| File | Action |
|------|--------|
| `src-tauri/src/repository/sqlite/migrations/mod.rs` | Created — `MigrationRunner` struct + impl |
| `src-tauri/src/repository/sqlite/migrations/003_add_image_cache.sql` | Created — image cache migration |
| `src-tauri/src/repository/sqlite/image_cache.rs` | Created — SQL CRUD + eviction queries |
| `src-tauri/src/repository/sqlite/mod.rs` | Modified — wire in migration runner + image cache modules |
| `src-tauri/src/services/image_cache.rs` | Created — `ImageCacheService` |
| `src-tauri/src/services/mod.rs` | Modified — register image cache service |
| `src-tauri/src/commands/image_command.rs` | Created — Tauri IPC command |
| `src-tauri/src/commands/mod.rs` | Modified — register image command |
| `src-tauri/src/lib.rs` | Modified — `tracing` init, `MigrationRunner` init, `ImageCacheService` wired into `AppState` |
| `Cargo.toml` | Modified — added `tracing` + `tracing-subscriber` |

### Frontend (Svelte)
| File | Action |
|------|--------|
| `src/lib/components/ProductCard.svelte` | Modified — calls IPC instead of direct `<img src>` |

### Dev Experience
| File | Action |
|------|--------|
| `.devcontainer/devcontainer.json` | Created |
| `.devcontainer/Dockerfile` | Created |
| `.env.example` | Created |
| `Makefile` | Created |
| `.pre-commit-config.yaml` | Created |
| `docs/CONTRIBUTING.md` | Modified — deduplicated |

### Packaging
| File | Action |
|------|--------|
| `scripts/packaging/fdroid-reproducible-build.md` | Created |
| `scripts/packaging/com.guitarhub.metainfo.xml` | Created |
| `scripts/packaging/com.guitarhub.app.desktop` | Created |
| `scripts/packaging/icons/com.guitarhub.app.svg` | Created |

## Verification Notes

No formal verify-report was generated for this change. The tasks were marked complete in the task list (20/23 [x]), with 3 tasks explicitly deferred to future phases. The updated project plan (`guitarhub-plan-v4.md`) incorporates all design decisions and architecture changes from this revision.

## Archive Contents

```
openspec/changes/archive/2026-06-03-plan-v3-revision/
├── proposal.md         ✅
├── specs/
│   ├── db-migration-runner/spec.md    ✅
│   └── local-image-cache/spec.md      ✅
├── design/
│   ├── db-migration-runner.md         ✅
│   └── local-image-cache.md           ✅
└── tasks.md           ✅ (20/23 complete, 3 deferred)
```

## Source of Truth Updated

The following main specs now reflect the new capabilities:
- `openspec/specs/db-migration-runner/spec.md`
- `openspec/specs/local-image-cache/spec.md`

## SDD Cycle Complete

The plan-v3-revision change has been fully planned, designed, partially implemented, and archived. Remaining deferred tasks (T18, T19, T20) are scoped for future changes when their dependencies are available.
