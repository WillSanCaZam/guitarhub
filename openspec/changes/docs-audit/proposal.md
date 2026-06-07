# Proposal: Docs Audit

## Intent

GuitarHub v0.1.0 has stale docs — placeholder values (`{owner}`, `user/guitarhub`), outdated CONTRIBUTING content (scraper marked "not yet implemented" when it is), and missing architecture/release docs create contributor friction.

## Scope

### In Scope

1. **SECURITY.md** — Replace `{owner}` placeholder with actual GitHub owner
2. **CONTRIBUTING.md** — Remove stale "scraper not implemented" banner; fix CI matrix example (only `reverb` is live); fix discussion link (`user` → `WillSanCaZam`)
3. **.env.example** — Replace `user/guitarhub` with `WillSanCaZam/guitarhub`
4. **docs/ARCHITECTURE.md** — New doc: module tree (Rust backend, Svelte frontend, Python scraper), IPC flow (commands → services → repository → SQLite), data flow (scrape → catalog → sync → local DB), key design decisions
5. **docs/RELEASE.md** — New doc: versioning, tag-and-release workflow, CI pipeline walkthrough, verification
6. **CHANGELOG** — Update `[Unreleased]` with post-v0.1.0 changes (CI fixes, in-app updater, URL corrections)

### Out of Scope

- docs/TESTING.md, docs/API.md, docs/DATABASE.md, ROADMAP.md
- scripts/packaging/fdroid-reproducible-build.md (needs its own change)
- scripts/generate_latest_json.py documentation

## Capabilities

### New Capabilities

None — documentation-only change, no new spec-level behavior.

### Modified Capabilities

None — no existing specs change requirements.

## Approach

Audit each file against the actual project state, replace all placeholder references, and create the two missing docs by synthesizing existing code structure and CI config. CHANGELOG entries are extracted from the git log since the v0.1.0 tag.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `SECURITY.md` | Modified | Replace `{owner}` placeholder |
| `docs/CONTRIBUTING.md` | Modified | Update scraper status, CI matrix, discussion link |
| `.env.example` | Modified | Fix `user/guitarhub` references |
| `docs/ARCHITECTURE.md` | New | Module tree, IPC flow, data flow, design decisions |
| `docs/RELEASE.md` | New | Versioning, CI walkthrough, verification |
| `CHANGELOG.md` | Modified | Add [Unreleased] entries from git log |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| ARCHITECTURE.md drifts from code | Med | Review against code at proposal time; mark with last-reviewed date |
| RELEASE.md skips edge case in CI | Low | Walk through release.yml manually before finalizing |
| Placeholder missed in large files | Low | Grep for `{owner}`, `user/`, `TODO` across all docs |

## Rollback Plan

Each file change is independent. Revert individual files with `git checkout HEAD -- <file>`. No migration, no schema, no runtime impact.

## Dependencies

None.

## Success Criteria

- [ ] No `{owner}` or `user/` placeholders remain in any tracked doc
- [ ] CONTRIBUTING.md scraper banner removed, CI matrix matches `.github/workflows/scrape.yml`, discussion link resolves
- [ ] ARCHITECTURE.md describes module tree, IPC flow, and data flow
- [ ] RELEASE.md documents end-to-end release process
- [ ] CHANGELOG [Unreleased] lists all post-v0.1.0 changes
