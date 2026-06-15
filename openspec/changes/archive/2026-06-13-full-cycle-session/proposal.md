# Proposal: Full-Cycle Autonomous Session

## Intent

GuitarHub has ~25+ untracked Community Hub files, 3 unmigrated Svelte 4 stores, stale branches, and outdated docs. This session commits and validates all pending work in a single coherent pass, bringing the repo to a clean state ready for v0.4.0.

## Scope

### In Scope
- **Phase A**: Commit the Community Hub (auth, profiles, lessons, riffs, feed, comments, streaks, navigation shell, design tokens)
- **Phase B**: Migrate 3 remaining Svelte 4 stores → Svelte 5 runes (`dashboard.ts`, `sync.ts`, `wishlist.ts`)
- **Phase C**: Sync README (version badge v0.3.0→v0.4.0), CHANGELOG, AGENTS.md
- **Phase D**: Delete 11 stale branches behind master
- **Phase E**: Quick wins — remove dead `beautifulsoup4` dep, fix Python version mismatch

### Out of Scope
- Server backend (only client integration ships)
- E2E test verification (deferred — needs debug binary + tauri-driver)
- i18n foundation, a11y audit
- Second scraper adapter
- Real-time chat/DMs, video hosting, content moderation system

## Capabilities

### New Capabilities
- `community-hub`: Auth (OAuth/JWT), user profiles, practice streaks, lessons, riffs, feed, comments, follows, challenges, leaderboards, adaptive navigation (sidebar/bottom nav), Acoustic Dark Modern design system

### Modified Capabilities
- `repo-presentable`: README version badge update to v0.4.0, CHANGELOG new release section, missing feature documentation (Collection, Community Hub, Export)

## Approach

**Staged commits per phase**, each verified with `make lint && make test` before moving on:

1. **Phase A** (Community Hub): `git add` the ~25 untracked files + modifications → single commit with message `feat(community): add Community Hub — auth, profiles, lessons, feed, navigation shell`. Verify build.
2. **Phase B** (Svelte 5 stores): Migrate `dashboard.ts`, `sync.ts`, `wishlist.ts` to `$state` runes → commit `refactor(svelte): migrate remaining stores to Svelte 5 runes`. Verify tests.
3. **Phase C** (Docs): Update README, CHANGELOG, AGENTS.md → commit `docs: sync documentation for v0.4.0`.
4. **Phase D** (Branches): `git branch -d` for 11 stale branches → commit `chore: clean up stale feature branches`.
5. **Phase E** (Quick wins): Remove dead dep, fix version → commit `chore: remove unused deps and fix version mismatch`.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src/lib/components/community/` | New | ~10 community components |
| `src/lib/components/auth/` | New | Auth UI components |
| `src/lib/components/layout/` | New | Navigation shell (sidebar + bottom nav) |
| `src/lib/components/ui/` | New | Shared UI atoms |
| `src/lib/stores/` | Modified | 5 new stores (auth, community, profile, collection, filter) + 3 migrated |
| `src/routes/` | Modified | New routes: feed, explore, lessons, profile, my-gear, saved-riffs |
| `src-tauri/src/commands/` | Modified | 3 new command files (auth, community, profile) |
| `src-tauri/src/services/` | Modified | 3 new services (auth, community, profile) |
| `src-tauri/src/repository/sqlite/migrations/` | Modified | Migration 010 (community schema) |
| `src-tauri/capabilities/community.json` | New | Tauri permissions for community features |
| `docs/design/` | New | Acoustic Dark Modern design tokens |
| `README.md` | Modified | Version badge, feature docs |
| `CHANGELOG.md` | Modified | v0.4.0 release section |
| `AGENTS.md` | Modified | Skills index, roles |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Community Hub breaks existing tests | Medium | Run `make test` after Phase A; isolate regressions before Phase B |
| Svelte 5 store migration causes runtime bugs | Low | Each store migrated independently; test after each |
| Stale branch deletion loses unmerged work | Low | Verify each branch is fully merged via `git branch --merged master` |
| Large commit hard to review | Medium | Phases produce separate commits; each is reviewable independently |

## Rollback Plan

- **Phase A**: `git revert <community-hub-commit>` — all Community Hub changes are additive
- **Phase B**: `git revert <store-migration-commit>` — stores are self-contained
- **Phase C**: `git revert <docs-commit>` — doc changes only
- **Phase D**: Branches are already merged; deletion is non-destructive (reflog recovery available for 90 days)
- **Phase E**: `git revert <quick-wins-commit>`

## Dependencies

- None — all work uses existing tooling and tested patterns

## Success Criteria

- [ ] `make test` passes after each phase (171 frontend + 373 Rust + 49 Python)
- [ ] `make lint` passes (clippy, ruff, mypy, svelte-check)
- [ ] Community Hub files committed and tracked in git
- [ ] All 8 stores use Svelte 5 `$state` runes (zero `writable()` imports in `src/lib/stores/`)
- [ ] README version badge matches `package.json` version
- [ ] CHANGELOG has v0.4.0 section with Community Hub features
- [ ] 11 stale branches deleted
- [ ] Zero new dependencies introduced
