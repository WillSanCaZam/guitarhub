# Capability: product-detail

> **Status**: New capability  
> **Change**: product-display-pipeline

## Purpose

Provide a Tauri IPC command (`get_product_detail`) that returns a single product by SKU from the SQLite catalog, resolving the full `RawProduct` with parsed `specs_json` for frontend display.

## Requirements

### Requirement: get_product_detail MUST return a product by its SKU

`get_product_detail(sku: String) -> Result<RawProduct, AppError>` — `SELECT * FROM products_meta WHERE sku = ?`.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Valid SKU | Product with `sku = "FENDER-STRAT-001"` exists | invoke with `sku = "FENDER-STRAT-001"` | Returns the full `RawProduct` |
| Invalid SKU | No product with that SKU | invoke with `sku = "NONEXISTENT"` | Returns `AppError::NotFound` |
| Case-insensitive | Product exists with `sku = "FENDER-001"` | invoke with `sku = "fender-001"` | Match found, product returned |
| Empty SKU | Empty string passed | invoke with `sku = ""` | Returns `AppError::InvalidInput` |
| Inactive product | Product exists with `is_active = 0` | invoke by SKU | Returns `AppError::NotFound` |

### Requirement: specs_json MUST be returned as raw string; frontend parses it

The command MUST return `specs_json` as a raw JSON string (the stored DB value). The frontend SHOULD parse it via `JSON.parse()` with a try/catch fallback to `{}`. This avoids coupling the backend to a specific JSON structure and allows flexible frontend rendering.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Valid JSON specs | Product has `specs_json = "{\"body_wood\":\"alder\",\"pickups\":\"SSS\"}"` | invoke | Response includes `specs_json` as a raw JSON string |
| Empty specs | Product has `specs_json = ""` | invoke | `specs_json` returns `""` (empty string) |
| Malformed JSON stored | Product has `specs_json = "not json"` | invoke | Returns the raw string `"not json"` — frontend falls back to `{}` |

### Requirement: Product detail page MUST render full product info

The `/product/[sku]` page MUST display the product name, brand, model, price, condition, category, URL, image, and parsed specs when `get_product_detail` succeeds.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Product found | Valid SKU, response has `RawProduct` | Detail page mounts | `ProductDetail` renders with all fields: name, brand, price, specs grid |
| Not found | Backend returns `AppError::NotFound` | Detail page loads | "Product not found" message with "← Back to Home" link |
| Loading state | IPC in flight | Page mounts | `SkeletonLoader variant="detail"` shown |
| Error state | Backend error (DB failure) | invoke rejects | Error banner: "Failed to load product: {error}" with back link |
| Page title | Product loaded | `<svelte:head>` renders | Title set to `"{product.name} — GuitarHub"` |
