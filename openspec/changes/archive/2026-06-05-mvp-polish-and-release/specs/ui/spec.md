# Delta for ui

## MODIFIED Requirements

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

#### Scenario: Full 3-line tooltip with all fields

- GIVEN `confidence = 78`, `cnt_30d = 45`, `source_count_30d = 2`, `last_recorded_at = 2 days ago`, `min_30d = 850.00`, `avg_90d = 950.00`, `current = 899.00`
- WHEN the badge renders
- THEN the `title` attribute contains exactly 3 lines separated by newlines
- AND line 1 is "Confidence: 78% (Medium)"
- AND line 2 is "45 data points · 2 sources · last 2 days ago"
- AND line 3 is "Min 30d: $850.00  |  Avg 90d: $950.00  |  Current: $899.00"

#### Scenario: Tooltip with missing fields gracefully omits lines

- GIVEN `confidence = 78` but `cnt_30d` is undefined
- WHEN the badge renders
- THEN the `title` attribute contains only line 1
- AND no empty lines or placeholder text are present

#### Scenario: Singular day formatting

- GIVEN `last_recorded_at` is 1 day ago
- WHEN the badge renders
- THEN line 2 contains "last 1 day ago" (or "last 1 day ago" — singular "day" is acceptable)

#### Scenario: Zero day formatting

- GIVEN `last_recorded_at` is today (0 days ago)
- WHEN the badge renders
- THEN line 2 contains "last 0 days ago" or "today"

## ADDED Requirements

### Requirement: Settings save button MUST provide feedback

The `Settings` component MUST render a save button that, when clicked, persists the current form values to the settings store. After a successful save, the button MUST display a "Saved" feedback state for at least 2 seconds before reverting to the default label.

#### Scenario: Save button persists settings

- GIVEN the user has changed the currency from "USD" to "EUR"
- WHEN the save button is clicked
- THEN the settings store is updated with `currency: "EUR"`
- AND the save button text changes to "Saved"
- AND after 2 seconds the button text reverts to "Save"

#### Scenario: Save button disabled while saving

- GIVEN the user clicks the save button
- WHEN the save operation is in progress
- THEN the save button is disabled
- AND no duplicate save requests can be triggered

---

### Requirement: Settings form fields MUST reflect store state

The `Settings` component MUST bind its form fields (currency, price alert threshold, notification preferences) to the `settingsStore` so that changes are reactive and the store is the single source of truth.

#### Scenario: Form fields load from store

- GIVEN the `settingsStore` contains `{ currency: "GBP", threshold: 100, notifications: false }`
- WHEN the Settings component mounts
- THEN the currency select shows "GBP"
- AND the threshold input shows "100"
- AND the notifications toggle is unchecked

#### Scenario: Form fields update store on change

- GIVEN the Settings component is rendered
- WHEN the user changes the threshold input to "75"
- THEN the `settingsStore.threshold` is updated to 75
- AND the change is reflected immediately

---

### Requirement: PriceBadge `aria-label` MUST be updated for full tooltip

The badge's `aria-label` MUST include the full 3-line context when all fields are present, so screen-reader users receive the same information as the tooltip.

#### Scenario: aria-label with full context

- GIVEN `confidence = 85`, `cnt_30d = 45`, `source_count_30d = 2`, `last_recorded_at = 2 days ago`
- WHEN the badge renders
- THEN the `aria-label` contains "Confidence 85%, high. 45 data points, 2 sources, last 2 days ago"

---

### Requirement: Hidden level MUST continue to suppress the badge

When `level === 'hidden'`, the badge MUST NOT be rendered. This is the existing behaviour, preserved unchanged. Confidence at hidden level is not exposed through this component (parent does not render it).

#### Scenario: Hidden suppressed

- GIVEN `level = "hidden"`
- WHEN the component renders
- THEN no badge is present in the DOM

#### Scenario: Parent guard

- GIVEN `ProductCard` checks `level !== 'hidden'`
- WHEN `ProductCard` renders
- THEN `PriceBadge` is not instantiated when level is hidden
