# UI Specification

> **Status**: New capability
> **Change**: enhance-price-insight-confidence

## Purpose

Govern the Svelte 5 component contracts that render price-insight-derived information to the user. This change introduces the `PriceBadge` confidence UI (3-dot scale, enriched tooltip, ARIA embedding) and the `ProductCard` prop forwarding. The capability is scoped to `PriceBadge` and `ProductCard` only — other components are out of scope.

## Requirements

### Requirement: PriceBadge MUST accept an optional `confidence: number` prop

`src/lib/components/PriceBadge.svelte` MUST declare a `confidence: number` prop in its `$props()` destructure with a default of `0`. The component MUST continue to accept the existing `level` and `pct` props unchanged.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Default confidence | Parent omits the prop | Render | `confidence` defaults to `0` (Low tier) |
| Explicit confidence | Parent passes `confidence={78}` | Render | Tier is "Medium"; dots render accordingly |
| Numeric type | TS strict mode | Compile | Prop is typed as `number`, not `string` |

### Requirement: Confidence MUST render as a 3-dot scale

The badge MUST render exactly three dots to the right of the existing label, where the count of filled dots corresponds to the tier:

| Tier | Range | Glyph | Dot colour |
|------|-------|-------|-----------|
| High | `>= 80` | `•••` (all filled) | tier-tinted (green when badge is green, amber when badge is amber) |
| Medium | `50..=79` | `••○` (2 filled, 1 empty) | tier-tinted |
| Low | `< 50` | `•○○` (1 filled, 2 empty) | neutral grey |

The dots MUST be announced to assistive tech via `aria-hidden="true"`; the tier is conveyed in `aria-label` (see REQ-BADGE-6).

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| High renders | `confidence = 85` | Render | 3 filled dots next to "Good price" |
| Medium renders | `confidence = 65` | Render | 2 filled + 1 empty dot |
| Low renders | `confidence = 30` | Render | 1 filled + 2 empty dots |
| Dots hidden from AT | Any render | Inspect DOM | `aria-hidden="true"` on dot container |

### Requirement: Tooltip MUST embed confidence context (native `title=`)

The badge's native `title=` attribute MUST include, in this order:

```
Confidence: NN% (Tier)
D data points · S sources · last D days ago
Min 30d: $X.XX  |  Avg 90d: $Y.YY  |  Current: $Z.ZZ
```

The tooltip MUST be exactly 3 lines when all fields are present. The badge colour (green/amber) MUST NOT change the tooltip content; only the dot tint follows the colour.

The tooltip MUST display all 3 lines when the IPC payload includes `cnt_30d`, `source_count_30d`, and `last_recorded_at`. If any field is missing, the component MUST gracefully omit the corresponding line and not render empty placeholders.

(Previously: The v1 tooltip showed only the first line (`Confidence: NN% (Tier)`) because the IPC payload lacked the full data fields. The full 3-line tooltip was deferred to a follow-up change.)

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| All fields present | `confidence = 78`, 45 points, 2 sources, 2d ago | Inspect `title` | 3-line tooltip with all 6 numeric fields |
| Day count = 0 | last point today | Render | "last 0 days ago" (or "today") |
| Day count = 1 | last point yesterday | Render | "last 1 day ago" |
| Full 3-line tooltip with all fields | `confidence = 78`, `cnt_30d = 45`, `source_count_30d = 2`, `last_recorded_at = 2 days ago`, `min_30d = 850.00`, `avg_90d = 950.00`, `current = 899.00` | Inspect `title` | Exactly 3 lines: line 1 is "Confidence: 78% (Medium)", line 2 is "45 data points · 2 sources · last 2 days ago", line 3 is "Min 30d: $850.00  |  Avg 90d: $950.00  |  Current: $899.00" |
| Tooltip with missing fields gracefully omits lines | `confidence = 78` but `cnt_30d` is undefined | Inspect `title` | Contains only line 1; no empty lines or placeholder text |
| Singular day formatting | `last_recorded_at` is 1 day ago | Inspect `title` | Line 2 contains "last 1 day ago" |
| Zero day formatting | `last_recorded_at` is today (0 days ago) | Inspect `title` | Line 2 contains "last 0 days ago" or "today" |

### Requirement: `aria-label` MUST embed the confidence percentage and tier

The badge's `aria-label` MUST include the confidence percentage and the tier word so screen-reader users get the same signal as sighted users. The existing `pct%` reference MUST be preserved.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Green + High | `level = "green"`, `confidence = 92` | Inspect `aria-label` | Contains "Confidence 92%, high" and existing pct context |
| Amber + Medium | `level = "amber"`, `confidence = 65` | Inspect `aria-label` | Contains "Confidence 65%, medium" |
| Amber + Low | `level = "amber"`, `confidence = 30` | Inspect `aria-label` | Contains "Confidence 30%, low" |
| Full context | `confidence = 85`, `cnt_30d = 45`, `source_count_30d = 2`, `last_recorded_at = 2 days ago` | Inspect `aria-label` | Contains "Confidence 85%, high. 45 data points, 2 sources, last 2 days ago" |

---

### Requirement: Settings save button MUST provide feedback

The `Settings` component MUST render a save button that, when clicked, persists the current form values to the settings store. After a successful save, the button MUST display a "Saved" feedback state for at least 2 seconds before reverting to the default label.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Save button persists settings | User changed currency from "USD" to "EUR" | Click save button | Settings store updated with `currency: "EUR"`; button text changes to "Saved"; after 2 seconds reverts to "Save" |
| Save button disabled while saving | User clicks save button | Save operation in progress | Save button has `disabled` attribute; no duplicate save requests triggered |

---

### Requirement: Settings form fields MUST reflect store state

The `Settings` component MUST bind its form fields (currency, price alert threshold, notification preferences) to the `settingsStore` so that changes are reactive and the store is the single source of truth.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Form fields load from store | `settingsStore` contains `{ currency: "GBP", threshold: 100, notifications: false }` | Component mounts | Currency select shows "GBP"; threshold input shows "100"; notifications toggle is unchecked |
| Form fields update store on change | Settings component is rendered | User changes threshold input to "75" | `settingsStore.threshold` is updated to 75; change is reflected immediately |

### Requirement: Hidden level MUST continue to suppress the badge

When `level === 'hidden'`, the badge MUST NOT be rendered. This is the existing behaviour, preserved unchanged. Confidence at hidden level is not exposed through this component (parent does not render it).

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Hidden suppressed | `level = "hidden"` | Render | No badge in the DOM |
| Parent guard | `ProductCard` checks `level !== 'hidden'` | Render | `PriceBadge` not instantiated |

### Requirement: `ProductCard` MUST pass the new `confidence` prop

`src/lib/components/ProductCard.svelte` MUST pass `confidence={priceInsight.confidence}` to `PriceBadge` alongside the existing `level` and `pct` props. No other structural changes to `ProductCard` are required.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Prop forwarded | `priceInsight = { level: "green", confidence: 85, ... }` | Render | `PriceBadge` receives `confidence={85}` |
| Numeric value | TS strict mode | Compile | `confidence` is `number`, not coerced to string |

## Out of Scope

- Custom popover / popper libraries (native `title=` is sufficient)
- Animations on the dot scale
- Hover state beyond the native tooltip
- Components other than `PriceBadge` and `ProductCard`
- Recolouring or dimming the badge at low confidence (Option B from exploration)
