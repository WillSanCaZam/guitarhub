# Delta for collection-input

> **Change**: mvp-fixes — Input fixes D6 and D7

## ADDED Requirements

### Requirement: addToCollection MUST accept condition from caller

The `addToCollection` store function MUST accept a `condition` parameter and pass it through to the `add_to_collection` Tauri command. The `condition` field in `CollectionItemInput` MUST NOT be hardcoded to `'good'`.

#### Scenario: Caller passes condition

- GIVEN `addToCollection` is called with `{ sku, name, brand, price, condition: 'excellent' }`
- WHEN the Tauri command is invoked
- THEN `input.condition` is `'excellent'`
- AND the saved row has `condition = 'excellent'`

#### Scenario: Caller omits condition

- GIVEN `addToCollection` is called without a `condition` field
- WHEN the Tauri command is invoked
- THEN `input.condition` defaults to `'good'` as a sensible fallback

### Requirement: Currency fallback MUST use nullish coalescing

The `purchase_currency` field in `addToCollection` MUST use `??` (nullish coalescing) instead of `||` (logical OR) for the fallback to `'USD'`. This ensures that an empty string `""` — a valid currency value in some contexts — is not treated as falsy and replaced with `'USD'`.

#### Scenario: Currency is a valid non-USD value

- GIVEN `addToCollection` is called with `{ ..., currency: 'EUR' }`
- WHEN the Tauri command is invoked
- THEN `input.purchase_currency` is `'EUR'`

#### Scenario: Currency is empty string (preserved)

- GIVEN `addToCollection` is called with `{ ..., currency: '' }`
- WHEN the input is constructed
- THEN `purchase_currency` is `'' ?? 'USD'` = `''` (empty string preserved, not replaced with `'USD'`)

#### Scenario: Currency is undefined (fallback)

- GIVEN `addToCollection` is called without a `currency` field
- WHEN the input is constructed
- THEN `purchase_currency` is `undefined ?? 'USD'` = `'USD'`