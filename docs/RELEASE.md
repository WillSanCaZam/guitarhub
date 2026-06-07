# Release Process

## Versioning

GuitarHub follows **semantic versioning** (`v0.1.0`, `v0.2.0`, etc.).

- **Major**: breaking changes to the database schema, IPC API, or scraper output format
- **Minor**: new features, new source adapters, non-breaking enhancements
- **Patch**: bug fixes, documentation, CI improvements

## Pre-release Checklist

- [ ] All tests pass (Rust + frontend + Python)
- [ ] CHANGELOG updated with new version
- [ ] Lint passes (clippy, ruff, mypy)
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

1. **Builds** Linux `.deb` via `npx tauri build`
2. **Runs** `cargo test` on the Rust backend
3. **Uploads** the `.deb` artifact to the GitHub Release
4. **Updates** `latest.json` on the `gh-pages` branch (used by the in-app updater)

## Post-release Verification

- Check the GitHub Release page has the `.deb` asset attached
- Verify `latest.json` on the `gh-pages` branch contains the new version URL
- Launch the app and confirm the in-app updater detects and displays the update dialog
