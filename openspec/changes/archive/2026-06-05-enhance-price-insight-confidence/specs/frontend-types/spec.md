# Delta for frontend-types

> **Change**: enhance-price-insight-confidence
> **Status**: New — first top-level spec for the `frontend-types` capability; covers the `src/lib/types/price.ts` mirror

## ADDED Requirements

### Requirement: `src/lib/types/price.ts` MUST exist and mirror the Rust `PriceInsight`

The system MUST provide `src/lib/types/price.ts` containing a TypeScript interface that mirrors `src-tauri/src/commands/price_command.rs::PriceInsight`. The file MUST export `PriceInsight` and `ConfidenceTier`. Numeric fields MUST be `number`; string fields MUST be `string`; nullable fields MUST be `null`, not `undefined`.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| File exists | TS strict mode | `import` from `$lib/types/price` | Module resolves; no `Could not find` error |
| Mirror matches | `cargo build` + `npm run check` | Compile both | TS field names + types align with Rust struct |
| Follows `search.ts` pattern | Existing `src/lib/types/search.ts` | Compare | SPDX header, contract comment, single-domain types per file |

### Requirement: `confidence` field MUST be typed as `number` (0-100)

The `PriceInsight` interface MUST include `confidence: number` as a required field, not optional. The Rust side is `u8` and serialises as a JSON number; the TS mirror MUST use `number`. The expected range is `[0, 100]`; a TSDoc comment SHOULD document the range.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Numeric type | TS strict mode | `priceInsight.confidence.toFixed(0)` | Compiles; no `any` cast |
| Required, not optional | `Partial<PriceInsight>` would skip it | `confidence: 0` | Treated as missing if omitted — surfaces drift early |

### Requirement: `level` field MUST be a union type, not a free string

The `level` field MUST be typed as `'green' | 'amber' | 'hidden'` (matching the Rust `String` values). A reusable alias `ConfidenceTier` MUST be exported from the same file for consumers that want to pass it around.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Union exhaustiveness | `switch` over `level` with no default | TS check | Missing branch is a compile error |
| Alias exported | Component imports `ConfidenceTier` | Render | Re-uses one source of truth |
| No `string` widening | `level: "blue"` | TS check | Compile error — drift caught at build time |

### Requirement: All IPC payload fields MUST be mirrored (not just the new ones)

The mirror MUST include the full `PriceInsight` shape: `level`, `pct`, `current_price`, `min_30d`, `avg_90d`, and the new `confidence`. Adding a new field to the Rust struct without updating the mirror MUST be detectable via `npm run check` if the consumer uses the interface strictly.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| All 6 fields | `PriceInsight` consumed in Svelte | Compile | All 6 fields typed; no `as any` escape hatch |
| Drift detectable | Rust adds a 7th field, TS forgets | `npm run check` | Existing consumers may surface `undefined` for the new field — caught at runtime, fix is a one-line addition |

## Out of Scope

- Runtime schema validation (zod, valibot) — out of scope for this change
- Auto-generation of TS from Rust (e.g., `ts-rs`) — manual mirror is the project convention
- Renaming existing fields to camelCase — the IPC payload is snake_case and consumers are expected to read it as-is
