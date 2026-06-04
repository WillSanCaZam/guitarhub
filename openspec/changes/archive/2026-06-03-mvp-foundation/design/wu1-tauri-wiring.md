# Design: WU1 — Tauri Wiring

## Technical Approach

Wire Tauri 2 runtime into the existing binary: replace the current `main.rs` (db init + exit) with a Tauri builder that manages `AppState` and registers the `get_product_image` IPC command. Then create two configuration artifacts — `tauri.conf.json` with CSP in object format and `capabilities/default.json` with `core:default` permissions.

The key insight: `initialize_database()` already returns an `AppState`. We pass it through `.manage()`, and commands receive it via `tauri::State<'_, AppState>`. No state refactoring needed.

## Architecture Decisions

### Decision: CSP as object format (not string)
- **Choice**: Object format per Tauri 2 docs
- **The risk**: String format is deprecated in Tauri 2 and silently ignored
- **Rationale**: Object format is the supported way in Tauri 2.x (stable since May 2025). The config validation at build time will catch format errors

### Decision: `core:default` only in capabilities
- **Choice**: Minimal permission set, no custom command permissions yet
- **The risk**: `get_product_image` requires an auto-generated permission
- **Mitigation**: Tauri 2 auto-generates command permissions at build time in `gen/schemas/`. After first build, we verify it appears and add to capabilities if needed
- **Rationale**: Avoid granting permissions we don't need yet. MVP has 1 command

## File Changes

### `src-tauri/src/main.rs` — Modify

**Current state**: Initializes DB, logs success, exits. No Tauri runtime.

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let db_path = std::env::var("GUITARHUB_DB_PATH")
        .unwrap_or_else(|_| "guitarhub.db".to_string());
    let _state = guitarhub_lib::initialize_database(&db_path)
        .await
        .context("Failed to initialize database on startup")?;
    tracing::info!("GuitarHub database initialized successfully");
    Ok(())
}
```

**Target state**: Tauri builder with state + invoke handler. `tauri::Builder::default()` replaces the manual tokio main.

```rust
use anyhow::Context;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let db_path = std::env::var("GUITARHUB_DB_PATH")
        .unwrap_or_else(|_| "guitarhub.db".to_string());

    let state = guitarhub_lib::initialize_database(&db_path)
        .await
        .context("Failed to initialize database on startup")?;

    tracing::info!("GuitarHub database initialized successfully");

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            guitarhub_lib::commands::image_command::get_product_image,
        ])
        .run(tauri::generate_context!())
        .context("error while running tauri application")?;

    Ok(())
}
```

**Changes needed**:
1. Replace `let _state =` with `let state =`
2. Remove the trailing `Ok(())` (Tauri builder takes over)
3. Add `.manage(state)` and `.invoke_handler(...)` and `.run(...)` chain
4. `generate_handler!` references the full path to `get_product_image`

### `src-tauri/tauri.conf.json` — Create

**Current state**: Does not exist.

**Target state**:
```json
{
  "$schema": "https://raw.githubusercontent.com/tauri-apps/tauri/dev/crates/tauri-cli/schema.json",
  "productName": "GuitarHub",
  "version": "0.1.0",
  "identifier": "com.guitarhub.app",
  "build": {
    "frontendDist": "../src",
    "devUrl": "http://localhost:5173",
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build"
  },
  "app": {
    "windows": [
      {
        "title": "GuitarHub",
        "width": 1200,
        "height": 800
      }
    ],
    "security": {
      "csp": {
        "default-src": "'self' customprotocol: asset:",
        "connect-src": "ipc: http://ipc.localhost",
        "img-src": "'self' asset: http://asset.localhost blob: data: https:",
        "style-src": "'unsafe-inline' 'self'"
      },
      "dangerousDisableAssetCspModification": false
    }
  }
}
```

**CSP rationale**:
- `connect-src ipc: http://ipc.localhost` — Tauri 2 IPC uses `ipc:` scheme in production, `http://ipc.localhost` in dev mode. Both must be present for dev+CSP to work
- `img-src 'self' asset: http://asset.localhost blob: data: https:` — images come from `https:` (catalog CDNs), `data:` (base64 URIs from cache), `asset:` (Tauri's asset protocol)
- `default-src 'self' customprotocol: asset:` — fallback policy for other resources
- `dangerousDisableAssetCspModification: false` — ensures CSP applies to asset loads

### `src-tauri/capabilities/default.json` — Create

**Current state**: Does not exist.

**Target state**:
```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "main-capability",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default"
  ]
}
```

`core:default` expands to: `core:app:default`, `core:event:default`, `core:image:default`, `core:menu:default`, `core:path:default`, `core:resources:default`, `core:tray:default`, `core:webview:default`, `core:window:default`.

## Configuration Values

| Setting | Value | Source |
|---------|-------|--------|
| CSP default-src | `'self' customprotocol: asset:` | Tauri 2 docs |
| CSP connect-src | `ipc: http://ipc.localhost` | Tauri 2 docs (IPC scheme) |
| CSP img-src | `'self' asset: http://asset.localhost blob: data: https:` | Required for CDN images + base64 cache |
| CSP style-src | `'unsafe-inline' 'self'` | Tauri 2 default for webview |
| dangerousDisableAssetCspModification | `false` | Security requirement |
| capabilities $schema | `../gen/schemas/desktop-schema.json` | Tauri 2 capabilities convention |
| capabilities permissions | `["core:default"]` | Minimal set for MVP |

## Sequence

1. Create `tauri.conf.json` — without it, `tauri::generate_context!()` fails at compile time
2. Create `capabilities/default.json` — Tauri 2 requires at least one capability
3. Modify `main.rs` — wire the builder
4. `cargo build` — verify compilation, check `gen/schemas/` for auto-generated command permissions

## Risks

| Risk | Likelihood | Mitigation |
|------|-----------|------------|
| CSP blocks IPC in dev | Medium | Test with `tauri dev` + DevTools console; both `ipc:` and `http://ipc.localhost` included |
| Auto-generated command permission missing | Low | After build, verify `gen/schemas/commands/get_product_image.json` exists; add to capabilities if absent |
| `tauri::generate_context!()` fails without config files | Low | Creates configs BEFORE modifying main.rs — ensures configs exist at compile time |
| `AppState` not `Send + Sync` | Low | Already derived via `#[derive(Clone)]` — SqlitePool and ImageCacheService are both Arc-based internally |

## Testing Approach

| Layer | What | How |
|-------|------|-----|
| Compile | `cargo build` | Must exit 0 |
| Lint | `cargo clippy` | Must exit 0 |
| Integration | `tauri dev` | Open DevTools → verify no CSP/permission errors in console |
| E2E | IPC roundtrip | Frontend calls `invoke("get_product_image", {...})` → returns data URI |
| Manual | Capability generation | Inspect `gen/schemas/` after build for `get_product_image` permission entry |

WU1 is verified when `cargo build` passes, `tauri dev` opens a window without CSP errors, and the custom command is registered (visible in auto-generated schema).
