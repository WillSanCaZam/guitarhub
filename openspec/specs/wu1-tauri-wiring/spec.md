# Tauri Wiring Specification

## Purpose

Wire Tauri 2 runtime infrastructure: app builder with state and invoke handler, Content Security Policy in object format, and capability-based permissions. Enables all backend-to-frontend IPC for the MVP.

## Requirements

### Requirement: Tauri builder with app state and invoke handler

`src-tauri/src/main.rs` MUST initialize `tauri::Builder::default()` with `.manage(state)` for `AppState` and `.invoke_handler(tauri::generate_handler![...])` listing ALL `#[tauri::command]` functions that the frontend needs to call. This MUST include the settings commands `get_setting`, `save_setting`, and `delete_setting`. `AppState` parameter type MUST be `tauri::State<'_, AppState>`.

A `#[tauri::command]` function that is NOT listed in `generate_handler!` is unreachable from the frontend and MUST NOT happen for any settings command.

#### Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Build compiles | All deps present in Cargo.toml | `cargo build` | Exits 0, no errors |
| State injectable | `main.rs` manages `AppState` | Command calls `tauri::State<'_, AppState>` | State accessible, Ok returned |
| Invoke resolves | Frontend calls `invoke("get_product_image", {...})` | IPC roundtrip | Command runs, returns expected JSON |
| `delete_setting` is callable | App running with `delete_setting` registered | Frontend calls `invoke("delete_setting", { key: "theme" })` | Setting is deleted, command returns `Ok(())` |
| All settings commands registered | App starts | Invoke handler initialized | `get_setting`, `save_setting`, and `delete_setting` are all present in `generate_handler!` |
| Unregistered command returns error | A `#[tauri::command]` function is NOT in `generate_handler!` | Frontend calls `invoke()` for that command | Tauri returns a "command not found" error |

### Requirement: Content Security Policy in object format

`src-tauri/tauri.conf.json` MUST define `security.csp` as an object (not a string), per Tauri 2 docs. MUST include `connect-src ipc: http://ipc.localhost` for IPC and `img-src 'self' asset: http://asset.localhost blob: data: https:` for images.

#### Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| IPC not blocked | CSP includes `connect-src ipc: http://ipc.localhost` | Frontend invokes command | DevTools console shows no CSP violation |
| Images load | CSP includes `img-src https: data: asset:` | Frontend renders image from cache | Image displays, no CSP error |
| Dev mode IPC | Tauri dev server running | Invoke command | IPC reaches backend via `http://ipc.localhost` endpoint |

### Requirement: Capability-based permissions

`src-tauri/capabilities/default.json` MUST define identifier `main-capability` targeting `windows: ["main"]` with `permissions: ["core:default"]`. The `$schema` MUST reference `../gen/schemas/desktop-schema.json`.

#### Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Minimal permissions | `capabilities/default.json` with only `core:default` | `tauri dev` opens window | Window renders, no permission errors |
| Custom command registered | `get_product_image` compiled | Build completes | Auto-generated permission appears in `gen/schemas/` |

### Requirement: Dangerous asset CSP modification disabled

`src-tauri/tauri.conf.json` MUST set `security.dangerousDisableAssetCspModification` to `false`.

#### Scenario: Asset loading

- GIVEN `dangerousDisableAssetCspModification` is `false`
- WHEN Tauri serves local asset files
- THEN CSP asset: rules apply, no bypass

## Acceptance Criteria

| Criterion | How to verify |
|-----------|---------------|
| `cargo build` passes | `cargo build --release` exits 0 |
| No CSP violations in console | `tauri dev`, open devtools — no CSP errors |
| IPC commands work | Frontend invokes command, receives expected response |
| Capabilities valid | `tauri dev` does not log permission-denied errors |
