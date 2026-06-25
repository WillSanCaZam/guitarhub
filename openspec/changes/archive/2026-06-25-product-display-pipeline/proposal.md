# Proposal: Product Display Pipeline

## Intent

4 Tauri commands are missing and SearchPanel is unmounted, so the frontend shows empty/error where real product data should render. Bridge the gap from sync'd SQLite catalog to working UI.

## Scope

### In Scope
- 4 Tauri commands: `get_featured_products`, `get_price_drops`, `get_new_arrivals`, `get_product_detail`
- `ProductQueryService` with read methods on `products_meta` (+ `price_history` join for drops)
- Register all 4 in `main.rs`
- Wire home page sections (already call commands with `.catch(() => [])` — just need commands)
- Wire `/product/[sku]` detail page (already invokes `get_product_detail`)
- Create `/catalog` route mounting SearchPanel with FTS5 search + filters

### Out of Scope
- Personalized recommendations (use random/recency)
- Real price chart data, reviews (mock data for now)
- Community content migration from explore page

## Capabilities

### New Capabilities
- `product-discovery`: 3 listing commands — featured (random), price drops (price_history delta), new arrivals (synced_at DESC)
- `product-detail`: SKU lookup — single product by PK

### Modified Capabilities
None

## Approach

**Backend**: Follow `dashboard_command` pattern — thin commands delegate to `ProductQueryService`. Each is a SQL query with `LIMIT ?` + `is_active=1`. Price drops JOIN with `price_history` subquery.

**Frontend**: Zero Svelte changes to existing pages. Create `routes/catalog/+page.svelte` mounting `<SearchPanel />`. Register commands.

## Affected Areas

| Area | Impact |
|------|--------|
| `services/product_query.rs` | New |
| `services/mod.rs` | Add module |
| `commands/product_command.rs` | New |
| `commands/mod.rs` | Add module |
| `main.rs` | Register 4 commands |
| `routes/catalog/+page.svelte` | New |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Price drops query slow on large catalogs | Med | Index exists on `price_history.sku`, limit to 50 |

## Rollback Plan

Single commit revert. `.catch(() => [])` guards make missing commands non-fatal. Remove `/catalog` route file.

## Dependencies

- `search_products` command (exists — SearchPanel depends on it)
- `RawProduct` / `SearchResult` types in `$lib/types/search.ts` (exist)

## Success Criteria

- [ ] Home page sections show real products (not empty, not error)
- [ ] Product detail shows real product for valid SKU; 404 for invalid
- [ ] `/catalog` route renders SearchPanel with working FTS5 search + GearCard grid
- [ ] All 4 commands have unit tests in service layer
- [ ] `cargo test && make lint` pass
