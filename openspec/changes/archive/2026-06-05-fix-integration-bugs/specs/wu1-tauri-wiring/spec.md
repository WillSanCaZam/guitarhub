# Delta for wu1-tauri-wiring

## MODIFIED Requirements

### Requirement: Tauri builder with app state and invoke handler

`src-tauri/src/main.rs` MUST initialize `tauri::Builder::default()` with `.manage(state)` for `AppState` and `.invoke_handler(tauri::generate_handler![...])` listing ALL `#[tauri::command]` functions that the frontend needs to call. This MUST include `delete_setting` alongside `get_setting` and `save_setting`.

(Previously: `delete_setting` was defined with `#[tauri::command]` but omitted from `generate_handler!`, making it unreachable from the frontend.)

#### Scenario: delete_setting is callable from frontend

- GIVEN the app is running with `delete_setting` registered in `generate_handler!`
- WHEN the frontend calls `invoke("delete_setting", { key: "theme" })`
- THEN the setting is deleted and the command returns `Ok(())`

#### Scenario: All settings commands are registered

- GIVEN the app starts
- WHEN the invoke handler is initialized
- THEN `get_setting`, `save_setting`, and `delete_setting` are all present in `generate_handler!`

#### Scenario: Unregistered command returns error

- GIVEN a `#[tauri::command]` function is NOT in `generate_handler!`
- WHEN the frontend calls `invoke()` for that command
- THEN Tauri returns a "command not found" error
- AND this MUST NOT happen for any settings command
