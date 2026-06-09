# Delta for wu3-ci-cd-hardening

## MODIFIED Requirements

### Requirement: Bundle config MUST declare explicit targets

`tauri.conf.json` MUST include a `bundle` section with `targets: ["deb", "AppImage"]`, `identifier: "com.guitarhub.app"`, and valid `icon` paths.
(Previously: targets: ["deb"] only)

#### Scenario: Bundle config present
- GIVEN `tauri.conf.json` is inspected
- THEN `bundle.active` is `true`
- AND `bundle.targets` includes both `deb` and `AppImage`
- AND `bundle.icon` references existing files

#### Scenario: AppImage built on release
- GIVEN the release workflow runs on `ubuntu-latest`
- WHEN `cargo tauri build` completes
- THEN a `.AppImage` file exists in the bundle output directory
- AND the `.AppImage` is uploaded as a release artifact

### Requirement: Build pipeline npm ci → cargo test → cargo tauri build

Every build job MUST run `actions/setup-node@v4` (Node 22, cache: npm), then `npm ci`, then `cargo test` in `src-tauri/`, then `cargo tauri build --target ${{ matrix.target }}`. After the build step, the job MUST run `tauri signer sign` using the private key from the `TAURI_PRIVATE_KEY` secret. `TAURI_SKIP_SIGNING` MUST NOT be set.
(Previously: TAURI_SKIP_SIGNING: true set at workflow env level; no signing step)

#### Updated Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Happy path — signed build | All deps installed, key present | npm ci → cargo test → build → sign | Bundle + signature produced |
| Test failure | Rust test fails | cargo test runs | Build + signing skipped, job fails |
| Signing key missing | TAURI_PRIVATE_KEY not set | `tauri signer sign` runs | Step fails, job reports error |

### Requirement: Artifacts uploaded per target with empty-bundle guard

After build, each job MUST upload `src-tauri/target/${{ matrix.target }}/release/bundle/` via `actions/upload-artifact@v4` with `if-no-files-found: error`. Artifact name MUST be `guitarhub-${{ matrix.target }}`. The upload MUST include both `.deb` and `.AppImage` bundle files.
(Previously: only .deb was produced; now includes .AppImage)

#### Updated Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Both bundles exist | Build succeeded for both targets | Upload step runs | Both `.deb` and `.AppImage` stored |
| AppImage missing | AppImage build failed, .deb succeeded | Upload step runs | Job errors — `if-no-files-found: error` fires |

### Requirement: Release creation from all bundle artifacts

A `create-release` job (needs: `build`) MUST download all `guitarhub-*` artifacts with `merge-multiple: true`, discover bundle files by extension (`.deb`, `.AppImage`, `.dmg`, `.msi`, `.zip`, `.tar.gz`), and run `gh release create` with `--generate-notes`. If no bundle files are found, the job MUST `exit 1` after printing a debug file listing.
(Previously: .AppImage was mentioned in extensions but never produced; now it is actually built and uploaded)

#### Scenario: Release includes AppImage
- GIVEN the release pipeline succeeds
- WHEN `gh release create` runs
- THEN the release includes both the `.deb` and `.AppImage` assets
