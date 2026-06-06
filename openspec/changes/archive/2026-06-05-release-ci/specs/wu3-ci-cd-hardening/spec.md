# Delta for wu3-ci-cd-hardening

## ADDED Requirements

### Requirement: Build matrix covers 4 platform targets

`release.yml` MUST define a build matrix with 4 entries: `x86_64-pc-windows-msvc` on `windows-latest`, `x86_64-unknown-linux-gnu` on `ubuntu-latest`, `x86_64-apple-darwin` on `macos-13`, `aarch64-apple-darwin` on `macos-latest`. `fail-fast` MUST be `false`. Each build job MUST have a 30-minute timeout.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| All platforms succeed | Tag push triggers workflow | 4 matrix jobs run | 4 independent bundles produced |
| macOS fails, Linux succeeds | macOS runner fails | Matrix continues | Linux completes, macOS failure logged |

### Requirement: Linux system deps installed conditionally

Only on `runner.os == 'Linux'`, the workflow MUST install `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libgirepository1.0-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `libsoup-3.0-dev`, `libjavascriptcoregtk-4.1-dev` via `sudo apt-get install -y -qq`.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Linux build | ubuntu-latest runner | apt-get step executes | 7 deps installed |
| macOS build | macos-13 or macos-latest | Skipped by `if: runner.os` | No apt operation |

### Requirement: Build pipeline npm ci → cargo test → cargo tauri build

Every build job MUST run `actions/setup-node@v4` (Node 22, cache: npm), then `npm ci`, then `cargo test` in `src-tauri/`, then `cargo tauri build --target ${{ matrix.target }}`. `TAURI_SKIP_SIGNING: true` MUST be set at the workflow `env` level.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Happy path | All deps installed | npm ci → cargo test → build | Bundles in `target/${{ matrix.target }}/release/bundle/` |
| Test failure | Rust test fails | cargo test runs | Build skipped, job fails |

### Requirement: Artifacts uploaded per target with empty-bundle guard

After build, each job MUST upload `src-tauri/target/${{ matrix.target }}/release/bundle/` via `actions/upload-artifact@v4` with `if-no-files-found: error`. Artifact name MUST be `guitarhub-${{ matrix.target }}`.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Bundles exist | Build succeeded | Upload step runs | Artifact stored |
| No bundles | Build produced no output | Upload step runs | Job errors — "No files found" |

### Requirement: Release creation from all bundle artifacts

A `create-release` job (needs: `build`) MUST download all `guitarhub-*` artifacts with `merge-multiple: true`, discover bundle files by extension (`.deb`, `.AppImage`, `.dmg`, `.msi`, `.zip`, `.tar.gz`), and run `gh release create` with `--generate-notes`. If no bundle files are found, the job MUST `exit 1` after printing a debug file listing.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| All 4 builds succeeded | 4 artifacts uploaded | Download → find → gh release create | Release with 4+ bundle files |
| Empty artifact set | Builds produced no bundles | `find` returns empty | Job fails with debug output |

### Requirement: Update endpoint pushed to gh-pages with retry

A `publish-update-endpoint` job (needs: `create-release`) MUST checkout `gh-pages`, run `python scripts/generate_latest.json.py ${{ github.ref_name }}`, commit, and push. On push failure, it MUST retry with `git pull --rebase` up to 3 times with 5s delay. Timeout MUST be 10 minutes.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Clean push | gh-pages is current | Generate → commit → push | latest.json updated |
| Push race | Concurrent push to gh-pages | Push fails → pull --rebase → retry | Push succeeds within 3 retries |
| Max retries exceeded | 3 consecutive push failures | Loop exhausts | Job fails with push error |
