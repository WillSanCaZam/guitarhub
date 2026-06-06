# Design: Release CI Pipeline

## Technical Approach

Sequential 4-job pipeline on tag push (`v*`). Build runs across a 4-target matrix with `fail-fast: false`. Artifacts collected by a separate `create-release` job, then updater metadata pushed to `gh-pages`. All builds use direct `cargo tauri build` (not `tauri-action`).

Implements spec `wu3-ci-cd-hardening` requirements: concurrency gate, 30-min timeout, 4 targets, conditional Linux deps, Tauri 2 compatible build, asset upload, release creation, updater push with retry.

## Architecture

### Flow Diagram

```
Tag push v*
     │
     ▼
┌─────────────────────────────┐
│ concurrency: gh-pages-publish│
│ cancel-in-progress: false    │
└──────────┬──────────────────┘
           │ serializes gh-pages commits,
           │ does NOT cancel running builds
           ▼
┌──────────────────────────────────────┐
│            build (matrix ×4)         │
│                                      │
│  ├─ linux  (ubuntu-latest)           │
│  ├─ win    (windows-latest)          │
│  ├─ mac-x64 (macos-13)               │
│  └─ mac-arm (macos-latest)           │
│                                      │
│  Steps per job:                      │
│  checkout → rustup + target          │
│  → [Linux: apt deps]                 │
│  → setup-node + npm ci               │
│  → cargo test                        │
│  → cargo tauri build                 │
│  → upload-artifact (guitarhub-*)     │
└──────────┬───────────────────────────┘
           │ 4 artifacts:
           │ guitarhub-x86_64-pc-windows-msvc
           │ guitarhub-x86_64-unknown-linux-gnu
           │ guitarhub-x86_64-apple-darwin
           │ guitarhub-aarch64-apple-darwin
           ▼
┌──────────────────────────────────────┐
│         create-release               │
│                                      │
│  download-artifact (guitarhub-*)     │
│  → merge-multiple                    │
│  → find *.deb *.AppImage *.dmg *.msi │
│  → gh release create                 │
│  → if empty: exit 1 + debug output   │
└──────────┬───────────────────────────┘
           ▼
┌──────────────────────────────────────┐
│    publish-update-endpoint           │
│                                      │
│  checkout gh-pages                   │
│  → python generate_latest_json.py    │
│  → git commit                        │
│  → git push (3x retry with rebase)   │
└──────────────────────────────────────┘
```

## Architecture Decisions

### Decision: Direct `cargo tauri build` over `tauri-action`

| Option | Tradeoff | Decision |
|--------|----------|----------|
| `tauri-apps/tauri-action@v0` | Abstracted but pinned to Tauri 1 API — incompatible with Tauri 2 | ❌ Rejected |
| `cargo tauri build --target` | Explicit, works with Tauri 2, full error visibility, no action version risk | ✅ Chosen |

### Decision: `TAURI_SKIP_SIGNING: true`

macOS code signing requires an Apple developer cert in the runner keychain. No cert available. Skipping signing means the `.dmg` triggers a Gatekeeper warning on first launch — deferred until certs are procured.

### Decision: `fail-fast: false`

Each matrix job is independent. A macOS runner timeout should not abort a passing Linux or Windows build. Without this, GHA cancels all in-progress matrix jobs on any single failure.

### Decision: `concurrency` + `cancel-in-progress: false`

Prevents two concurrent tag pushes from racing on the `gh-pages` updater push. `cancel-in-progress: false` ensures an in-flight release is NOT cancelled by a subsequent push — it completes, then the next one starts.

### Decision: Separate `create-release` job

If `gh release create` fails (missing assets, token issue), only the release job needs re-run — not all 4 builds. `download-artifact` reuses already-built bundles from the same workflow run.

## Data Flow

### Bundle paths by target

| Target | Bundle dir | Extensions |
|--------|-----------|------------|
| `x86_64-unknown-linux-gnu` | `target/release/bundle/deb/`, `.../appimage/` | `.deb`, `.AppImage` |
| `x86_64-pc-windows-msvc` | `target/release/bundle/msi/` | `.msi` |
| `x86_64-apple-darwin` | `target/release/bundle/dmg/` | `.dmg` |
| `aarch64-apple-darwin` | `target/release/bundle/dmg/` | `.dmg` |

### Artifact naming

```
guitarhub-<matrix.target>
       │
       └── artifact name — unique per target, pattern-matched in create-release
```

### Asset discovery (create-release)

```
find . -type f \( -name "*.deb" -o -name "*.AppImage" \
  -o -name "*.dmg" -o -name "*.msi" \
  -o -name "*.zip" -o -name "*.tar.gz" \) | head -20
```

If empty: `exit 1` after dumping `find . -type f | head -30` for debugging.

### Updater flow

```
generate_latest_json.py v0.1.0
  → writes latest.json (version, pub_date, empty platform URLs)
  → git commit -am "chore: update latest.json to v0.1.0"
  → git push (3x retry: pull --rebase → push, 5s delay)
  → lands on gh-pages branch → served via GitHub Pages
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `.github/workflows/release.yml` | Modified | Full rewrite: matrix build, artifact upload, create-release, updater push |
| `src/routes/__tests__/+page.test.ts` | Renamed | → `page.test.ts` — prevents SvelteKit routing conflict in production build |

## Failure Modes

| Failure | Trigger | Behavior | Recovery |
|---------|---------|----------|----------|
| macOS codesign error | No Apple cert in keychain | `TAURI_SKIP_SIGNING` suppresses it | N/A — stopgap until cert procured |
| npm install flake | Network blip, registry down | `npm ci` fails at install step | Retry by pushing tag again |
| `cargo test` failure | Rust test regression | Build step skipped, job fails | Fix test, push new tag (artifact never reached) |
| Empty artifact set | Build produced no bundles | Upload errors (`if-no-files-found: error`) | Debug from upload error; push new tag |
| `gh release create` fails | Token scope, API rate limit | Release job fails, no artifacts published | Fix issue, re-run job (build artifacts cached) |
| gh-pages push race | Concurrent push to gh-pages | Push rejected → 3x retry with rebase → if exhausted, job fails | Manual merge + push |

## Security Implications

- `TAURI_SKIP_SIGNING`: unsigned macOS builds trigger Gatekeeper — acceptable until cert procured
- `GITHUB_TOKEN`: auto-scoped to workflow, `contents: write` only on `create-release` job — least privilege
- No secrets stored; codesign certs deferred to future change

## Open Questions

None — all decisions gated by cert availability, not design ambiguity.
