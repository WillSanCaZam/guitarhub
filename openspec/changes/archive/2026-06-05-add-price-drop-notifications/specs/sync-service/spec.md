# Delta for sync-service

> **Change**: `add-price-drop-notifications`
> **Phase**: sdd-spec
> **Type**: MODIFIED capability
> **Mode**: hybrid (engram + openspec)

This delta adds price-drop detection and dispatch to the existing sync flow. It does NOT modify or remove any existing requirement from `openspec/specs/sync-service/spec.md`.

## ADDED Requirements

### Requirement: upsert_products MUST record price history

`CatalogSyncService::upsert_products` MUST, for every successfully upserted product, write a row to `price_history` with `(sku, price, recorded_at = now)`. The write MUST occur after the `products_meta` INSERT and MUST NOT alter the return type of the helper called by the state machine.

#### Scenario: First sync writes one history row per product

- GIVEN an empty `price_history` table
- WHEN `upsert_products` is called with 3 products
- THEN `price_history` MUST contain exactly 3 rows with `recorded_at = now`

#### Scenario: Second sync writes a new row even when price is unchanged

- GIVEN one prior `price_history` row for SKU `X` at price `100.0`
- WHEN a second sync ingests SKU `X` at price `100.0`
- THEN a second row for `X` at `100.0` MUST be appended (history, not idempotent)

### Requirement: SyncResult MUST carry detected drops

The `SyncResult` struct MUST add a `drops: Vec<PriceDrop>` field. The field MUST be empty when no drop fires and MUST be populated by `upsert_products` from the in-pass drop evaluation.

#### Scenario: Drops surfaced in SyncResult

- GIVEN 2 products where 1 drops 15% and 1 is unchanged
- WHEN sync completes
- THEN `SyncResult.drops` MUST contain exactly 1 `PriceDrop` for the dropped SKU

#### Scenario: No drops on first sync

- GIVEN an empty `price_history` (first-ever sync)
- WHEN sync completes
- THEN `SyncResult.drops` MUST be empty (first-observation suppression)

### Requirement: sync_command MUST dispatch detected drops

`sync_command` MUST, after `sync_catalog` returns, read `settings.alert_channel`, build the corresponding `AlertDispatcher`, and invoke `dispatcher.send(&drop)` for each entry in `SyncResult.drops`. The channel-to-dispatcher mapping MUST support at least `ntfy`, `webhook`, and `app` (stub).

#### Scenario: Ntfy channel uses NtfyAlert

- GIVEN `settings.alert_channel = "ntfy"` and a drop with `channel = "ntfy"`
- WHEN `sync_command` runs
- THEN `NtfyAlert::send` MUST be invoked once with the drop payload

#### Scenario: Webhook channel uses WebhookAlert

- GIVEN `settings.alert_channel = "webhook"`
- WHEN `sync_command` runs
- THEN `WebhookAlert::send` MUST be invoked once

### Requirement: Dispatch failures MUST NOT block sync

`sync_command` MUST treat any `Err` returned by `dispatcher.send` as non-fatal: log the error and continue to the next drop. The function MUST still return `Ok(SyncResult)` to the frontend with the original `drops` count.

#### Scenario: One failure does not abort the batch

- GIVEN 3 drops and the dispatcher returns `Err` for drop 2 only
- WHEN `sync_command` runs
- THEN drops 1 and 3 MUST still be attempted and `SyncResult` MUST still be returned to the caller

### Requirement: Frontend toast reports drops and sent counts

After a successful `sync_catalog` invocation, the frontend MUST display a toast with the text `"X price drops, Y sent"` where `X = SyncResult.drops.length` and `Y = count of successful dispatcher.send calls`.

#### Scenario: All dispatches succeed

- GIVEN `drops.length = 3` and all 3 dispatches return `Ok`
- WHEN the frontend receives the result
- THEN the toast MUST read `"3 price drops, 3 sent"`

#### Scenario: Partial failure

- GIVEN `drops.length = 3` and 1 dispatch returns `Err`
- WHEN the frontend receives the result
- THEN the toast MUST read `"3 price drops, 2 sent"`

## MODIFIED Requirements

_None — no existing requirement from `openspec/specs/sync-service/spec.md` is altered._

## REMOVED Requirements

_None._
