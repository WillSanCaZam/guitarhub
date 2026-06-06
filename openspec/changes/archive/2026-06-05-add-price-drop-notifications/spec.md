# Specs for add-price-drop-notifications

> **Change**: `add-price-drop-notifications`
> **Phase**: sdd-spec
> **Mode**: hybrid (engram + openspec)

## Index of Delta Specs

| Capability | Type | Path | Requirements | Scenarios |
|---|---|---|---|---|
| `price-drop-notifications` | NEW (full) | `specs/price-drop-notifications/spec.md` | 3 added | 11 |
| `sync-service` | MODIFIED (delta) | `specs/sync-service/spec.md` | 5 added, 0 modified, 0 removed | 8 |

Total: 8 requirements, 19 scenarios across 2 specs.

## Coverage Map

| Layer | Status |
|---|---|
| Happy path (significant drop fires) | covered — PDN S1 |
| Edge boundaries (exact 10%, exact $50) | covered — PDN S3, S4 |
| Negative cases (small drop, increase, first-obs) | covered — PDN S2, S5, S6 |
| Cooldown gate | covered — PDN S7, S8 |
| Cooldown mutation on success/failure | covered — PDN S9, S10 |
| Sync wiring (price_history, SyncResult.drops) | covered — SYNC req 1, 2 |
| Dispatcher integration (ntfy / webhook / app) | covered — SYNC req 3 |
| Failure isolation | covered — SYNC req 4 |
| Frontend toast | covered — SYNC req 5 |

## Out of Scope (explicit)

- User-tunable threshold UI
- Notifications history/log view
- Background scheduler
- Per-SKU opt-out
- Drops in export ZIP
