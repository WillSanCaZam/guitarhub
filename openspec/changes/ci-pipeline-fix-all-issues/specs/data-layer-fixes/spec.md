# Delta for data-layer-fixes

> **New domain** — Fix `specs_json` data loss in batch upsert and migration chain completeness.

## Purpose

Prevent silent data loss during product sync by including the `specs_json` column in the batch upsert INSERT, and ensure the export migration chain covers all migrations including 009.

## Requirements

### Requirement: batch_upsert_products MUST include specs_json column

The `batch_upsert_products` SQL INSERT in `product.rs` MUST include the `specs_json` column. When a product has `specs_json` data, it MUST be persisted to the database. When `specs_json` is absent or null, the column MUST default to `'{}'`.

#### Scenario: Product with specs_json is persisted

- GIVEN a product with `specs_json = "{\"finish\":\"sunburst\"}"` in the catalog
- WHEN `batch_upsert_products` runs
- THEN the row in `products_meta` has `specs_json = "{\"finish\":\"sunburst\"}"`

#### Scenario: Product without specs_json gets default

- GIVEN a product with no `specs_json` field in the catalog
- WHEN `batch_upsert_products` runs
- THEN the row in `products_meta` has `specs_json = '{}'`

#### Scenario: specs_json survives sync round-trip

- GIVEN a product with `specs_json` stored in the database
- WHEN the product is read back via SELECT
- THEN the `specs_json` value matches what was inserted

### Requirement: Export migration chain MUST include migration 009

The `apply_full_migration_chain` function in `export_command.rs` MUST execute migration 009 (the latest schema addition) in sequence. Skipping migration 009 causes export failures for databases that need the column or table it adds.

#### Scenario: Fresh database gets all migrations

- GIVEN a fresh database with no tables
- WHEN `apply_full_migration_chain` runs
- THEN all migrations 001 through 009 are applied in order
- AND the database schema includes the 009 additions

#### Scenario: Existing database at migration 008 gets 009

- GIVEN a database with migrations 001–008 applied
- WHEN `apply_full_migration_chain` runs
- THEN only migration 009 is applied (idempotent)
- AND the database schema is complete
