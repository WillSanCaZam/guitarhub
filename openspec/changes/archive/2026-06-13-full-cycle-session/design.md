# Design: Full-Cycle Autonomous Session

## Technical Approach

Sequential phase execution with verification gates. Each phase produces a single atomic commit, verified with `make lint && make test` before proceeding. No phase depends on another's output — phases are independent work units that share a common "clean repo" goal.

## Architecture Decisions

### Decision: Single commit per phase (not squash)

| Option | Tradeoff | Decision |
|--------|----------|----------|
| One big commit | Simpler, but unreviewable at 1000+ lines | Rejected |
| Atomic commit per phase | Reviewable chunks, clean `git revert` per phase | **Chosen** |
| Commit per file | Excessive noise, hard to correlate related changes | Rejected |

**Rationale**: Each phase is a logical unit. `git revert` works cleanly per-phase. PR review budget (400 lines) is respected per commit.

### Decision: Migrate stores to `$state` runes with direct mutation

| Option | Tradeoff | Decision |
|--------|----------|----------|
| `writable()` → `$state()` with `.update()` | Preserves old API, no real benefit | Rejected |
| `writable()` → `$state()` with direct mutation | Clean Svelte 5 idioms, matches existing `.svelte.ts` stores | **Chosen** |
| Class-based stores | Over-engineered, doesn't match project conventions | Rejected |

**Rationale**: Existing migrated stores (`auth.svelte.ts`, `collection.svelte.ts`, `filter.svelte.ts`) already use `$state` with direct property assignment. Follow the established pattern.

### Decision: Branch deletion via `git branch -d` (not `-D`)

| Option | Tradeoff | Decision |
|--------|----------|----------|
| `git branch -D` (force) | Faster, risk of losing unmerged work | Rejected |
| `git branch -d` (safe) | Refuses if not merged, requires pre-verification | **Chosen** |

**Rationale**: Safety first. `git branch --merged master` pre-check + `-d` = zero risk of data loss.

## Phase A: Community Hub Commit

### Implementation Strategy

1. `git add` all untracked community files + modified files
2. Verify no secrets/tokens staged (`git diff --cached --name-only` against `.gitignore`)
3. Commit with message: `feat(community): add Community Hub — auth, profiles, lessons, feed, navigation shell`
4. Run `make lint && make test`

### File Manifest

**New files (untracked):**
| Path | Description |
|------|-------------|
| `src/lib/components/auth/` | Auth UI components (AuthGuard, LoginModal) |
| `src/lib/components/community/` | Community components (Feed, Comments, Riffs, etc.) |
| `src/lib/components/layout/` | Navigation shell (sidebar + bottom nav) |
| `src/lib/components/ui/` | Shared UI atoms (Button, Card, Avatar) |
| `src/lib/stores/auth.svelte.ts` | Auth state (already Svelte 5) |
| `src/lib/stores/community.svelte.ts` | Community state |
| `src/lib/stores/profile.svelte.ts` | Profile state |
| `src/lib/stores/collection.svelte.ts` | Collection state (already Svelte 5) |
| `src/lib/stores/filter.svelte.ts` | Filter state (already Svelte 5) |
| `src/lib/types/community.ts` | Community TypeScript types |
| `src/lib/styles/tokens.css` | Design tokens |
| `src/lib/styles/page.css` | Page styles |
| `src/lib/styles/typography.css` | Typography styles |
| `src/routes/explore/` | Explore route |
| `src/routes/feed/` | Feed route |
| `src/routes/lessons/` | Lessons route |
| `src/routes/my-gear/` | My Gear route |
| `src/routes/profile/` | Profile route |
| `src/routes/saved-riffs/` | Saved Riffs route |
| `src-tauri/src/commands/auth_command.rs` | Auth Tauri commands |
| `src-tauri/src/commands/community_command.rs` | Community Tauri commands |
| `src-tauri/src/commands/profile_command.rs` | Profile Tauri commands |
| `src-tauri/src/services/auth_service.rs` | Auth service |
| `src-tauri/src/services/community_service.rs` | Community service |
| `src-tauri/src/services/profile_service.rs` | Profile service |
| `src-tauri/src/repository/sqlite/migrations/010_community_schema.sql` | Migration |
| `src-tauri/src/repository/sqlite/migrations/010_community_schema.down.sql` | Down migration |
| `src-tauri/capabilities/community.json` | Tauri permissions |
| `docs/design/` | Design tokens docs |
| `src/lib/components/__tests__/` | Test files |

**Modified files:**
| Path | Change |
|------|--------|
| `src-tauri/src/commands/mod.rs` | Register new command modules |
| `src-tauri/src/services/mod.rs` | Register new service modules |
| `src-tauri/src/repository/sqlite/migrations/mod.rs` | Register migration 010 |
| `src-tauri/src/main.rs` | Add new commands to invoke_handler |
| `src-tauri/gen/schemas/capabilities.json` | Auto-generated capabilities |
| `src/routes/+layout.svelte` | Add nav shell, community imports |
| `src/routes/+page.svelte` | Dashboard updates |
| `src/lib/components/CollectionView.svelte` | Community integration |
| `src/lib/components/FilterBar.svelte` | Filter updates |
| `src/lib/components/ProductCard.svelte` | Card updates |
| `src/lib/components/SearchPanel.svelte` | Search updates |
| `src/lib/components/Settings.svelte` | Community server settings |
| Various `__tests__/` files | Test updates |

### Verification

```bash
make lint && make test
```

## Phase B: Svelte 5 Migration

### Migration Pattern

Three stores (`dashboard.ts`, `sync.ts`, `wishlist.ts`) use `writable()` and must migrate to `$state` runes.

**Before (`dashboard.ts`):**
```ts
import { writable } from 'svelte/store';
export const dashboardStats = writable<DashboardStats>({ ...defaultStats });
// Usage: dashboardStats.update(s => ({ ...s, loading: true }));
// Usage: dashboardStats.set({ ... });
```

**After (`dashboard.svelte.ts`):**
```ts
export const dashboardState: DashboardStats = $state({ ...defaultStats });
// Usage: dashboardState.loading = true; (direct mutation)
```

**Before (`sync.ts`):**
```ts
import { writable } from 'svelte/store';
export const syncResult = writable<SyncResult | null>(null);
```

**After (`sync.svelte.ts`):**
```ts
export const syncState: SyncResult | null = $state(null);
```

**Before (`wishlist.ts`):**
```ts
import { writable } from 'svelte/store';
export const wishlistStore = writable<WishlistStore>({ ...defaultStore });
// Usage: wishlistStore.update(s => ({ ...s, loading: true }));
```

**After (`wishlist.svelte.ts`):**
```ts
export const wishlistState: WishlistStore = $state({ ...defaultStore });
// Usage: wishlistState.loading = true; (direct mutation)
```

### File Manifest

| File | Action |
|------|--------|
| `src/lib/stores/dashboard.ts` → `src/lib/stores/dashboard.svelte.ts` | Rewrite |
| `src/lib/stores/sync.ts` → `src/lib/stores/sync.svelte.ts` | Rewrite |
| `src/lib/stores/wishlist.ts` → `src/lib/stores/wishlist.svelte.ts` | Rewrite |

### Consumer Updates

| Consumer | Import Change |
|----------|--------------|
| `src/routes/+page.svelte` | `dashboardStats` → `dashboardState`, `syncResult` → `syncState` |
| `src/routes/+layout.svelte` | `syncResult` → `syncState`, `wishlistStore` → `wishlistState` |
| `src/routes/wishlist/+page.svelte` | `wishlistStore` → `wishlistState` |
| `src/routes/__tests__/page.test.ts` | Same import renames |

### Verification

```bash
# After each store migration:
npm run check && npm run test
# After all three:
make lint && make test
```

## Phase C: Docs Sync

### File Manifest

| File | Change |
|------|--------|
| `README.md` | Version badge: `v0.1.0` → `v0.4.0`; add Community Hub, Collection, Export features |
| `CHANGELOG.md` | Add `[0.4.0]` section with Community Hub features |
| `AGENTS.md` | Verify skills index, roles, file references are current |

### Verification

```bash
# Visual review — no automated test for docs content
grep -n "v0.4.0" README.md CHANGELOG.md
```

## Phase D: Branch Cleanup

### Safety Checks

1. List merged branches: `git branch --merged master | grep -v '^\*' | grep -v 'master'`
2. Verify each is truly merged (no diff): `git diff master...<branch>` should be empty
3. Confirm on master branch: `git branch --show-current`

### Deletion Sequence

```bash
git checkout master
git branch --merged master | grep -v '^\*' | grep -v 'master' | xargs git branch -d
```

Expected branches (7 based on current state):
- `feature/sprint3-optimization`
- `feature/sprint2-pr1-searchpanel`
- `feature/sprint2-pr2-cells`
- `feature/sprint2-pr3-css-cleanup`
- `feature/sprint3-pr1-batch-upserts`
- `feature/sprint3-pr2-virtual-scroll`
- `feature/sprint4-pr1-integration-tests`

**Note**: Proposal says 11 branches — actual count may differ. Verify with `git branch --merged master` before executing.

### Verification

```bash
git branch  # should show only master
```

## Phase E: Quick Wins

### File Manifest

| File | Change |
|------|--------|
| `scraper/requirements.txt` | Remove `beautifulsoup4>=4.12` (unused — no `bs4` imports found in scraper code) |
| `package.json` | Bump version from `0.3.0` → `0.4.0` |

### Verification

```bash
# Verify beautifulsoup4 not imported anywhere
grep -r "bs4\|beautifulsoup" scraper/  # should return nothing
# Verify version
grep '"version"' package.json  # should show 0.4.0
make lint && make test
```

## Testing Strategy

| Phase | Test Layer | What | Command |
|-------|-----------|------|---------|
| A | Unit + Integration | All existing tests still pass | `make test` |
| A | Lint | clippy, ruff, mypy, svelte-check | `make lint` |
| B | Unit | Each migrated store's consumers | `npm run test` |
| B | Type check | svelte-check passes | `npm run check` |
| C | Manual | Docs render correctly, links work | Visual review |
| D | Git | Only master remains | `git branch` |
| E | Lint + Test | No regressions from dep removal | `make lint && make test` |

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Community Hub breaks tests | Phase A runs `make test` before proceeding. If failures, isolate and fix before Phase B |
| Store migration runtime bugs | Migrate one store at a time, verify after each. Rollback: `git revert` |
| Stale branch has unmerged work | `git branch -d` refuses unmerged branches. Pre-check with `--merged` |
| Large Phase A commit | Acceptable — all files are additive Community Hub features. Revertable as a unit |
| beautifulsoup4 removal breaks scraper | Verified: zero `bs4` imports exist in scraper code |

## Open Questions

- [ ] Confirm exact branch list matches proposal's "11 branches" (current count: 7 merged)
- [ ] Verify `package.json` version bump scope (proposal says v0.4.0 — confirm no Cargo.toml version needs bump)
