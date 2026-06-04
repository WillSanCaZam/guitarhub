# Structured Errors Specification

> **Status**: New capability  
> **Change**: fix-critical-fallas

## Purpose

Introduce a typed error enum (`AppError`) replacing raw `String` errors across all Tauri IPC commands, and consolidate HTTP client usage to reuse the `AppState.http_client` instance instead of creating per-request clients.

## Requirements

### Requirement: AppError enum MUST be defined with thiserror

The system MUST define `AppError` in `lib.rs` or a new `errors.rs` module using `thiserror`. Variants MUST include: `NotFound`, `InvalidInput(String)`, `Database(String)`, `Network(String)`, `Internal(String)`. The enum MUST implement `Serialize` for Tauri IPC serialization.

#### Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| NotFound serializes | Command returns `Err(AppError::NotFound)` | Frontend catches error | Error contains variant discriminant |
| InvalidInput carries message | Validation fails | Command returns `InvalidInput("sku_required")` | Message visible in error payload |
| DB error wraps source | SQL query fails | Command returns `Database(msg)` | Error includes query error details |

### Requirement: All Tauri commands MUST return AppError

Every `#[tauri::command]` function MUST return `Result<T, AppError>` instead of `Result<T, String>`: `get_product_image`, `get_price_history`, `get_price_insight`, `get_setting`, `save_setting`, `test_alert_channel`, `export_data`, `delete_setting`, and `sync_catalog`.

#### Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Image error typed | Invalid URL to `get_product_image` | Command returns `InvalidInput(...)` | Correct variant, test assertions updated |
| Price error typed | Empty SKU to `get_price_insight` | Command returns `InvalidInput(...)` | No `sku_required` string leak, variant used |
| Export error typed | Empty path to `export_data` | Command returns `InvalidInput(...)` | Wraps write_error context |

### Requirement: AlertService MUST reuse AppState http_client

`NtfyAlert::new` and `WebhookAlert::new` MUST accept `reqwest::Client` from the caller. `test_alert_channel_cmd` MUST receive the client via `state.http_client` rather than creating a new client inline. The inline `reqwest::Client::new()` calls in `alert_service.rs` test helpers SHOULD be replaced with the shared client pattern.

#### Scenarios

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Shared client passed | `AppState.http_client` exists | Invoke `test_alert_channel("ntfy", ...)` | `state.http_client` passed to `NtfyAlert::new` |
| App channel bypasses client | Channel is "app" | `test_alert_channel_cmd("app", ...)` | Returns `unsupported_in_test`, client unused |
