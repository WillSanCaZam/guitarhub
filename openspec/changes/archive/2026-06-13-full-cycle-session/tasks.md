# Tasks: Full-Cycle Autonomous Session

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 2000-3000+ (Phase A alone is ~50 files) |
| 400-line budget risk | High |
| Chained PRs recommended | No — single branch, sequential commits on master |
| Suggested split | 5 atomic commits (one per phase) |
| Delivery strategy | exception-ok |
| Chain strategy | size-exception |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: size-exception
400-line budget risk: High

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | Commit Community Hub files | Commit 1 on master | ~50 files, all additive, revertable as unit |
| 2 | Migrate 3 stores to Svelte 5 | Commit 2 on master | Small diff, high value |
| 3 | Sync docs + cleanup + quick wins | Commits 3-5 on master | Small diffs, safe |

## Phase A: Community Hub Commit

- [ ] A1. `git add` all untracked community files: `src/lib/components/{auth,community,layout,ui}/`, `src/lib/stores/{auth,community,profile,collection,filter}.svelte.ts`, `src/lib/types/community.ts`, `src/lib/styles/{tokens,page,typography}.css`, `src/routes/{explore,feed,lessons,my-gear,profile,saved-riffs}/`, `src-tauri/src/commands/{auth,community,profile}_command.rs`, `src-tauri/src/services/{auth,community,profile}_service.rs`, `src-tauri/src/repository/sqlite/migrations/010_community_schema{,.down}.sql`, `src-tauri/capabilities/community.json`, `docs/design/`, test files. **Estimate**: 3 min. **Verify**: `git diff --cached --stat` shows all expected files.
- [ ] A2. `git add` all modified files: `src-tauri/src/commands/mod.rs`, `src-tauri/src/services/mod.rs`, `src-tauri/src/repository/sqlite/migrations/mod.rs`, `src-tauri/src/main.rs`, `src-tauri/gen/schemas/capabilities.json`, `src/routes/+layout.svelte`, `src/routes/+page.svelte`, `src/lib/components/{CollectionView,FilterBar,ProductCard,SearchPanel,Settings}.svelte`, test files. **Depends on**: A1. **Estimate**: 2 min. **Verify**: `git diff --cached --stat` shows modified files.
- [ ] A3. Verify no secrets/tokens staged: `git diff --cached --name-only | grep -iE 'secret|token|key|password'` returns nothing. **Depends on**: A2. **Estimate**: 1 min. **Verify**: grep returns empty.
- [ ] A4. `make lint` — clippy, ruff, mypy, svelte-check. **Depends on**: A3. **Estimate**: 10 min. **Verify**: zero errors.
- [ ] A5. `make test` — 171 frontend + 373 Rust + 49 Python. **Depends on**: A4. **Estimate**: 15 min. **Verify**: all pass.
- [ ] A6. Commit: `feat(community): add Community Hub — auth, profiles, lessons, feed, navigation shell`. **Depends on**: A5. **Estimate**: 1 min. **Verify**: `git log -1 --oneline` shows correct message.

## Phase B: Svelte 5 Store Migration

- [ ] B1. Migrate `src/lib/stores/dashboard.ts` → `src/lib/stores/dashboard.svelte.ts`: replace `writable()` with `$state()`, rename export `dashboardStats` → `dashboardState`, update consumers in `src/routes/+page.svelte` and `src/routes/__tests__/page.test.ts`. **Depends on**: A6. **Estimate**: 5 min. **Verify**: `npm run check` passes.
- [ ] B2. Migrate `src/lib/stores/sync.ts` → `src/lib/stores/sync.svelte.ts`: replace `writable()` with `$state()`, rename export `syncResult` → `syncState`, update consumers in `src/routes/+page.svelte`, `src/routes/+layout.svelte`, `src/routes/__tests__/page.test.ts`. **Depends on**: B1. **Estimate**: 5 min. **Verify**: `npm run check` passes.
- [ ] B3. Migrate `src/lib/stores/wishlist.ts` → `src/lib/stores/wishlist.svelte.ts`: replace `writable()` with `$state()`, rename export `wishlistStore` → `wishlistState`, update consumers in `src/routes/+layout.svelte`, `src/routes/wishlist/+page.svelte`. **Depends on**: B2. **Estimate**: 5 min. **Verify**: `npm run check` passes.
- [ ] B4. Delete old `.ts` files: `git rm src/lib/stores/{dashboard,sync,wishlist}.ts`. **Depends on**: B3. **Estimate**: 1 min. **Verify**: `ls src/lib/stores/*.ts` returns nothing (only `.svelte.ts`).
- [ ] B5. `make lint && make test` — full verification. **Depends on**: B4. **Estimate**: 15 min. **Verify**: zero writable() imports in `src/lib/stores/`: `grep -r "writable" src/lib/stores/ --include="*.ts" --include="*.svelte.ts"` returns only `.svelte.ts` files that don't import writable.
- [ ] B6. Commit: `refactor(svelte): migrate remaining stores to Svelte 5 runes`. **Depends on**: B5. **Estimate**: 1 min. **Verify**: `git log -1 --oneline`.

## Phase C: Docs Sync

- [ ] C1. Update `README.md`: version badge `v0.1.0` → `v0.4.0`, add Community Hub, Collection, Export to features section. **Depends on**: B6. **Estimate**: 5 min. **Verify**: `grep "v0.4.0" README.md` shows match.
- [ ] C2. Update `CHANGELOG.md`: add `[0.4.0]` section with Community Hub features. **Depends on**: C1. **Estimate**: 3 min. **Verify**: `grep "0.4.0" CHANGELOG.md` shows match.
- [ ] C3. Verify `AGENTS.md` is current: check skills index, roles, file references. **Depends on**: C2. **Estimate**: 3 min. **Verify**: visual review.
- [ ] C4. Commit: `docs: sync documentation for v0.4.0`. **Depends on**: C3. **Estimate**: 1 min. **Verify**: `git log -1 --oneline`.

## Phase D: Branch Cleanup

- [ ] D1. Verify on master: `git branch --show-current`. **Depends on**: C4. **Estimate**: 1 min. **Verify**: output is `master`.
- [ ] D2. List merged branches: `git branch --merged master | grep -v '^\*' | grep -v 'master'`. **Depends on**: D1. **Estimate**: 1 min. **Verify**: shows expected branches.
- [ ] D3. Delete merged branches: `git branch --merged master | grep -v '^\*' | grep -v 'master' | xargs git branch -d`. Expected: `feature/sprint3-optimization`, `feature/sprint2-pr1-searchpanel`, `feature/sprint2-pr2-cells`, `feature/sprint2-pr3-css-cleanup`, `feature/sprint3-pr1-batch-upserts`, `feature/sprint3-pr2-virtual-scroll`, `feature/sprint4-pr1-integration-tests` (7 branches). **Depends on**: D2. **Estimate**: 1 min. **Verify**: `git branch` shows only `master`.
- [ ] D4. Commit: `chore: clean up stale feature branches`. **Depends on**: D3. **Estimate**: 1 min. **Verify**: `git log -1 --oneline`.

## Phase E: Quick Wins

- [ ] E1. Remove `beautifulsoup4>=4.12` from `scraper/requirements.txt`. **Depends on**: D4. **Estimate**: 1 min. **Verify**: `grep "beautifulsoup" scraper/requirements.txt` returns nothing.
- [ ] E2. Bump `package.json` version from `0.3.0` → `0.4.0`. **Depends on**: E1. **Estimate**: 1 min. **Verify**: `grep '"version"' package.json` shows `0.4.0`.
- [ ] E3. `make lint && make test` — final verification. **Depends on**: E2. **Estimate**: 15 min. **Verify**: all pass.
- [ ] E4. Commit: `chore: remove unused deps and fix version mismatch`. **Depends on**: E3. **Estimate**: 1 min. **Verify**: `git log -1 --oneline`.
