# In-App Updater Specification

## Purpose

Enable automatic update checks via Tauri's updater plugin. The app polls `latest.json` on GitHub Pages and notifies the user when a newer version is available.

## Requirements

### Requirement: Updater plugin MUST be registered in Cargo.toml

`Cargo.toml` MUST include `tauri-plugin-updater = "2"` under `[dependencies]`.

#### Scenario: Plugin dependency present

- GIVEN `Cargo.toml` is inspected
- THEN `[dependencies]` contains `tauri-plugin-updater`
- AND it resolves without conflict with other Tauri 2 plugins

### Requirement: Updater plugin MUST be wired in the Tauri builder

`src-tauri/src/lib.rs` MUST call `.plugin(tauri_plugin_updater::init())` on the Tauri builder.

#### Scenario: Plugin registered at startup

- GIVEN the app starts
- WHEN the Tauri builder initializes
- THEN the updater plugin registers without panic
- AND the frontend can invoke `@tauri-apps/plugin-updater` APIs

### Requirement: Updater MUST be configured in tauri.conf.json

`tauri.conf.json` MUST include an `updater` block under `app` with `active: true` and `endpoints` pointing to `https://willsancazam.github.io/guitarhub/latest.json`. `pubkey` MUST contain the output of `tauri signer generate` — a non‑empty base64‑encoded public key.
(Previously: pubkey must be non‑empty; now must be a real generated key, not a placeholder)

#### Scenario: Updater config present
- GIVEN `tauri.conf.json` is inspected
- THEN `app.updater.active` is `true`
- AND `app.updater.endpoints[0]` equals the gh-pages URL
- AND `app.updater.pubkey` is a non‑empty base64‑encoded string

#### Scenario: Placeholder pubkey rejected
- GIVEN `tauri.conf.json` has pubkey set to an empty string
- WHEN CI validation runs
- THEN the build fails with a missing pubkey error

### Requirement: Updater capability MUST be granted

The capabilities config MUST include `tauri-plugin-updater:default` in its `permissions` array.

#### Scenario: Capability present

- GIVEN capabilities are inspected
- THEN `tauri-plugin-updater:default` appears in `permissions`

### Requirement: latest.json MUST carry platform-specific URLs

`generate_latest_json.py` MUST populate `url` and `signature` for the `linux-x86_64` platform only. The `url` MUST point to the `.deb` or `.AppImage` release asset. The `signature` MUST be a non‑empty base64‑encoded value produced by `tauri signer sign`. Placeholder entries for untargeted platforms (darwin, windows) MUST NOT be included.
(Previously: generated url and signature per platform with entries for all targets, potentially with empty placeholders)

#### Scenario: latest.json for a new tag
- GIVEN tag `v0.1.1` triggers the release workflow
- WHEN `generate_latest_json.py v0.1.1` runs
- THEN `latest.json` contains `"version": "0.1.1"`
- AND `platforms["linux-x86_64"].url` matches the expected Release asset URL
- AND `platforms["linux-x86_64"].signature` is a non‑empty string

#### Scenario: No placeholder platforms
- GIVEN the release workflow runs
- WHEN `latest.json` is generated
- THEN no `darwin`, `windows`, or other untargeted platform entries exist

#### Scenario: Empty signature rejected
- GIVEN `generate_latest_json.py` runs without a valid signature file
- WHEN the script attempts to set `signature` to an empty string
- THEN the script exits with an error before writing latest.json

### Requirement: CI MUST sign updater artifacts

The release pipeline MUST run `tauri signer sign` on the built bundle using the signing key stored as a GitHub secret (`TAURI_PRIVATE_KEY` + `TAURI_PRIVATE_KEY_PASSWORD`). The resulting base64 signature MUST be written to a `.sig` file and consumed by `generate_latest_json.py`. `TAURI_SKIP_SIGNING` MUST NOT be set to `true` in the release workflow.
(Previously: no signing step existed; TAURI_SKIP_SIGNING: true was set)

#### Scenario: Release artifact signed
- GIVEN the release pipeline builds the bundle
- WHEN `tauri signer sign` runs with the private key
- THEN a valid base64 signature is produced
- AND the signature appears in `latest.json`

#### Scenario: Missing signing key
- GIVEN `TAURI_PRIVATE_KEY` is not set in the CI environment
- WHEN the release pipeline reaches the signing step
- THEN the step fails with a missing key error
- AND the release is not published

### Requirement: No notification when version matches

The updater MUST NOT show a notification when the installed version equals the latest available version.

#### Scenario: Up-to-date app

- GIVEN user has v0.1.0 installed
- WHEN the updater checks `latest.json`
- AND latest.json reports version `0.1.0`
- THEN no update dialog appears

### Requirement: Notification shown when newer version exists

The updater MUST display an "Update available" dialog when `latest.json` reports a semver-greater version.

#### Scenario: New version detected

- GIVEN user has v0.1.0 installed
- WHEN the updater checks `latest.json`
- AND latest.json reports version `0.1.1`
- THEN the user sees an "Update available" dialog
