# Design: Post-Release Hardening

## Technical Approach

Three independent subsystems hardening the release path: (1) CI/CD pipeline guardrails in `release.yml`, (2) in-app updater via `tauri-plugin-updater`, (3) one Dependabot merge for `httpmock`. Each is self-contained — failure in one does not block the others.

## Architecture Decisions

### Decision: Tag-scoped vs. global concurrency

| Option | Tradeoff | Decision |
|--------|----------|----------|
| Global group name (`gh-pages-publish`) | Serializes ALL releases, even to different branches; was blocking parallel work | ❌ Rejected |
| Tag-scoped group (`${{ github.ref_name }}`) | Each tag run is independent; retags of same version queue | ✅ **Chosen** |
| No concurrency | gh-pages race between concurrent releases; lost latest.json updates | ❌ Rejected |

**Rationale**: Tag names are unique per release intent. `group: ${{ github.ref_name }}` means v0.1.0 and v0.1.1 build in parallel. Re-pushing the same tag queues instead of colliding.

### Decision: `cancel-in-progress: false` for same-tag retries

**Rationale**: When re-pushing the same tag (e.g., to fix a failed build), the in-progress run should complete rather than be killed mid-step. Its artifacts may be useful for debugging, and canceling a running `tauri build` can leave dangling processes.

### Decision: Linux-only build matrix

| Option | Tradeoff | Decision |
|--------|----------|----------|
| 4-platform matrix (current) | Blocks release when macOS/Windows runners are unavailable; wasteful on scarce macOS capacity | ❌ Rejected |
| Linux-only + documented intent | Releases ship immediately; macOS/Windows deferred to cross-compilation or verified runners | ✅ **Chosen** |
| Full matrix with `continue-on-error` | Produces partial releases with confusing "failed but passed" signal | ❌ Rejected |

**Rationale**: The spec requires `fail-fast: false` and a single `x86_64-unknown-linux-gnu` entry. No macOS/Windows codesigning certs exist yet — producing unsigned bundles is misleading.

### Decision: Tauri updater signature handling

**Choice**: Empty `signature` string in `latest.json` for now; document that signing is deferred.
**Rationale**: Tauri updater accepts empty signatures with `TAURI_SKIP_SIGNING=true` (already set in CI). Generating a real keypair and signing bundles requires a secure secret management review — out of scope. The updater still works for notification; download will fail until URLs are populated.

### Decision: Only `httpmock` upgrade in scope

**Choice**: Merge `httpmock 0.7 → 0.8.3` only. Skip the other 3 Dependabot PRs.
**Rationale**: `httpmock` is test-only (`[dev-dependencies]`), lowest risk. Other bumps (`reqwest`, `serde`, `tauri`) affect production behavior and need separate validation.

## Data Flow

### Release Pipeline

```
Tag push (v*) ──→ concurrency check (group: ref_name)
                      │
                      ↓
                 Build job (Linux x86_64)
                      │
                      ↓
                 create-release (gh release create)
                      │
                      ↓
                 publish-update-endpoint
                 (generate_latest.json, push to gh-pages)
```

### In-App Updater

```
App startup ──→ tauri-plugin-updater init
                    │
                    ↓
               GET https://willsancazam.github.io/guitarhub/latest.json
                    │
                    ↓
               Compare version vs installed
                    │
                    ├── newer → show "Update available" dialog
                    └── same  → silent no-op
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `.github/workflows/release.yml` | Modify | Add concurrency group, reduce matrix to Linux |
| `src-tauri/Cargo.toml` | Modify | Add `tauri-plugin-updater = "2"` dep |
| `src-tauri/src/lib.rs` | Modify | Register `.plugin(tauri_plugin_updater::init())` |
| `src-tauri/tauri.conf.json` | Modify | Add `app.updater` block, add `bundle` section |
| `src-tauri/capabilities/default.json` | Modify | Add `"updater:default"` to permissions |
| `scripts/generate_latest_json.py` | Modify | No changes needed — already produces correct structure |
| `src-tauri/Cargo.toml` | Modify | Change `httpmock = "0.7"` → `"0.8.3"` in dev-dependencies |

## Interfaces / Contracts

### latest.json format (unchanged, already correct)

```json
{
  "version": "0.1.1",
  "notes": "",
  "pub_date": "2026-06-06T12:00:00Z",
  "platforms": {
    "linux-x86_64": { "signature": "", "url": "" },
    "windows-x86_64": { "signature": "", "url": "" },
    "darwin-x86_64": { "signature": "", "url": "" },
    "darwin-aarch64": { "signature": "", "url": "" }
  }
}
```

`url` and `signature` remain empty — populated by a future change when signing is set up. The Tauri updater treats missing URLs as "no update available."

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit | App starts with updater plugin registered | `cargo test` — no panic on builder init |
| Integration | httpmock 0.8.3 compatibility | `cargo test` — all existing mock tests pass |
| CI | Release workflow YAML validity | GitHub validates on push; `act` dry-run if available |
| Manual | Concurrency group behavior | Push two tags in quick succession, verify both run |

## Migration / Rollout

No data migration. The updater plugin is additive — no existing IPC overridden. Rollback: remove `tauri-plugin-updater` dep and plugin line, revert `httpmock` line.

## Failure Modes

| Scenario | Behavior |
|----------|----------|
| Linux build fails | No release created — correct; releases are Linux-only |
| Updater plugin panics at startup | Crashes app on launch — **test before tagging** |
| `latest.json` unreachable | Silent fail in updater (HTTP error logged, no crash) |
| `httpmock` upgrade breaks tests | Revert single Cargo.toml line and `Cargo.lock` |
| gh-pages push conflict | Retry loop (3 attempts with rebase) already in workflow |

## Open Questions

- [ ] Where is the Tauri updater public key stored / generated? Deferred to post-hardening.
