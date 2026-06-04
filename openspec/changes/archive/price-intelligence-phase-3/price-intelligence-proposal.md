# Proposal: price-intelligence-phase-3

## Intent

Users need price context to make buying decisions: historical trends, fairness indicators, and the ability to export their data. No servers, no cost — all local SQL computation.

## Scope

**In (4 features):**
- **F1**: Price History Chart — SVG line chart on ProductDetail, 365d window, one line per source, no chart library
- **F2**: "Good Price" Indicator — badge on ProductCard, SQL rolling stats, colored by threshold
- **F3**: Ntfy/Webhook Alert Delivery — configurable channel (App notifications / Ntfy.sh / webhook POST)
- **F4**: Data Export — ZIP with wishlist, price_history, settings via save dialog

**Out:** Price forecasting / ML predictions, GTIN dedup, AVIF/WEBP image conversion, multi-currency conversion, shared alert server, cloud sync.

**Constraints:** Zero infra cost, offline-first, strict TDD (RED→GREEN→REFACTOR), F-Droid compatible.

## Capabilities

**New:**
- `price-history` — chart query, outlier filtering, SVG rendering
- `price-insight` — rolling stats badge via pure SQL
- `alert-delivery` — Ntfy.sh / generic webhook dispatcher + App notifications
- `data-export` — SQLite → ZIP export with tauri-plugin-dialog
- `app-settings` — persistent key-value store (settings table)

**Modified:** `wishlist` — export support (no spec change, pure implementation).

## Approach

### F1: Price History Chart
- Migration **004**: add `source_id TEXT NOT NULL DEFAULT ''` to `price_history` + index `(sku, recorded_at)`
- `repository/price_history.rs` — `get_history(sku, window_days) → Vec<PricePoint>` with 5σ outlier filter in SQL
- `commands/price_command.rs` — `#[tauri::command] get_price_history(sku, window_days)` → IPC
- `ProductDetail.svelte` — new component, pure SVG `<polyline>` per source, viewBox scaling, "Insufficient data" if rows < 30

### F2: Good Price Indicator
- `commands/price_command.rs` — `get_price_insight(sku) → {level, pct}` with SQL:
  ```sql
  SELECT MIN(price), AVG(price) FROM price_history
  WHERE sku=?1 AND recorded_at >= ?2
  ```
- Badge: green if price ≤ 30d_min×1.05, amber if price ≥ 90d_avg×1.20, hidden if rows < 30
- `ProductCard.svelte` — add badge div, invoke `get_price_insight`

### F3: Alert Delivery
- `services/alert_service.rs` — `AlertService` trait + impls: `AppNotificationAlert`, `NtfyAlert`, `WebhookAlert`
- Config stored in new `settings` table (kv store, migration **005**)
- `commands/settings_command.rs` — `get_setting`, `save_setting` IPC
- `Settings.svelte` — form: channel selector, URL field, test button
- Uses existing `reqwest` for HTTP POST, `tauri::api::notification` for app alerts

### F4: Data Export
- `commands/export_command.rs` — `export_data(path)` → queries wishlist, price_history, settings → writes ZIP
- `services/export_service.rs` — uses `zip` crate, temp buffer, writes to user-chosen path
- Requires `tauri-plugin-dialog` (save dialog), `zip` crate in `Cargo.toml`
- `Settings.svelte` — "Export all data" button

## Inter-feature Dependencies

- F2 (price insight SQL) and F1 (price history query) share `price_history` table — no conflict
- F3 and F4 share `settings` table + `Settings.svelte` UI
- All 4 depend on migrations 004 (price_history + source_id) and 005 (settings table)

## Affected Areas

| Area | Action |
|------|--------|
| `src-tauri/Cargo.toml` | Add `zip`, `tauri-plugin-dialog` |
| `src-tauri/src/main.rs` | Register new IPC commands, plugins |
| `src-tauri/tauri.conf.json` | Add dialog permissions |
| `src-tauri/src/repository/sqlite/migrations/004_add_price_source.sql` | Create |
| `src-tauri/src/repository/sqlite/migrations/005_add_settings.sql` | Create |
| `src-tauri/src/repository/price_history.rs` | Create |
| `src-tauri/src/repository/mod.rs` | Wire |
| `src-tauri/src/services/alert_service.rs` | Create |
| `src-tauri/src/services/export_service.rs` | Create |
| `src-tauri/src/services/mod.rs` | Wire |
| `src-tauri/src/commands/price_command.rs` | Create |
| `src-tauri/src/commands/settings_command.rs` | Create |
| `src-tauri/src/commands/export_command.rs` | Create |
| `src-tauri/src/commands/mod.rs` | Wire |
| `src/lib/components/ProductCard.svelte` | Add price badge |
| `src/lib/components/ProductDetail.svelte` | Create (SVG chart) |
| `src/lib/components/Settings.svelte` | Create (alert config + export) |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| ZIP crate adds native dep | Med | `zip` is pure Rust, no C deps — verify for F-Droid |
| Ntfy.sh URL stored in plaintext | Low | Settings are local-only, no sync. Document as conscious tradeoff |
| SVG chart perf with 365+ points | Low | Use `<polyline>` with downsampled points, `requestAnimationFrame` for render |
| `tauri-plugin-dialog` API changes | Low | Pin version, test in CI |

## Effort Breakdown

| Feature | Rust (est.) | Svelte (est.) | Total |
|---------|------------|---------------|-------|
| F1: Price History | 3h | 4h | 7h |
| F2: Price Indicator | 1h | 2h | 3h |
| F3: Alert Delivery | 4h | 2h | 6h |
| F4: Data Export | 3h | 1h | 4h |
| **Total** | **11h** | **9h** | **20h** |

## Rollback Plan

Per-feature: revert the migration, delete the module, remove from `main.rs` invoke_handler. No shared schema changes across features (migrations are additive). `tauri-plugin-dialog` and `zip` can stay in `Cargo.toml` harmlessly. Full rollback = revert merge commit.

## Success Criteria

- [ ] `cargo test` + `cargo clippy` pass on all new code
- [ ] Price chart renders 365d of multi-source data with outlier removal
- [ ] Good price badge appears green/amber per threshold, hidden when <30d data
- [ ] Ntfy.sh test notification arrives on configured topic
- [ ] Data export ZIP contains all 3 files, valid archive
- [ ] Zero new `unsafe` blocks
- [ ] No network calls for F1, F2 (local SQL only)
