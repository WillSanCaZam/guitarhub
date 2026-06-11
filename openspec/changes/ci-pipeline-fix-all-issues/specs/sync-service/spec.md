# Delta for sync-service

> **Change**: ci-pipeline-fix-all-issues — Fix specs_json data loss

## MODIFIED Requirements

### Requirement: upsert_products MUST record price history

`CatalogSyncService::upsert_products` MUST, for every successfully upserted product, write a row to `price_history` with `(sku, price, recorded_at = now)`. The write MUST occur after the `products_meta` INSERT and MUST NOT alter the return type of the helper called by the state machine.

The batch INSERT SQL in `product.rs` (`batch_upsert_products`) MUST include the `specs_json` column so that product specification data is persisted during sync. When `specs_json` is absent or null, the column MUST default to `'{}'`.

(Previously: `batch_upsert_products` INSERT omitted `specs_json`, causing silent data loss on every sync — the column existed in the schema but was never populated by the upsert.)

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| First sync writes history | `price_history` is empty | `upsert_products` with 3 products | 3 rows with `recorded_at = now` |
| Second sync appends | One row for SKU `X` at `100.0` | Second sync ingests `X` at `100.0` | Second row appended |
| specs_json persisted | Product has `specs_json = "{\"finish\":\"sunburst\"}"` | `batch_upsert_products` runs | Row has `specs_json = "{\"finish\":\"sunburst\"}"` |
| specs_json defaults | Product has no `specs_json` field | `batch_upsert_products` runs | Row has `specs_json = '{}'` |
| specs_json round-trip | Product with `specs_json` in DB | SELECT reads row back | Value matches what was inserted |
