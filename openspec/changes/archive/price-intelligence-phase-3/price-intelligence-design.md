# Design: Price Intelligence — Phase 3

> **Change**: `price-intelligence-phase-3`
> **Strict TDD**: RED → GREEN → REFACTOR
> **Pattern**: Clean Architecture — commands → services → repository (sqlite)

---

## 1. Architecture Overview

### Module Dependency Map

```
┌──────────────────────────────────────────────────────────────┐
│                        Tauri IPC Layer                       │
│                                                              │
│  get_price_history  get_price_insight  get_setting           │
│  save_setting       test_alert_channel  export_data          │
│         │                 │                  │               │
│  ┌──────┴──────┐  ┌──────┴──────┐  ┌───────┴───────┐       │
│  │ price_      │  │ settings_   │  │ export_       │       │
│  │ command     │  │ command     │  │ command       │       │
│  └──────┬──────┘  └──────┬──────┘  └───────┬───────┘       │
│         │                │                  │               │
├─────────┼────────────────┼──────────────────┼───────────────┤
│         │                │                  │               │
│  ┌──────┴──────┐  ┌──────┴──────┐  ┌───────┴───────┐       │
│  │ PriceHistory│  │ AlertService│  │ ExportService │       │
│  │ Repo        │  │ (trait)     │  │               │       │
│  └──────┬──────┘  │             │  │               │       │
│         │         │ AppNotif    │  │ zip crate     │       │
│         │         │ NtfyAlert   │  │ temp buffer   │       │
│         │         │ WebhookAlert│  │               │       │
│         │         └──────┬──────┘  └───────────────┘       │
│         │                │                                  │
│  ┌──────┴────────────────┴──────────────────┐               │
│  │           SqlitePool (shared)             │               │
│  └───────────────────────────────────────────┘               │
│                                                              │
│  ┌───────────────────┐   ┌───────────────────┐              │
│  │ price_history     │   │ settings          │              │
│  │ +source_id (004)  │   │ (005)             │              │
│  └───────────────────┘   └───────────────────┘              │
└──────────────────────────────────────────────────────────────┘
```

### Architecture Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Repository pattern | Concrete struct, no trait | Matches existing `ImageCacheRepo` — pragmatic simplicity for a single-database app |
| Alert service | Trait + 3 impls | Polymorphism needed — 3 fundamentally different delivery mechanisms |
| Export path routing | Frontend handles dialog, backend writes | Keeps IPC contract simple; backend doesn't need `AppHandle` |
| Settings access | Direct sqlx queries in command | Too simple for a service layer — pure UPSERT/SELECT |

### Shared State Changes (`AppState`)

```rust
// src-tauri/src/lib.rs — additions
pub struct AppState {
    pub pool: sqlx::SqlitePool,
    pub image_cache_service: ImageCacheService,
    pub http_client: reqwest::Client,  // NEW — shared for alerts/webhooks
}
```

`reqwest::Client` is cheap to clone (internally `Arc`-based), so it lives in `AppState` without a wrapper.

---

## 2. Module Details

### 2.1 `src-tauri/src/repository/price_history.rs` — CREATE

**Responsibility**: SQL queries against `price_history` table with outlier filtering.

```rust
pub struct PriceHistoryRepo {
    pool: SqlitePool,
}

pub struct PricePoint {
    pub source_id: String,
    pub recorded_at: i64,   // Unix epoch seconds
    pub price: f64,
}

impl PriceHistoryRepo {
    pub fn new(pool: SqlitePool) -> Self;

    /// Get price points within window, filtered by 5σ per source.
    /// If a source has < 30 points, returns unfiltered.
    pub async fn get_history(
        &self, sku: &str, window_days: u32
    ) -> Result<Vec<PricePoint>, sqlx::Error>;

    /// Compute rolling stats for price insight.
    /// Returns (min_30d, avg_90d, count_30d, current_price).
    pub async fn get_insight(
        &self, sku: &str
    ) -> Result<Option<PriceInsightRow>, sqlx::Error>;
}
```

**Key SQL — `get_history` with outlier filter**:

```sql
WITH stats AS (
    SELECT source_id,
           AVG(price) AS mean,
           SQRT(AVG(price * price) - AVG(price) * AVG(price)) AS stddev,
           COUNT(*) AS cnt
    FROM price_history
    WHERE sku = ?1 AND recorded_at >= ?2
    GROUP BY source_id
)
SELECT ph.source_id, ph.recorded_at, ph.price
FROM price_history ph
JOIN stats s ON ph.source_id = s.source_id
WHERE ph.sku = ?1
  AND ph.recorded_at >= ?2
  AND (s.cnt < 30 OR ABS(ph.price - s.mean) <= 5 * s.stddev)
ORDER BY ph.recorded_at ASC
```

Stddev via `SQRT(AVG(price*price) - AVG(price)*AVG(price))` — population stddev, which is fine for filtering. If stddev is 0 (all same price), the `ABS(price - mean) <= 0` filter passes all points, which is correct.

**Key SQL — `get_insight`**:

```sql
SELECT MIN(price) FILTER (WHERE recorded_at >= ?2) AS min_30d,
       AVG(price) FILTER (WHERE recorded_at >= ?3) AS avg_90d,
       COUNT(*)   FILTER (WHERE recorded_at >= ?2) AS cnt_30d,
       -- current_price = most recent price point
       (SELECT price FROM price_history
        WHERE sku = ?1 ORDER BY recorded_at DESC LIMIT 1) AS current_price
FROM price_history
WHERE sku = ?1
```

### 2.2 `src-tauri/src/repository/mod.rs` — MODIFY

```rust
pub mod sqlite;
pub mod price_history;  // NEW
```

### 2.3 `src-tauri/src/repository/sqlite/migrations/004_add_price_source.sql` — CREATE

See section 3.

### 2.4 `src-tauri/src/repository/sqlite/migrations/005_add_settings.sql` — CREATE

See section 3.

### 2.5 `src-tauri/src/commands/price_command.rs` — CREATE

**Responsibility**: Tauri IPC commands for price history and insights.

```rust
#[tauri::command]
pub async fn get_price_history(
    sku: String,
    window_days: Option<u32>,
    state: State<'_, AppState>,
) -> Result<Vec<PricePoint>, String>;

#[tauri::command]
pub async fn get_price_insight(
    sku: String,
    state: State<'_, AppState>,
) -> Result<Option<PriceInsight>, String>;
```

**Key decision**: `window_days` defaults to 365 if `None`. Validation: `sku` must be non-empty. Empty SKU → `Err("sku_required")`.

### 2.6 `src-tauri/src/commands/settings_command.rs` — CREATE

**Responsibility**: Tauri IPC commands for settings CRUD and alert testing.

```rust
#[tauri::command]
pub async fn get_setting(
    key: String,
    state: State<'_, AppState>,
) -> Result<String, String>;
// Returns empty string for unknown keys (not an error)

#[tauri::command]
pub async fn save_setting(
    key: String,
    value: String,
    state: State<'_, AppState>,
) -> Result<(), String>;

#[tauri::command]
pub async fn test_alert_channel(
    channel: String,    // "app" | "ntfy" | "webhook"
    config: String,     // topic (ntfy) or URL (webhook); ignored for "app"
    state: State<'_, AppState>,
) -> Result<AlertTestResult, String>;
```

### 2.7 `src-tauri/src/commands/export_command.rs` — CREATE

```rust
#[tauri::command]
pub async fn export_data(
    path: String,
    state: State<'_, AppState>,
) -> Result<ExportResult, String>;
```

### 2.8 `src-tauri/src/commands/mod.rs` — MODIFY

```rust
pub mod image_command;
pub mod price_command;    // NEW
pub mod settings_command; // NEW
pub mod export_command;   // NEW
```

### 2.9 `src-tauri/src/services/alert_service.rs` — CREATE

**Responsibility**: Trait + implementations for alert delivery.

```rust
/// Alert channel configuration read from settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    pub channel: AlertChannel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AlertChannel {
    App,
    Ntfy { topic: String },
    Webhook { url: String },
}

/// Result from a test send.
#[derive(Debug, Clone, Serialize)]
pub struct AlertTestResult {
    pub success: bool,
    pub message: String,
}

#[async_trait]
pub trait AlertDispatcher: Send + Sync {
    /// Send an alert. Returns Ok(()) on success, Err(message) on failure.
    /// MUST NOT panic. MUST log errors internally.
    async fn send(&self, title: &str, message: &str) -> Result<(), String>;

    /// Send a test notification specific to this channel type.
    async fn test(&self) -> AlertTestResult;
}

pub struct AppNotificationAlert;
pub struct NtfyAlert {
    topic: String,
    http: reqwest::Client,
}
pub struct WebhookAlert {
    url: String,
    http: reqwest::Client,
}
```

**AppNotificationAlert** — uses `tauri::api::notification::Notification`:
```rust
impl AlertDispatcher for AppNotificationAlert {
    async fn send(&self, title: &str, message: &str) -> Result<(), String> {
        // Tauri 2: Notification::new("com.guitarhub.app")
        //   .title(title).body(message).show()
        // This runs on the main thread — use tauri's async notification API
        Ok(())
    }
}
```

**NtfyAlert** — POST to `https://ntfy.sh/{topic}`:
```
Headers: Title, Message, Priority: high, Tags: guitar
```

**WebhookAlert** — POST JSON `{title, message, sku?, price?, url?}`.

**URL validation** (shared by NtfyAlert and WebhookAlert):
```rust
fn validate_webhook_url(raw: &str) -> Result<Url, String> {
    let url = Url::parse(raw).map_err(|_| "invalid_url".to_string())?;
    match url.scheme() {
        "http" | "https" => Ok(url),
        _ => Err("invalid_url: only http/https allowed".to_string()),
    }
}
```

### 2.10 `src-tauri/src/services/export_service.rs` — CREATE

**Responsibility**: Build a ZIP buffer from database contents.

```rust
pub struct ExportService {
    pool: SqlitePool,
}

#[derive(Serialize)]
pub struct ExportResult {
    pub success: bool,
    pub size_bytes: u64,
    pub file_count: u32,
}

impl ExportService {
    pub fn new(pool: SqlitePool) -> Self;

    /// Collect data from DB, build ZIP in memory, write to path.
    pub async fn export_to(&self, path: &str) -> Result<ExportResult, ExportError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("write_error: {0}")]
    Write(String),
    #[error("permission_denied: {0}")]
    Permission(String),
}
```

**ZIP structure** (all via `zip` crate, `ZipWriter`):
```
wishlist.json       — JSON array from SELECT * FROM wishlist
price_history.json  — JSON array from SELECT * FROM price_history
settings.json       — JSON object {key: value, ...} from SELECT * FROM settings
```

### 2.11 `src-tauri/src/services/mod.rs` — MODIFY

```rust
pub mod image_cache;
pub mod alert_service;  // NEW
pub mod export_service; // NEW
```

### 2.12 `src-tauri/src/main.rs` — MODIFY

```rust
fn main() {
    // ... existing setup ...

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())  // NEW
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            guitarhub_lib::commands::image_command::get_product_image,
            guitarhub_lib::commands::price_command::get_price_history,   // NEW
            guitarhub_lib::commands::price_command::get_price_insight,   // NEW
            guitarhub_lib::commands::settings_command::get_setting,      // NEW
            guitarhub_lib::commands::settings_command::save_setting,     // NEW
            guitarhub_lib::commands::settings_command::test_alert_channel, // NEW
            guitarhub_lib::commands::export_command::export_data,        // NEW
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 2.13 `src-tauri/Cargo.toml` — MODIFY

```toml
[dependencies]
# ... existing ...
zip = "2"              # NEW — pure Rust ZIP
tauri-plugin-dialog = "2"  # NEW — native save dialog

[dev-dependencies]
# httpmock already exists
```

### 2.14 `src-tauri/tauri.conf.json` — MODIFY

```json
{
  "app": {
    // ... existing ...
  },
  "plugins": {
    "dialog": {
      "all": true
    }
  }
}
```

### 2.15 Svelte Components

See section 5.

---

## 3. Data Model Changes

### 3.1 Migration 004 — `004_add_price_source.sql`

```sql
-- Migration 004: Add source_id to price_history
-- Enables multi-source per-SKU price chart lines.

ALTER TABLE price_history ADD COLUMN source_id TEXT NOT NULL DEFAULT '';

CREATE INDEX IF NOT EXISTS idx_price_history_sku_recorded
  ON price_history(sku, recorded_at);
```

**Backward compatibility**: Existing rows get `source_id = ''`. The `DEFAULT ''` means inserts without a source still work. Chart rendering handles empty `source_id` gracefully (single line, labeled "Unknown").

### 3.2 Migration 005 — `005_add_settings.sql`

```sql
-- Migration 005: Create settings table
-- Key-value store for app configuration (alert channel, export prefs, etc.)

CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

### 3.3 Migration Strategy

- **Forward-only, additive**: Both migrations are purely additive — `ALTER TABLE ADD COLUMN` and `CREATE TABLE`. Full rollback per the proposal (revert merge commit).
- **Idempotent**: `CREATE INDEX IF NOT EXISTS`, `CREATE TABLE IF NOT EXISTS`.
- **Sequential**: Migration 004 must run before 005 (numerical ordering). The existing `MigrationRunner` enforces gap detection.
- **Test isolation**: Each test creates its own in-memory DB and applies only the migrations it needs.

---

## 4. IPC Contracts

### 4.1 `get_price_history`

```rust
#[tauri::command]
pub async fn get_price_history(
    sku: String,
    window_days: Option<u32>,  // default 365
    state: State<'_, AppState>,
) -> Result<Vec<PricePoint>, String>;

// Returns:
[
  { "source_id": "reverb", "recorded_at": 1717200000, "price": 299.99 },
  { "source_id": "reverb", "recorded_at": 1717286400, "price": 289.99 },
  ...
]

// Error strings: "sku_required", "no_data"
```

### 4.2 `get_price_insight`

```rust
#[tauri::command]
pub async fn get_price_insight(
    sku: String,
    state: State<'_, AppState>,
) -> Result<Option<PriceInsight>, String>;

// Returns:
{
  "level": "green",       // "green" | "amber" | "hidden"
  "pct": 2.04,            // percentage above min_30d or avg_90d
  "current_price": 100.0,
  "min_30d": 98.0,
  "avg_90d": 85.0
}

// Returns None (Ok(None)) when no price data at all for the SKU.
// Error strings: "sku_required"
```

### 4.3 `get_setting`

```rust
#[tauri::command]
pub async fn get_setting(
    key: String,
    state: State<'_, AppState>,
) -> Result<String, String>;

// Returns empty string for unknown keys (NOT an error).
// Never errors.
```

### 4.4 `save_setting`

```rust
#[tauri::command]
pub async fn save_setting(
    key: String,
    value: String,
    state: State<'_, AppState>,
) -> Result<(), String>;

// Error strings: "key_required"
```

### 4.5 `test_alert_channel`

```rust
#[tauri::command]
pub async fn test_alert_channel(
    channel: String,   // "app" | "ntfy" | "webhook"
    config: String,    // for ntfy: topic name; for webhook: URL; for app: ignored
    state: State<'_, AppState>,
) -> Result<AlertTestResult, String>;

// Returns:
{ "success": true, "message": "Notification sent" }

// Error strings: "invalid_channel", "invalid_url", timeout after 5s internally
```

### 4.6 `export_data`

```rust
#[tauri::command]
pub async fn export_data(
    path: String,   // absolute path from frontend save dialog
    state: State<'_, AppState>,
) -> Result<ExportResult, String>;

// Returns:
{ "success": true, "size_bytes": 12345, "file_count": 3 }

// Error strings: "write_error: {detail}", "permission_denied", "dialog_cancelled"
```

---

## 5. Component Tree (Svelte 5)

### 5.1 `src/lib/components/ProductCard.svelte` — MODIFY

**Changes**: Add price badge after the price paragraph.

```svelte
<script>
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  let { product } = $props();
  let imageData = $state('');
  let imageError = $state(false);
  let priceInsight = $state(null);        // NEW

  onMount(async () => {
    try {
      imageData = await invoke('get_product_image', { imageUrl: product.image_url });
    } catch (e) {
      imageError = true;
    }
    // NEW: fetch price insight after product loads (avoid cascading)
    try {
      priceInsight = await invoke('get_price_insight', { sku: product.sku });
    } catch (e) {
      // silent fail — badge is optional
    }
  });
</script>

<div class="product-card">
  <!-- existing image block -->
  <div class="product-info">
    <h3>{product.name}</h3>
    {#if product.brand}<p class="brand">{product.brand}</p>{/if}
    {#if product.price}
      <p class="price">
        {product.price} {product.currency ?? ''}
        {#if priceInsight && priceInsight.level !== 'hidden'}
          <PriceBadge level={priceInsight.level} pct={priceInsight.pct} />
        {/if}
      </p>
    {/if}
  </div>
</div>
```

### 5.2 `src/lib/components/PriceBadge.svelte` — CREATE

```svelte
<script>
  let { level = 'green', pct = 0 } = $props();
</script>

{#if level === 'green'}
  <span class="badge badge--green" title="Current price is near the 30-day low">
    ✓ Good price
  </span>
{:else if level === 'amber'}
  <span class="badge badge--amber" title="Current price is above the 90-day average">
    ↑ Above average
  </span>
{/if}

<style>
  .badge { /* inline-block, small font, rounded, 4px padding */ }
  .badge--green { background: #d4edda; color: #155724; }
  .badge--amber  { background: #fff3cd; color: #856404; }
</style>
```

**Props**: `level: string`, `pct: number`. No events. No slots.

### 5.3 `src/lib/components/PriceChart.svelte` — CREATE

```svelte
<script>
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  let { sku, windowDays = 365 } = $props();
  let points = $state([]);
  let loading = $state(true);
  let error = $state(null);

  onMount(async () => {
    try {
      points = await invoke('get_price_history', { sku, windowDays });
    } catch (e) {
      error = e;
    } finally {
      loading = false;
    }
  });

  // Pure SVG rendering — compute viewBox from data extents
  $derived svgData = computeSvgData(points);
</script>

{#if loading}
  <div aria-busy="true">Loading chart...</div>
{:else if error}
  <div class="empty-state" role="status">{error}</div>
{:else}
  <div class="chart-container" role="img" aria-label="Price history chart for {sku}">
    <svg viewBox="0 0 {svgData.width} {svgData.height}" preserveAspectRatio="xMidYMid meet">
      <title>Price history for {sku}</title>
      {#each svgData.sources as source}
        <polyline points={source.points} fill="none" stroke={source.color}
                  stroke-width="2" vector-effect="non-scaling-stroke" />
      {/each}
    </svg>
    {#each svgData.insufficientSources as source}
      <p class="note">Insufficient data for {source}</p>
    {/each}
  </div>
{/if}
```

**Props**: `sku: string`, `windowDays?: number`. No events. No slots.

**`computeSvgData` logic**:
- Group `PricePoint[]` by `source_id`
- Compute global min/max of price and time for `viewBox`
- If a source has < 30 points, render it but flag as "Insufficient data"
- Downsample: if a source has > 500 points, sample evenly to 500 (keeps SVG responsive)
- Color palette: 5 distinct colors hardcoded, cycle if >5 sources
- `viewBox` = `0 0 {timeRange} {priceRange + padding}` — uses pixel-like unit mapping

### 5.4 `src/lib/components/Settings.svelte` — CREATE

```
┌─────────────────────────────────────────┐
│ ⚙ Settings                              │
│                                         │
│ ─── Alert Channel ───────────────────── │
│                                         │
│ ○ App notifications (default)           │
│ ○ Ntfy.sh             topic: [______]  │
│ ○ Webhook POST        URL:   [______]  │
│                                         │
│ [Test Notification]  ✓ Sent!            │
│                                         │
│ ─── Data Export ─────────────────────── │
│                                         │
│ [Export All Data]  Last export: 2.3 KB  │
│                                         │
└─────────────────────────────────────────┘
```

**State**: `alertChannel` (radio group), `alertConfig` (text input), `testResult`, `exportResult`.

**Key behavior**:
- `test_alert_channel` called with current channel + config on "Test" click
- `@tauri-apps/plugin-dialog` `save()` opens native dialog, then passes path to `export_data`
- Settings are reactive — `save_setting` on change, `get_setting` on mount

---

## 6. Security Analysis

### 6.1 Unsafe Blocks

**Goal: zero new `unsafe` blocks.** All new code uses safe Rust:
- `zip` crate is pure Rust, no `unsafe` in its public API
- `reqwest::Client` reused from existing code
- `sqlx` queries are safe (no raw `format!` in SQL strings)
- All url parsing uses the `url` crate

### 6.2 Input Validation

| Input | Validation | Location |
|-------|-----------|----------|
| SKU | Non-empty string check | `price_command.rs` |
| Webhook URL | `Url::parse`, scheme `http`/`https` only, no IP literals | `alert_service.rs` |
| Ntfy topic | Non-empty, alphanumeric + hyphens | `alert_service.rs` |
| Export path | Passed from save dialog (OS-validated) | `export_command.rs` |
| Setting key | Non-empty | `settings_command.rs` |

**URL validation** reuses the same `url` crate pattern from `image_command.rs` — no IP literals, no non-http schemes. This prevents SSRF via webhook URLs.

### 6.3 Data Sensitivity

| Data | Storage | Risk | Mitigation |
|------|---------|------|------------|
| Ntfy.sh topic | Plaintext in `settings` table | Low — topic is public | Document as conscious tradeoff. No cloud sync. |
| Webhook URL | Plaintext in `settings` table | Low-Medium — could contain API keys in URL | No cloud sync, local DB only. User controls their own machine. |
| Price data | Local SQLite | None | Already local-only. Export is user-initiated. |

**Tradeoff ADR-002**: Plaintext webhook URLs are a local-only risk. Encrypting at rest adds complexity (key management, platform-specific keystores) for an app that never syncs data.

---

## 7. Architecture Decision Records

### ADR-001: Pure SVG Over Chart Library

**Context**: Price chart rendering on ProductDetail.

**Options**:
| Option | Tradeoff |
|--------|----------|
| Chart.js / D3.js | + Rich interactivity — Heavy dependency (200KB+), npm audit surface |
| Canvas-based | + Performance for dense data — Accessibility harder, no vector scaling |
| **Pure SVG `<polyline>`** | + Zero deps, native responsiveness, accessible, Tauri CSP friendly — Manual axis/scaling logic |

**Decision**: Pure SVG `<polyline>` with `viewBox`. The chart data ≤ 500 points per source. SVG `role="img"` + `<title>` provides accessibility. Manual viewBox math is ~50 lines of code vs 200KB of library.

### ADR-002: Plaintext Webhook URL Storage

**Context**: Alert channel configuration stored in local SQLite.

**Options**:
| Option | Tradeoff |
|--------|----------|
| **Plaintext in SQLite** | + Simple, no key management — Exposed if DB file is stolen |
| macOS Keychain / Linux Secret Service | + Encrypted — Platform-specific, unavailable on F-Droid targets |
| SQLCipher / encrypted SQLite | + Encrypted rows — Heavy dep, audit complexity, licensing concerns |

**Decision**: Plaintext. The app is offline-first, F-Droid compatible, and never syncs data. The SQLite file is on the user's own filesystem with OS-level permissions. Encrypting webhook URLs adds significant complexity for a local-only threat model.

### ADR-003: Separate Migrations Per Capability

**Context**: Two schema changes (migration 004 and 005).

**Options**:
| Option | Tradeoff |
|--------|----------|
| **Single migration** | + Fewer files — Tight coupling, harder to revert per-capability |
| **Separate migrations** | + Capability-isolated, per-feature rollback — More migration files |

**Decision**: Two separate migrations (004, 005). The existing `MigrationRunner` supports sequential numbering. Decoupling means:
- Alert delivery can be rolled back without touching price history schema
- Each migration SQL file is small and focused

### ADR-004: No Caching Layer for Price Data

**Context**: Price history queries and insight queries hit SQLite directly.

**Options**:
| Option | Tradeoff |
|--------|----------|
| **No cache** | + Zero complexity, always consistent, <50ms queries — Re-query on every chart render |
| In-memory cache (DashMap) | + Faster re-renders — Staleness, invalidation complexity, memory cost |
| SQLite view / materialized view | + Database-native — SQLite doesn't support materialized views |

**Decision**: No caching. Price queries are fast (indexed on `(sku, recorded_at)`), the data is small (typically <1000 rows per SKU), and re-rendering the chart requires fresh data. Benchmark target: queries complete in <50ms on 10k+ rows. If that fails, add caching later.

---

## 8. Test Strategy (TDD)

### 8.1 Repository: `price_history.rs`

| Test | Mock | Assert |
|------|------|--------|
| `get_history returns points within window` | In-memory SQLite with seeded data | Correct count, ordered ASC |
| `get_history filters 5σ outliers per source` | 2 sources, one with outlier | Outlier excluded, other source intact |
| `get_history returns all when < 30 points` | Source with 25 points | All 25 returned (no filter) |
| `get_history no data returns empty vec` | Empty table | Empty vec, no error |
| `get_insight returns green level` | Seed: current_price close to 30d min | `level: "green"` |
| `get_insight returns amber level` | Seed: current_price above 90d avg | `level: "amber"` |
| `get_insight returns hidden when <30 rows` | Seed: only 20 rows in 30d window | `level: "hidden"` |
| `get_insight no data returns None` | Empty table | `Ok(None)` |

**How to run**: `cargo test --package guitarhub -- repository::price_history`

### 8.2 Services: `alert_service.rs`

| Test | Mock | Assert |
|------|------|--------|
| `AppNotificationAlert::send returns Ok` | None (no-op impl) | `Ok(())` |
| `NtfyAlert::send POSTs correct URL` | `httpmock` server | Correct URL, headers, retry-once logic |
| `WebhookAlert::send POSTs valid JSON` | `httpmock` server | Valid JSON body, correct Content-Type |
| `WebhookAlert::send handles 4xx/5xx` | `httpmock` returns 500 | `Err` with descriptive message |
| `WebhookAlert::send handles network failure` | No server | `Err`, no panic |
| `validate_webhook_url rejects empty string` | None | `Err("invalid_url")` |
| `validate_webhook_url rejects non-http` | `"ftp://bad"` | `Err("invalid_url")` |
| `validate_webhook_url rejects IP literal` | `"http://10.0.0.1/hook"` | `Err` with SSRF message |
| `validate_webhook_url accepts valid URL` | `"https://hooks.example.com/alert"` | `Ok(url)` |

**How to run**: `cargo test --package guitarhub -- services::alert_service`

### 8.3 Services: `export_service.rs`

| Test | Mock | Assert |
|------|------|--------|
| `export produces valid ZIP` | In-memory DB seeded with data | ZIP parses, 3 files present |
| `export with empty tables` | Empty in-memory DB | ZIP valid, 3 files with `[]` arrays |
| `export writes to temp path` | Real temp dir | File exists, non-zero size |
| `export disk full error` | Unwritable path | `Err(ExportError::Write(...))` |

**How to run**: `cargo test --package guitarhub -- services::export_service`

### 8.4 Commands: IPC Integration

| Test | Scenario | Assert |
|------|----------|--------|
| `get_price_history with empty sku` | `sku: ""` | `Err("sku_required")` |
| `get_setting unknown key` | `key: "nonexistent"` | `Ok("")` |
| `save_setting then get_setting` | Round-trip | `Ok("value")` |
| `test_alert_channel invalid channel` | `channel: "slack"` | `Err("invalid_channel")` |
| `export_data with empty path` | `path: ""` | `Err("write_error")` |

**How to run**: `cargo test --package guitarhub -- commands::`

### 8.5 Frontend (Svelte) — Manual / E2E

| Test | Method |
|------|--------|
| PriceChart renders SVG with data | Manual verification + DevTools |
| PriceChart shows empty state | No-data scenario |
| PriceBadge green/amber/hidden | Inspect DOM class |
| Settings test button feedback | Click → see success/failure toast |
| Export triggers save dialog | Click → see native dialog |

**E2E**: `tauri-driver` + WebDriver (not in scope for this phase — marked as manual).

### 8.6 Running All Tests

```bash
# All backend tests
cargo test --package guitarhub

# Specific module
cargo test --package guitarhub -- repository::price_history
cargo test --package guitarhub -- services::alert_service
cargo test --package guitarhub -- services::export_service
cargo test --package guitarhub -- commands::

# Lint
cargo clippy --package guitarhub -- -D warnings
```

---

## Summary

| Dimension | Count |
|-----------|-------|
| **New Rust files** | 8 (4 commands, 2 services, 1 repo, 0 shared) |
| **Modified Rust files** | 5 (mod.rs ×3, main.rs, lib.rs) |
| **Migration files** | 2 (004, 005) |
| **New Svelte components** | 3 (PriceChart, PriceBadge, Settings) |
| **Modified Svelte components** | 1 (ProductCard) |
| **New dependencies** | 2 (zip, tauri-plugin-dialog) |
| **New unreachable blocks** | 0 |
| **ADRs** | 4 |
| **Test modules** | 4 (repo, 2 services, 1 command) |
