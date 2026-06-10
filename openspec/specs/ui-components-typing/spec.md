# Delta for ui-components-typing

> **Status**: Type enforcement — zero behaviour change
> **Change**: mvp-ui

## ADDED Requirements

### Requirement: Script tags MUST declare `lang="ts"`

Settings.svelte, PriceChart.svelte, PriceBadge.svelte, ProductCard.svelte, and ProductDetail.svelte MUST open script tags with `<script lang="ts">`. Each MUST define `interface Props` with typed destructuring, following the DashboardCell pattern.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| All 5 components compile | All scripts use `lang="ts"` | `npm run check` | Zero type errors |
| Props typed | `interface Props` declared | Compile | Each prop has explicit TS type |
| Missing `lang="ts"` | Any script tag lacks it | TypeScript check | Compile error surfaced |

### Requirement: Reactive state MUST be typed

Each `$state()` call MUST carry an explicit type annotation (`$state<Type>(initial)`). `$derived()` expressions SHOULD infer from typed sources. `$props()` MUST destructure against a declared `Props` interface.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| `$state` typed | `let x = $state<string>('')` | Compile | `x` typed `string` |
| `$derived` inferred | Expression references typed state | Compile | Derived type inferred correctly |
| Props destructured | `let { ... }: Props = $props()` | Compile | All props typed |
| `$state` missing annotation | `let x = $state(someUntypedRef)` | TS strict check | Compile error (implicit `any`) |

### Requirement: `invoke()` MUST use typed generics

Every Tauri `invoke()` call in these 5 components MUST specify the return type via `invoke<ReturnType>('command', { ... })`.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Correct type | `invoke<SearchResult>('search_products', ...)` | Compile | Return type checked |
| Mismatched type | `invoke<number>('get_price_insight', ...)` returning object | TypeScript check | Type error surfaced |

### Requirement: Runtime behaviour MUST be preserved

Adding type annotations and `lang="ts"` MUST NOT alter rendered output, state transitions, IPC call payloads, or event handlers. All existing component tests MUST pass without modification.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Identical render | Component after typing | Render | DOM output identical to before |
| IPC unchanged | Component after typing | Invoke call | Payload structure unchanged |
| All tests pass | Full typing applied | `npm run check` + test suite | Zero failures |
