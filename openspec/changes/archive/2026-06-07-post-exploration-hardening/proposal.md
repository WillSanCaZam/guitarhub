# Proposal: Post-Exploration Hardening

## Intent

Fix verified gaps found in exploration: updater delivers unsigned payloads (pubkey empty), image domain allowlist is hardcoded (blocks new scraper sources), packaging only produces `.deb` (no AppImage), and release scripts still generate empty URL/signature placeholders. These are genuine hardening issues — security, maintainability, and CI completeness.

## Scope

### In Scope
1. **Updater signing**: Generate keypair, set `pubkey` in tauri.conf.json, wire signing in release.yml, remove `TAURI_SKIP_SIGNING`
2. **Image domain allowlist**: Read from `settings` table instead of `static ALLOWED_DOMAINS`, add UI controls, fallback to `["reverb.com", "mlstatic.com"]`
3. **Packaging: add AppImage**: Add `"AppImage"` to `bundle.targets`, update release.yml artifact paths
4. **generate_latest_json.py**: Populate real `url` + `signature` per platform, drop placeholder entries for untargeted platforms

### Out of Scope
- Landing page Astro → separate change (`feature/astro-landing-page`)
- Second scraper adapter → separate change
- Windows (.msi) / macOS (.dmg) builds → deferred; cross-platform runner verification needed
- AUR/Flathub/F-Droid packaging → each is a separate change
- Frontend test gaps → already covered (exploration data was stale)

## Capabilities

### New Capabilities
None — all changes modify existing specs.

### Modified Capabilities
- `in-app-updater`: Add pubkey requirement, signing step in CI, non-empty signature in latest.json
- `wu2-security-hardening`: Image domain allowlist MUST be configurable via settings table (not hardcoded static)
- `wu3-ci-cd-hardening`: Bundle MUST produce AppImage on Linux; CI MUST sign updater artifacts
- `local-image-cache`: Image validation MUST read allowed domains from `get_setting("allowed_image_domains")`

## Approach

1. **Updater signing**: `tauri signer generate` → save pubkey in config, private key as GH secret → `TAURI_SKIP_SIGNING: false` + `tauri signer sign` in release.yml
2. **Domain allowlist**: `SqliteSettingsRepository.get("allowed_image_domains")` → parse comma-separated list → fallback to static set → add frontend setting field + IPC save
3. **AppImage**: Add `"AppImage"` to `bundle.targets` in tauri.conf.json → update release.yml artifact upload to include `.AppImage` → update `generate_latest_json.py` to set linux-x86_64 URL
4. **latest.json**: Generate only `linux-x86_64` platform entry (remove placeholder windows/darwin entries)

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src-tauri/tauri.conf.json` | Modified | Add pubkey, add AppImage target |
| `src-tauri/src/commands/image_command.rs` | Modified | Read domains from settings, fallback to static |
| `.github/workflows/release.yml` | Modified | Wire signing step, add AppImage to artifacts |
| `scripts/generate_latest_json.py` | Modified | Populate URL + signature, linux-only |
| `src/lib/components/Settings.svelte` | Modified | Add image domain allowlist field |
| `.github/workflows/ci.yml` | Modified | Verify signed update artifact in test build |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Signing key leaked | Low | Store as GH encrypted secret; rotate if compromised |
| Settings domain parsing breaks image loading | Med | Fallback to built-in allowlist on parse failure |
| AppImage size > CDN limit (GH Pages 100MB) | Low | Already ~15MB Tauri app; monitor at release |

## Rollback Plan

- **Signing breaks release**: set `TAURI_SKIP_SIGNING: true` + restore empty pubkey → ship unsigned release
- **Settings domain breaks images**: delete `allowed_image_domains` setting → code falls back to static list
- **AppImage CI fails**: revert `bundle.targets` addition; release `.deb` only

## Dependencies

- `tauri signer` CLI (bundled with `tauri-cli` ≥2.0)
- GitHub Actions runner `ubuntu-latest` must have `appimagetool` available (or use bundled Tauri AppImage creation)

## Success Criteria

- [ ] `tauri.conf.json` has non-empty `pubkey`; `TAURI_SKIP_SIGNING` removed from CI
- [ ] `release.yml` signs the `.deb` + `.AppImage` artifacts
- [ ] `generate_latest_json.py` outputs non-empty `signature` and valid `url`
- [ ] Image domain allowlist can be set/read via `Settings.svelve` form and persists across restarts
- [ ] `make test` passes (all existing 293+ Rust tests + frontend tests)
