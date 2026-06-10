# Delta for price-chart

> **Change**: mvp-fixes — Semantic bug C6

## ADDED Requirements

### Requirement: PriceChart MUST guard against empty or undefined SKU

The `PriceChart.svelte` component MUST NOT invoke the Tauri `get_price_history` command when the `sku` prop is falsy (`null`, `undefined`, empty string, or whitespace-only). When `sku` is falsy, the component MUST render the empty state immediately without making an IPC call.

#### Scenario: SKU is undefined

- GIVEN `PriceChart` is rendered without a `sku` prop
- WHEN the component mounts
- THEN no `invoke('get_price_history')` call is made
- AND the component renders the "No price history available" empty state

#### Scenario: SKU is empty string

- GIVEN `PriceChart` is rendered with `sku = ""`
- WHEN the component mounts
- THEN no `invoke('get_price_history')` call is made
- AND the component renders the empty state

#### Scenario: SKU is whitespace-only

- GIVEN `PriceChart` is rendered with `sku = "   "`
- WHEN the component mounts
- THEN no `invoke('get_price_history')` call is made
- AND the component renders the empty state

#### Scenario: Valid SKU fetches normally

- GIVEN `PriceChart` is rendered with `sku = "FENDER-STRAT-001"`
- WHEN the component mounts
- THEN `invoke('get_price_history', { sku, windowDays })` is called
- AND the chart renders the returned data