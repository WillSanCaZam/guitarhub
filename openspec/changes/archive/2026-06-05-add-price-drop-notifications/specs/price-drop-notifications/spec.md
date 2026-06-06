# Delta for price-drop-notifications

> **Change**: `add-price-drop-notifications`
> **Phase**: sdd-spec
> **Type**: NEW capability (no prior spec exists at `openspec/specs/price-drop-notifications/`)
> **Mode**: hybrid (engram + openspec)

This is a full specification because the capability has no prior behavior.

# Price Drop Notifications Specification

## Purpose

Detect significant price drops during catalog sync, suppress noise via a three-layer anti-spam gate, and dispatch a notification through the user's configured `AlertDispatcher` channel. The user sees a native notification the moment a tracked product drops ≥10% **or** ≥$50, and a post-sync toast reports "X drops, Y sent".

## Requirements

### Requirement: Pure drop detection function

The system MUST expose `is_price_drop(new_price: Option<f64>, previous_price: Option<f64>, thresholds: &Thresholds) -> Option<PriceDrop>` in `services/price_drop.rs` as a pure function with no I/O. The returned `PriceDrop` MUST carry `sku`, `previous_price`, `new_price`, `channel`, and a `DropReason` (Relative / Absolute).

Thresholds MUST be declared as `pub const` in the module: `RELATIVE_DROP_PCT = 0.10`, `ABSOLUTE_DROP_USD = 50.0`, `COOLDOWN_SECONDS = 86_400`. The function MUST be unit-testable without a database or clock.

#### Scenario: Significant drop fires

- GIVEN `previous = 1000.0`, `new = 800.0` (20% / $200)
- WHEN `is_price_drop` is called with defaults
- THEN it MUST return `Some(PriceDrop { reason: Relative, .. })`

#### Scenario: Small drop is suppressed

- GIVEN `previous = 100.0`, `new = 97.0` (3% / $3)
- WHEN `is_price_drop` is called
- THEN it MUST return `None`

#### Scenario: Boundary — exactly 10% fires

- GIVEN `previous = 100.0`, `new = 90.0`
- WHEN `is_price_drop` is called
- THEN it MUST return `Some(_)` (inclusive boundary)

#### Scenario: Boundary — exactly $50 fires

- GIVEN `previous = 200.0`, `new = 150.0` (25% / $50)
- WHEN `is_price_drop` is called
- THEN it MUST return `Some(_)` (inclusive boundary)

#### Scenario: Price increase is not a drop

- GIVEN `previous = 100.0`, `new = 120.0`
- WHEN `is_price_drop` is called
- THEN it MUST return `None`

#### Scenario: First observation suppressed

- GIVEN `previous = None` (no baseline in `price_history`)
- WHEN `is_price_drop` is called
- THEN it MUST return `None` (cannot compare without a baseline)

### Requirement: Cooldown table persists last-dispatch state

Migration 007 MUST create a `price_drop_notifications` table with columns `sku TEXT PRIMARY KEY`, `last_notified INTEGER NOT NULL` (unix seconds), `last_price REAL NOT NULL`, `channel TEXT NOT NULL`. A repository MUST provide `get_last_notified(sku) -> Option<i64>` and `upsert(sku, ts, price, channel) -> Result<(), AppError>`.

#### Scenario: Cooldown row created on first dispatch

- GIVEN no row exists for SKU `ABC-123`
- WHEN `upsert("ABC-123", now, 800.0, "ntfy")` runs
- THEN the table MUST contain one row with the given values

### Requirement: 24-hour cooldown enforced per SKU

The dispatch loop MUST skip a drop when `now - last_notified ≤ 86_400` seconds for that SKU. A SKU with no row (or `NULL` `last_notified`) MUST NOT be in cooldown.

#### Scenario: Within cooldown — skip

- GIVEN `last_notified = now - 12h` for SKU `X`
- WHEN the drop is evaluated
- THEN the dispatcher MUST NOT be called and no cooldown row is mutated

#### Scenario: Past cooldown — fire

- GIVEN `last_notified = now - 25h` for SKU `X`
- WHEN the drop is evaluated
- THEN the dispatcher MUST be invoked exactly once

#### Scenario: Successful dispatch updates cooldown

- GIVEN a drop fires for SKU `X` at time `t`
- WHEN the dispatcher returns `Ok(())`
- THEN the cooldown row MUST be `(X, t, new_price, channel)` after the call

#### Scenario: Failed dispatch leaves cooldown unchanged

- GIVEN a drop fires for SKU `X` and the dispatcher returns `Err(_)`
- WHEN the dispatch loop continues
- THEN the cooldown row MUST NOT be mutated so the next sync retries

## Out of Scope

- User-tunable threshold UI (constants only this iteration).
- Notifications history/log view.
- Background scheduler (sync is on-demand only).
- Per-SKU opt-out column.
- Including drops in the export ZIP.
