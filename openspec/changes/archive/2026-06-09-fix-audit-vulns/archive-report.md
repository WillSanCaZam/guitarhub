# Archive Report

**Change**: fix-audit-vulns
**Archived**: 2026-06-09
**Archive location**: `openspec/changes/archive/2026-06-09-fix-audit-vulns/`

## Artifact Inventory

| Artifact | Status |
|----------|--------|
| proposal.md | ✅ |
| specs/dependency-security/spec.md | ✅ |
| design.md | ✅ |
| tasks.md | ✅ (7/7 tasks complete) |
| verify-report.md | ✅ — Verdict: PASS |

## Specs Synced to Source of Truth

| Domain | Action | Details |
|--------|--------|---------|
| dependency-security | Created (new) | 5 requirements, 7 scenarios — delta spec was full spec; copied to `openspec/specs/dependency-security/spec.md` |

## Implementation Summary

- **sqlx 0.7 → 0.8.6** with `default-features = false`, explicit `runtime-tokio`/`sqlite`/`derive` features — eliminated `rsa` (RUSTSEC-2023-0071) and fixed RUSTSEC-2024-0363
- **`.cargo/audit.toml`** at project root ignoring 8 GTK3 advisories (0411, 0412, 0413, 0415, 0416, 0418, 0419, 0420)
- **`src-tauri/.cargo/audit.toml`** — duplicate config so `cargo audit` from `src-tauri/` picks up ignores
- **2 flaky tests fixed** in `price_history.rs` — refactored `get_insight` → `get_insight_at(sku, now)` for deterministic timestamp
- **`cargo audit`** exits 0 with zero actionable vulnerabilities
- **`cargo build`** compiles successfully
- **`make test`** — 381 tests pass (303 Rust + 46 Python + 32 JS)

## Verification

- **Build**: ✅ Passed
- **Tests**: ✅ 381/381 passed
- **Audit**: ✅ Exit 0, zero actionable vulns, GTK3 advisories `[ignored]`
- **Verdict**: PASS — all criteria met

## Deviations from Design

- Design predicted 7 GTK3 advisories (RUSTSEC-2024-0412–0418); actual had 8 different IDs
- Design predicted zero `.rs` changes; 2 pre-existing flaky tests required a refactor (`get_insight_at`)
- Critical issue (audit config not found from `src-tauri/`) was identified and fixed during verification

## SDD Cycle Complete

The change has been fully planned, implemented, verified, and archived.

## Traceability

| System | ID |
|--------|----|
| Engram archive report | obs-5f71cc3e61495231 |
| Engram apply-progress | obs-298 |
