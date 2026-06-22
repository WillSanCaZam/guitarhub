# Release Process

## Versioning

GuitarHub follows **semantic versioning** (`v0.1.0`, `v0.2.0`, etc.).

- **Major**: breaking changes to the database schema, IPC API, or scraper output format
- **Minor**: new features, new source adapters, non-breaking enhancements
- **Patch**: bug fixes, documentation, CI improvements

## Pre-release Checklist

- [ ] All tests pass (`make test`)
- [ ] CHANGELOG updated with new version
- [ ] Lint passes (`make lint`)
- [ ] Audit passes (`make audit`)
- [ ] Builds locally (`npx tauri build`)

## Release Steps

```bash
# Update version in src-tauri/tauri.conf.json
# Update CHANGELOG.md with release date
git commit -m "chore: release v0.2.0"
git tag v0.2.0
git push origin master --tags
```

## What CI Does

When a tag is pushed, the release workflow (`.github/workflows/release.yml`) runs automatically:

1. **Builds** for all platforms in parallel:
   - Linux: `.deb` + `.AppImage` (x86_64)
   - macOS: `.dmg` (aarch64)
   - Windows: `.exe` (NSIS) + `.msi` (x86_64)
2. **Runs** `cargo test` on each platform
3. **Signs** bundles if `TAURI_PRIVATE_KEY` is configured
4. **Uploads** artifacts to the GitHub Release
5. **Updates** `latest.json` on the `gh-pages` branch (used by the in-app updater)

## Required Secrets

| Secret | Purpose |
|--------|---------|
| `TAURI_PRIVATE_KEY` | Bundle signing key |
| `TAURI_PRIVATE_KEY_PASSWORD` | Signing key password |

## Post-release Verification

- Check the GitHub Release page has all platform assets (`.deb`, `.AppImage`, `.dmg`, `.exe`, `.msi`)
- Verify `latest.json` on the `gh-pages` branch contains the new version URLs
- Launch the app on each platform and confirm the in-app updater detects and displays the update dialog
