# Proposal: Fix cargo audit vulnerabilities

## Intent

Two actionable `cargo audit` vulnerabilities in the Rust backend: `rsa` 0.9.10 (RUSTSEC-2023-0071, transitive via sqlx default `any` feature — no fix version) and `sqlx` 0.7.4 (RUSTSEC-2024-0363, fixed in 0.8.1+). The app uses SQLite only; `rsa` is dead weight. Seven GTK3 advisories are informational only (Tauri requires GTK3). Fix both actionable vulnerabilities by upgrading sqlx to 0.8 and dropping unused features.

## Scope

### In Scope
- **Bump sqlx 0.7 → 0.8** with `default-features = false` + explicit `sqlite`/`runtime-tokio`/`derive` features (drops `rsa`)
- **Create `.cargo/audit.toml`** to ignore 7 known-safe GTK3 advisories
- **Fix any sqlx 0.8 API breaks** — app uses runtime-style API (query, query_as, query_scalar, FromRow), not compile-time macros
- **Run `cargo update -p sqlx`** to resolve new dependency tree
- **Verify with `cargo audit`** both actionable vulnerabilities are gone

### Out of Scope
- GTK3 advisories (atk, gdk, gtk, pango, etc.) — informational, no fix available, Tauri dependency
- MSRV changes — current Rust 1.95.0, no concern
- Any other dependency upgrades beyond sqlx

## Capabilities

No spec-level behavior changes. Pure dependency management & security fix — no new or modified capabilities.

### New Capabilities
None

### Modified Capabilities
None

## Approach

1. Edit `src-tauri/Cargo.toml`: sqlx `{ version = "0.8", default-features = false, features = ["runtime-tokio", "sqlite", "derive"] }`
2. Create `.cargo/audit.toml` with `[advisories] ignore` listing 7 GTK3 advisory IDs
3. Attempt `cargo build` — fix any sqlx 0.8 API changes (expected: minor FromRow impl changes, query_as return types)
4. `cargo update -p sqlx` to settle dependency tree
5. `cargo audit` — confirm zero actionable warnings

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src-tauri/Cargo.toml` | Modified | sqlx version + features |
| `.cargo/audit.toml` | New | Advisory ignore list |
| `src-tauri/src/` | Modified | Any sqlx 0.8 API break fixes |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| sqlx 0.8 API break in query macros | Low | App uses runtime API, not compile-time; minor fixes expected |
| Transitive dep conflict | Low | `cargo update` resolves; no pinned deps conflict |
| `.cargo/audit.toml` path mismatch | Low | Verify with `cargo audit --config .cargo/audit.toml` |

## Rollback Plan

Revert `src-tauri/Cargo.toml` to previous sqlx line, delete `.cargo/audit.toml`, revert any code changes, run `cargo update` to restore lockfile. Covers all 3 changed areas independently.

## Dependencies

- None (self-contained Rust dependency update)

## Success Criteria

- [ ] `cargo audit` exits 0 with zero actionable vulnerabilities
- [ ] `cargo build` and `cargo test` pass
- [ ] RUSTSEC-2023-0071 and RUSTSEC-2024-0363 no longer appear in audit output
- [ ] GTK3 advisories present but marked as ignored in audit output
