# Capability: store-connections

> **Status**: New capability  
> **Change**: store-connections

## Purpose

Let users connect external store accounts (starting with Reverb) via Personal Access Token, manage connection lifecycle, and have their personal listings appear in the unified catalog. Token storage MUST be encrypted via OS keyring.

## Requirements

### Requirement: Store Registry MUST define supported stores

The system MUST provide a hardcoded registry of `StoreDef` entries keyed by store ID, each containing `name`, `auth_type`, `icon`, `website`, and `token_url`. The registry SHALL include Reverb with auth type `pat` (Personal Access Token).

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Registry populated | App starts | Read registry | Returns `[{ id: "reverb", name: "Reverb", auth_type: "pat" }]` |
| Unknown store queried | Registry loaded | Request `id: "ebay"` | Returns `None` for undefined store |

### Requirement: Connection Manager MUST support connect/disconnect/list/validate

The system MUST provide Tauri commands `connect_store`, `disconnect_store`, `list_connections`, and `validate_token`. `connect_store` SHALL accept a store ID and token, validate via the store's API, encrypt the token, and persist the connection.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Connect succeeds | Valid Reverb PAT | `connect_store("reverb", "pat_xxx")` | Returns connection with status "connected" |
| Connect fails — bad token | Invalid PAT | `connect_store("reverb", "bad")` | Returns `AppError::InvalidInput("token invalid")` |
| Connect fails — network down | No internet | `connect_store("reverb", "pat_xxx")` | Returns `AppError::Network` |
| Disconnect | Active connection exists | `disconnect_store("reverb")` | Connection removed, user products delisted |
| List connections | 1 connected, 1 never used | `list_connections()` | Returns `[{store:"reverb",status:"connected"},{store:"gc",status:"disconnected"}]` |

### Requirement: Token MUST be encrypted at rest

Tokens SHALL be encrypted via `tauri-plugin-store` (AES-256-GCM, key derived from OS machine key). The `Debug` impl MUST redact the token. The system MUST NOT log raw tokens under any condition.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Token stored encrypted | Connection created | Read `store_connections.token_encrypted` | Value is ciphertext, NOT plaintext token |
| Debug redacted | Connection object | `format!("{:?}", conn)` | Output contains `token_encrypted: "REDACTED"` |
| Storage failure | OS keyring unavailable | `connect_store(...)` | Returns `AppError::Internal`, token NOT persisted |

### Requirement: /stores route MUST display store grid

The system MUST provide `src/routes/stores/+page.svelte` that renders a responsive grid of supported stores, each showing a connect/disconnect button, connection status badge (connected/disconnected/error), and the store name+icon.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Route renders | App running | Navigate to `/stores` | Store grid visible with Reverb card |
| Connect button | Reverb disconnected | Click "Connect" | Opens modal with inline guide + token input |
| Status badge | Reverb connected | Navigate to `/stores` | Shows green "Connected as @username" badge |
| Empty state | No connections | Route loads | All stores show "Connect" state |

### Requirement: Reconnect MUST replace existing token

If a connection already exists for a store, `connect_store` SHALL overwrite the stored token and re-sync the user's listings.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Token refresh | Reverb connected with old token | `connect_store("reverb", "new_pat")` | Token replaced, listings re-synced |
