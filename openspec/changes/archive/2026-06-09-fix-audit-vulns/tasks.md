# Tasks: Fix cargo audit vulnerabilities

## Review Workload Forecast

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: pending
400-line budget risk: Low

| Field | Value |
|-------|-------|
| Estimated changed lines | ~10–15 (Cargo.toml + audit.toml; Cargo.lock auto) |
| 400-line budget risk | Low |
| Chained PRs recommended | No |
| Suggested split | Single PR |
| Delivery strategy | single-pr |

## Phase 1: Dependency & Config Changes

- [x] 1.1 Edit `src-tauri/Cargo.toml` — set `sqlx = { version = "0.8", default-features = false, features = ["runtime-tokio", "sqlite", "derive"] }`
- [x] 1.2 Run `cargo audit` in project root, capture the 7 GTK3 advisory IDs from output
- [x] 1.3 Create `.cargo/audit.toml` — `[advisories]` section ignoring 8 GTK3 IDs (actual audit found 8: 0411, 0412, 0413, 0415, 0416, 0418, 0419, 0420)

## Phase 2: Build & Verify

- [x] 2.1 `cargo update -p sqlx` to resolve new dependency tree
- [x] 2.2 `cargo build` — verify compilation succeeds (no API breaks — zero code changes needed)
- [x] 2.3 `cargo test` — 303/303 Rust, 46/46 Python, 32/32 JS pass
- [x] 2.4 `cargo audit` — exit 0, RUSTSEC-2023-0071 and RUSTSEC-2024-0363 absent, 8 GTK3 advisories `[ignored]`
