# Price Intelligence — Specification

> Phase 3 capabilities for GuitarHub: price history charts, fair-price indicators, alert delivery, data export, and shared app settings.

---

## Capability: price-history

### Description

Query price history from the local SQLite database, filter statistical outliers, and render an SVG line chart on the ProductDetail page. Supports up to 365 days of history with one line per source.

### Dependencies

- Migration **004**: `source_id TEXT NOT NULL DEFAULT ''` column on `price_history`, index `(sku, recorded_at)`.
- Requires `price_history` table to exist (from earlier phase).

### Functional Requirements

- REQ-PH-1: The system MUST accept a `sku` (String) and `window_days` (u32, default 365) parameter.
- REQ-PH-2: The system MUST query `price_history` for rows within the window, ordered by `recorded_at ASC`.
- REQ-PH-3: The system MUST exclude outliers beyond 5 standard deviations (5σ) from the mean per source.
- REQ-PH-4: If fewer than 30 data points exist for a source, the system SHOULD include those points but the frontend MUST show an "Insufficient data" message for that source.
- REQ-PH-5: The SVG chart MUST render one `<polyline>` per unique `source_id`.
- REQ-PH-6: The SVG MUST use `viewBox` for responsive scaling (width 100% of container, aspect ratio preserved).
- REQ-PH-7: The chart MUST have accessible `role="img"` and an `<title>` describing the data.
- REQ-PH-8: The system MUST NOT make any network calls — all data comes from local SQLite.

### IPC Contract

| Field | Value |
|-------|-------|
| Command | `get_price_history` |
| Params | `{ sku: String, window_days?: u32 }` |
| Returns | `Vec<PricePoint> { source_id: String, recorded_at: i64, price: f64 }` |
| Error | `"sku_required"` if SKU is empty; `"no_data"` if no rows after filtering |

### Scenarios

**Scenario 1: Full 365-day chart with multi-source data**
GIVEN a SKU with 400+ price records from 2 sources spanning 365 days
WHEN the frontend invokes `get_price_history(sku: "ABC123", window_days: 365)`
THEN the backend returns `Vec<PricePoint>` with outliers filtered (5σ per source)
AND the frontend renders an SVG with 2 `<polyline>` elements
AND the chart fills 100% of its container width

**Scenario 2: Insufficient data for a source**
GIVEN a SKU with only 25 records from source "A" and 200 from source "B"
WHEN the frontend invokes `get_price_history(sku: "ABC123")`
THEN the backend returns all records (no outlier filter applied for source "A")
AND the frontend renders source "B" as a line and shows "Insufficient data" for source "A"

**Scenario 3: No price history at all**
GIVEN a SKU with zero rows in `price_history`
WHEN the frontend invokes `get_price_history(sku: "UNKNOWN")`
THEN the backend returns a `"no_data"` error
AND the frontend shows an empty state message: "No price history available"

**Scenario 4: Single data point**
GIVEN a SKU with exactly 1 record in `price_history`
WHEN the frontend invokes `get_price_history(sku: "ABC123")`
THEN the backend returns the single point as a `Vec` of length 1
AND the frontend renders it as a single dot (no line) with "Insufficient data" note

### Acceptance Criteria

- [ ] `cargo test` passes for `get_price_history` query, outlier filter, and edge cases
- [ ] SVG chart renders correctly with 1–5 sources, no visual overlap
- [ ] Chart is responsive — resizing the browser reflows the SVG
- [ ] Axe DevTools or equivalent reports no accessibility violations on the SVG
- [ ] No network calls observed in DevTools Network tab when chart loads

---

## Capability: price-insight

### Description

Compute a rolling price insight (green/amber/hidden badge) entirely in SQL. Displayed on `ProductCard` to inform users whether the current price is a good deal.

### Dependencies

- `price_history` table with 30+ days of data for the SKU.

### Functional Requirements

- REQ-PI-1: The system MUST accept a `sku` (String) parameter.
- REQ-PI-2: The system MUST compute `min_30d` = MIN(price) WHERE `recorded_at >= NOW() - 30 days`.
- REQ-PI-3: The system MUST compute `avg_90d` = AVG(price) WHERE `recorded_at >= NOW() - 90 days`.
- REQ-PI-4: The system MUST classify the insight as:
  - **green** if `current_price <= min_30d * 1.05`
  - **amber** if `current_price >= avg_90d * 1.20`
  - **hidden** if fewer than 30 data points exist in the 30d window
- REQ-PI-5: The system MUST return the current `current_price` along with the insight.
- REQ-PI-6: The badge MUST NOT be fetched until product data is loaded (avoid cascading requests).

### IPC Contract

| Field | Value |
|-------|-------|
| Command | `get_price_insight` |
| Params | `{ sku: String }` |
| Returns | `PriceInsight { level: "green" | "amber" | "hidden", pct: f64, current_price: f64, min_30d: f64, avg_90d: f64 }` |
| Error | `"sku_required"` if SKU empty |

### Scenarios

**Scenario 1: Green badge — current price is near 30-day low**
GIVEN a SKU where `current_price` is 100, `min_30d` is 98, and 45 data points exist
WHEN `get_price_insight(sku: "ABC123")` is called
THEN the response includes `level: "green"` and `pct: 2.04`
AND the frontend renders a green badge with "Good price" text

**Scenario 2: Amber badge — current price exceeds 90-day average by 20%+**
GIVEN a SKU where `current_price` is 250, `avg_90d` is 200 (ratio = 1.25), and 100 data points exist
WHEN `get_price_insight(sku: "ABC123")` is called
THEN the response includes `level: "amber"` and `pct: 25.0`
AND the frontend renders an amber badge with "Above average" text

**Scenario 3: Hidden badge — insufficient data**
GIVEN a SKU with only 20 records in the 30-day window
WHEN `get_price_insight(sku: "ABC123")` is called
THEN the response includes `level: "hidden"` and `pct: 0.0`
AND the frontend does NOT render any badge

### Acceptance Criteria

- [ ] `cargo test` passes for price insight query, all three classification paths
- [ ] Green badge renders on ProductCard when price ≤ `min_30d × 1.05`
- [ ] Amber badge renders when price ≥ `avg_90d × 1.20`
- [ ] No badge renders when data count < 30
- [ ] Badge appears without page re-render (Svelte reactive state)
- [ ] Query completes in < 50ms on a DB with 10k+ price_history rows (add benchmark assertion)

---

## Capability: alert-delivery

### Description

Deliver price-drop alerts through configurable channels: in-app notifications, Ntfy.sh push, or generic webhook POST. Users configure delivery via the Settings UI.

### Dependencies

- `app-settings` capability (settings table for channel config).
- Exiting `reqwest` HTTP client (already in `Cargo.toml`).
- `tauri::api::notification` for app-level alerts.

### Functional Requirements

- REQ-AD-1: The system MUST define an `AlertChannel` enum: `App`, `Ntfy`, `Webhook`.
- REQ-AD-2: The system MUST persist the active channel and its configuration in the `settings` table.
- REQ-AD-3: The Ntfy channel MUST POST to `https://ntfy.sh/{topic}` with `Title`, `Message`, `Priority`, and `Tags` headers.
- REQ-AD-4: The Webhook channel MUST POST a JSON payload `{ title, message, sku, price, url }` to a user-configured URL.
- REQ-AD-5: The Settings UI MUST offer a channel selector (radio or select), a URL/text field for Ntfy topic or webhook URL, and a "Test" button.
- REQ-AD-6: The "Test" button MUST send a sample notification and report success/failure within 5 seconds.
- REQ-AD-7: The App channel MUST use `tauri::api::notification::Notification` to show a native OS notification.
- REQ-AD-8: On failure, the system MUST log the error and MUST NOT crash the app.
- REQ-AD-9: The system SHOULD retry failed HTTP deliveries once after 3 seconds.
- REQ-AD-10: The system MUST validate URLs: reject empty strings, reject non-http(s) protocols.

### IPC Contract

| Field | Value |
|-------|-------|
| Command | `test_alert_channel` |
| Params | `{ channel: String, config: String }` |
| Returns | `{ success: bool, message: String }` |
| Error | `"invalid_channel"` for unknown channel; `"timeout"` after 5s |

### Scenarios

**Scenario 1: Ntfy.sh sends successfully**
GIVEN the user configured channel "ntfy" with topic "guitar-deals"
WHEN the system sends a test alert via `test_alert_channel(channel: "ntfy", config: "guitar-deals")`
THEN an HTTP POST is made to `https://ntfy.sh/guitar-deals` with valid headers
AND the response indicates `success: true`

**Scenario 2: Webhook URL is unreachable**
GIVEN the user configured channel "webhook" with URL `https://invalid.example.com/webhook`
WHEN the system sends a test alert
THEN the HTTP POST fails with a connection error
AND the system returns `success: false, message: "Connection refused"`
AND the error is logged but the app does not crash

**Scenario 3: URL validation rejects invalid input**
GIVEN the user enters "not-a-url" as the webhook URL
WHEN the system attempts to validate or send
THEN the system MUST reject with `"invalid_url"`
AND the Settings UI shows a validation error

**Scenario 4: App notification delivers natively**
GIVEN the user configured channel "app"
WHEN the system sends a test alert
THEN a native OS notification is displayed via Tauri notifications API
AND the backend returns `success: true`

### Acceptance Criteria

- [ ] `cargo test` passes for all alert service implementations
- [ ] Ntfy.sh test notification arrives at the configured topic (manual verification)
- [ ] Generic webhook POST reaches a local HTTP test server (e.g., `httpbin`)
- [ ] App notification appears as a native OS toast
- [ ] Settings UI test button shows pass/fail feedback within 5 seconds
- [ ] All channels handle errors gracefully (no panic, no crash)

---

## Capability: data-export

### Description

Export the user's wishlist, price history, and app settings as a ZIP archive via a native save dialog. All data is local — no network involved.

### Dependencies

- `zip` crate (added to `Cargo.toml`).
- `tauri-plugin-dialog` for the native save dialog.
- `app-settings` capability for reading settings into export.
- `wishlist` capability (existing) for wishlist data.

### Functional Requirements

- REQ-DE-1: The system MUST offer a `export_data` IPC command triggered from the Settings UI.
- REQ-DE-2: The system MUST open a native save dialog (via `tauri-plugin-dialog`) filtered to `.zip` files.
- REQ-DE-3: The ZIP archive MUST contain three files:
  - `wishlist.json` — array of wishlist items with all fields
  - `price_history.json` — array of price records with SKU, source, price, date
  - `settings.json` — flat key-value map
- REQ-DE-4: All JSON files MUST be valid, human-readable (2-space indent).
- REQ-DE-5: The system MUST write the ZIP to a temporary buffer before writing to disk.
- REQ-DE-6: The system MUST handle errors: disk full, permission denied, cancelled dialog.
- REQ-DE-7: If the wishlist or price_history tables are empty, the system MUST still create the ZIP with empty arrays `[]` in those files.
- REQ-DE-8: The system MUST NOT write any files outside the selected path.

### IPC Contract

| Field | Value |
|-------|-------|
| Command | `export_data` |
| Params | `{ path: String }` — path from save dialog |
| Returns | `{ success: bool, size_bytes: u64, file_count: u32 }` |
| Error | `"dialog_cancelled"`, `"write_error: {detail}"`, `"permission_denied"` |

### Scenarios

**Scenario 1: Full export succeeds**
GIVEN the user has 5 wishlist items, 200 price records, and 3 settings
WHEN the user clicks "Export all data" and selects `/home/user/Downloads/guitarhub-backup.zip`
THEN the system creates a valid ZIP with 3 files
AND each JSON file is parseable with valid content
AND the response includes `success: true`, `size_bytes: >0`, `file_count: 3`

**Scenario 2: Export with empty wishlist**
GIVEN the user has 0 wishlist items, 50 price records, and default settings
WHEN the user exports data
THEN `wishlist.json` contains `[]`
AND the ZIP is still valid with 3 files

**Scenario 3: Save dialog cancelled**
GIVEN the user clicks "Export all data"
WHEN the save dialog is cancelled
THEN `export_data` is never called (frontend guards against empty path)
AND no ZIP is written
AND no error state persists

**Scenario 4: Disk full during write**
GIVEN the target filesystem has no free space
WHEN the system writes the ZIP buffer to disk
THEN the write fails with an OS error
AND the system returns `"write_error: No space left on device"`
AND the temp buffer is dropped (no cleanup needed — buffer was in memory)

### Acceptance Criteria

- [ ] `cargo test` passes for export service (unit test with in-memory DB)
- [ ] Exported ZIP opens with `unzip -l` and `unzip -t` without errors
- [ ] All 3 JSON files are valid (verify with `python -m json.tool`)
- [ ] Save dialog appears with `.zip` filter
- [ ] Cancelling the dialog produces no file and no error
- [ ] Export with empty data produces valid ZIP with empty arrays

---

## Capability: app-settings

### Description

Persistent key-value settings table for the local SQLite database. Stores alert channel config, notification preferences, and any future user-facing settings. Shared by `alert-delivery` and `data-export`.

### Dependencies

- Migration **005**: CREATE TABLE `settings (key TEXT PRIMARY KEY, value TEXT NOT NULL)`.
- Migration runner (must apply migrations sequentially).

### Functional Requirements

- REQ-AS-1: Migration 005 MUST create a `settings` table with `key TEXT PRIMARY KEY` and `value TEXT NOT NULL`.
- REQ-AS-2: The system MUST expose `get_setting(key: String) -> Option<String>` IPC command.
- REQ-AS-3: The system MUST expose `save_setting(key: String, value: String)` IPC command.
- REQ-AS-4: Structured values (e.g., channel config) MUST be stored as JSON strings and coerced at read time.
- REQ-AS-5: The system MUST define these default keys:
  - `alert_channel` — default `"app"`
  - `alert_config` — default `""`
- REQ-AS-6: The system MUST return an empty string for unknown keys (not an error).
- REQ-AS-7: Migration 005 MUST be idempotent — `CREATE TABLE IF NOT EXISTS`.

### IPC Contract

**get_setting**

| Field | Value |
|-------|-------|
| Command | `get_setting` |
| Params | `{ key: String }` |
| Returns | `String` (empty string if not found) |
| Error | None |

**save_setting**

| Field | Value |
|-------|-------|
| Command | `save_setting` |
| Params | `{ key: String, value: String }` |
| Returns | `()` |
| Error | `"key_required"` if key is empty |

### Scenarios

**Scenario 1: Save and retrieve a string setting**
GIVEN the database has no `alert_channel` row
WHEN the frontend calls `save_setting(key: "alert_channel", value: "ntfy")`
THEN a row is inserted into `settings`
AND the next `get_setting(key: "alert_channel")` returns `"ntfy"`

**Scenario 2: Overwrite existing setting**
GIVEN `settings` has `alert_channel = "app"`
WHEN the frontend calls `save_setting(key: "alert_channel", value: "webhook")`
THEN the existing value is updated (UPSERT)
AND `get_setting(key: "alert_channel")` returns `"webhook"`

**Scenario 3: Unknown key returns empty string**
GIVEN no row exists for key `"nonexistent"`
WHEN `get_setting(key: "nonexistent")` is called
THEN it returns `""` (empty string)
AND no error is returned

### Acceptance Criteria

- [ ] `cargo test` passes for get/set operations
- [ ] Migration 005 runs idempotently — second start does not fail
- [ ] Settings persist after app restart (verified by read-back)
- [ ] Structured JSON values survive round-trip (write JSON, read back, parse)
- [ ] Unknown keys return empty string (not error, not null)

---

## Migration Plan

### Migration 004 — Add source_id to price_history

```sql
ALTER TABLE price_history ADD COLUMN source_id TEXT NOT NULL DEFAULT '';
CREATE INDEX IF NOT EXISTS idx_price_history_sku_recorded
  ON price_history(sku, recorded_at);
```

### Migration 005 — Create settings table

```sql
CREATE TABLE IF NOT EXISTS settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
```

### Out of Scope

- Cloud sync of settings or alerts
- Shared alert relay server
- Price forecasting or ML-based predictions
- GTIN deduplication
- Multi-currency conversion
- Image format conversion (AVIF/WEBP)
- Remote notification push without Ntfy.sh
