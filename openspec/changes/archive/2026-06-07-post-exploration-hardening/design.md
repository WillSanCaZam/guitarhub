# Design: Post-Exploration Hardening

## Technical Approach

Four independent but sequentially-ordered changes: (1) wire real pubkey + signing in CI, (2) make image domain allowlist configurable via settings DB, (3) add AppImage to bundle targets, (4) generate real latest.json. Each builds on the previous — signing feeds latest.json, AppImage adds another signed artifact.

## Architecture Decisions

### Decision: Domain validation — inject SettingsRepository into ImageCacheService

| Option | Tradeoff | Decision |
|--------|----------|----------|
| Validate only in IPC command layer | Cache layer stays "dumb" but bypassable via direct code path | ❌ |
| Pass allowlist to ImageCacheService.get() | Changes public API; no DB coupling | ❌ |
| Inject `Arc<dyn SettingsRepository>` into ImageCacheService | Adds one DB read per cache miss; defense-in-depth | ✅ |

**Rationale**: Spec requires domain validation at cache layer before any HTTP call. Injecting the repo trait follows existing patterns (`SqliteSettingsRepository::new(pool)` is used ad-hoc elsewhere). The DB read is negligible — single indexed key lookup. Defense-in-depth catches anything that bypasses IPC.

### Decision: Signing flow — `tauri signer sign` writes .sig file ingested by script

| Option | Tradeoff | Decision |
|--------|----------|----------|
| Output signature directly to generate_latest_json.py | Tight coupling; script needs to parse stdout | ❌ |
| Write .sig file in signing step, read in script | Decoupled; works with any signing tool | ✅ |

**Rationale**: Release.yml signing step writes `guitarhub-${{ github.ref_name }}.sig`, generate_latest_json.py reads it. Clean separation, easy to test.

### Decision: live domain updates — no restart needed

Settings are read on every image request (not cached in service struct). The `ImageCacheService` calls `settings_repo.get("allowed_image_domains")` per `get()` invocation. No restart required — changes take effect immediately.

## Data Flow

```
┌─ Signing ─────────────────────────────────────────────────────┐
│ tauri signer generate ──→ tauri.conf.json (pubkey)            │
│         ↓                                                     │
│ release.yml:                                                  │
│   cargo tauri build ──→ .deb + .AppImage                      │
│   tauri signer sign ──→ guitarhub.sig                         │
│   generate_latest_json.py ──→ latest.json (url + signature)   │
└───────────────────────────────────────────────────────────────┘

┌─ Domain Allowlist ────────────────────────────────────────────┐
│ Settings.svelte ──IPC──→ save_setting("allowed_image_domains")│
│                              ↓                                │
│ get_product_image ──→ validate_image_url(parsed domains)      │
│                              ↓                                │
│ ImageCacheService.get() ──→ domain check via SettingsRepository│
│                              ↓                                │
│                            HTTP fetch (if allowed)            │
└───────────────────────────────────────────────────────────────┘
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `src-tauri/tauri.conf.json` | Modify | Add `pubkey` from `tauri signer generate`; add `"AppImage"` to `bundle.targets` |
| `src-tauri/src/commands/image_command.rs` | Modify | Remove `static ALLOWED_DOMAINS`; read domains from settings; pass to `validate_image_url()` |
| `src-tauri/src/services/image_cache.rs` | Modify | Add `settings_repo: Arc<dyn SettingsRepository>` field; validate domain before HTTP |
| `src-tauri/src/lib.rs` | Modify | Inject `Arc<dyn SettingsRepository>` into `ImageCacheService::new()` |
| `src/lib/components/Settings.svelte` | Modify | Add text input field for `allowed_image_domains` |
| `.github/workflows/release.yml` | Modify | Remove `TAURI_SKIP_SIGNING`; add signing step; add `.sig` upload |
| `scripts/generate_latest_json.py` | Modify | Accept `.sig` file arg; build real URL; linux-x86_64 only; reject empty sig |

## Interfaces / Contracts

```rust
// image_command.rs — new signature
fn validate_image_url(raw: &str, allowed_domains: &[&str]) -> Result<Url, String>
// caller reads settings, parses ","-separated, passes to validate

// ImageCacheService — new field
pub struct ImageCacheService {
    // ... existing fields ...
    settings_repo: Arc<dyn SettingsRepository>,
}
// get() reads allowed_domains from repo before http_get()
```

```python
# generate_latest_json.py — new usage
# generate_latest_json.py <version> <signature_file>
```

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit (Rust) | `validate_image_url` with dynamic domain list | Parameterize existing tests with `&["reverb.com", "mlstatic.com"]` |
| Unit (Rust) | ImageCacheService domain rejection w/ MockSettingsRepository | Inject mock, assert `Placeholder` for blocked domain |
| Unit (Rust) | SettingsRepository trait — parse comma list | Test parse-and-fallback logic as pure fn |
| Frontend | Settings field save/load roundtrip | Vitest component test or manual verify |
| CI | Signing step produces valid `.sig` | Dry-run on `ci.yml` with test key (not in release) |
| Script | generate_latest_json.py rejects empty sig | Unit test with mock sig file |
| E2E | Image from new domain loads after settings update | Deferred (manual) |

## Migration / Rollout

1. **Key generation**: Run `tauri signer generate` once locally → copy pubkey to `tauri.conf.json` → add private key to GH secrets `TAURI_PRIVATE_KEY` + `TAURI_PRIVATE_KEY_PASSWORD`. **Must happen before merge.**
2. **Domain allowlist migration**: Existing users have no `allowed_image_domains` setting → fallback covers them. No migration needed.
3. **AppImage**: Zero-impact for existing `.deb` users. `.AppImage` appears in next release.
4. **Rollback**: See proposal.md rollback plan.

## Open Questions

- [ ] Who generates the signing keypair and sets the GH secrets? (ops task, pre-merge)
- [ ] Does `tauri signer sign` work on GitHub Actions ubuntu-latest runners out of the box, or does it need `cargo install tauri-cli`?
