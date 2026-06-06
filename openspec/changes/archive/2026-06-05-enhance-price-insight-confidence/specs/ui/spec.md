# Delta for ui

> **Change**: enhance-price-insight-confidence
> **Status**: New — first top-level spec for the `ui` capability; covers the `PriceBadge` component contract

## ADDED Requirements

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

The tooltip MUST be 3 lines or fewer. The badge colour (green/amber) MUST NOT change the tooltip content; only the dot tint follows the colour.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| All fields present | `confidence = 78`, 45 points, 2 sources, 2d ago | Inspect `title` | 3-line tooltip with all 6 numeric fields |
| Day count = 0 | last point today | Render | "last 0 days ago" (or "today") |
| Day count = 1 | last point yesterday | Render | "last 1 day ago" |

### Requirement: `aria-label` MUST embed the confidence percentage and tier

The badge's `aria-label` MUST include the confidence percentage and the tier word so screen-reader users get the same signal as sighted users. The existing `pct%` reference MUST be preserved.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Green + High | `level = "green"`, `confidence = 92` | Inspect `aria-label` | Contains "Confidence 92%, high" and existing pct context |
| Amber + Medium | `level = "amber"`, `confidence = 65` | Inspect `aria-label` | Contains "Confidence 65%, medium" |
| Amber + Low | `level = "amber"`, `confidence = 30` | Inspect `aria-label` | Contains "Confidence 30%, low" |

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

- Custom popover / popper libraries (native `title=` is sufficient for v1)
- Animations on the dot scale
- Hover state beyond the native tooltip
- Components other than `PriceBadge` and `ProductCard`
- Recolouring or dimming the badge at low confidence (Option B from exploration)
