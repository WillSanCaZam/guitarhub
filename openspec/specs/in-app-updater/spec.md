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

`tauri.conf.json` MUST include an `updater` block under `app` with `active: true` and `endpoints` pointing to `https://willsancazam.github.io/guitarhub/latest.json`. `pubkey` MUST be set to the project's Tauri public key.

#### Scenario: Updater config present

- GIVEN `tauri.conf.json` is inspected
- THEN `app.updater.active` is `true`
- AND `app.updater.endpoints[0]` equals the gh-pages URL
- AND `app.updater.pubkey` is a non-empty string

### Requirement: Updater capability MUST be granted

The capabilities config MUST include `tauri-plugin-updater:default` in its `permissions` array.

#### Scenario: Capability present

- GIVEN capabilities are inspected
- THEN `tauri-plugin-updater:default` appears in `permissions`

### Requirement: latest.json MUST carry platform-specific URLs

`generate_latest_json.py` MUST populate `url` and `signature` per platform. The `linux-x86_64` entry MUST point to the `.deb` or `.AppImage` asset for the tag.

#### Scenario: latest.json for a new tag

- GIVEN tag `v0.1.1` triggers the release workflow
- WHEN `generate_latest_json.py v0.1.1` runs
- THEN `latest.json` contains `"version": "0.1.1"`
- AND `platforms["linux-x86_64"].url` matches the expected Release asset URL

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
