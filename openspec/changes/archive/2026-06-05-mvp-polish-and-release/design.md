# Design: MVP Polish and Release (v0.1.0)

## Technical Approach

Four sequential, independently-verifiable phases. No schema changes, no backend IPC contract changes.

1. **UI Polish** — Fix Settings save button, add `.glassmorphism` class to `DashboardCell`, expand `PriceBadge` tooltip, and add gain/loss to Cell 8 via reactive store derivation.
2. **Frontend Tests** — Write vitest component tests following the existing `DashboardCell.test.ts` pattern. Mock `tauriInvoke` via `setupTests.ts` and mock stores by importing and calling `.set()`.
3. **CI/CD & Docs** — Extend `scraper/cli.py` with a `--validate-input` flag; add the `validate-input` step to `scrape.yml`. Rewrite CHANGELOG `[Unreleased]` into `v0.1.0`.
4. **Collection Dashboard** — Compute gain/loss reactively in `+page.svelte` Cell 8 from `$collectionStore.items`.

## Architecture Decisions

### Decision: Glassmorphism via `@supports` with testable class

| Option | Tradeoff | Decision |
|--------|----------|----------|
| Global `.glassmorphism` class in `+page.svelte` | Duplicates existing `DashboardCell` styles; violates DRY | **Rejected** |
| Rename `.dashboard-cell` to `.glassmorphism` | Breaks existing CSS selectors and tests | **Rejected** |
| Add `.glassmorphism` as extra class on `DashboardCell` root | Reuses existing styles, satisfies testability, zero breaking change | **Chosen** |

Add `@supports (backdrop-filter: blur(12px))` and `@supports not` blocks inside `DashboardCell.svelte` so unsupported browsers fall back to `rgba(255,255,255,0.08)`.

### Decision: Settings save button — explicit feedback over auto-save

The component already auto-saves on every `onchange`/`oninput`. The no-op `type="submit"` button will be wired to an explicit `saveAll()` function that re-invokes `save_setting` for both keys, then flips a `saved` state for 2 seconds. This gives the user tangible feedback without changing the existing persistence model.

### Decision: Gain/loss computed in component, not backend

| Option | Tradeoff | Decision |
|--------|----------|----------|
| Add `total_gain` to `CollectionStats` / IPC | Requires Rust backend change; violates "no schema changes" | **Rejected** |
| Derive from `$collectionStore.items` in `+page.svelte` | Zero backend touch; reactive; uses existing data | **Chosen** |

Formula: `items.reduce((sum, i) => sum + ((i.estimated_value ?? 0) - (i.purchase_price ?? 0)), 0)`. Displayed with locale formatting via `toLocaleString()` and sign prefix.

### Decision: CI validation — extend existing CLI instead of new file

The spec references `scraper/run_all.py` which does not exist. Rather than creating a new wrapper, extend `scraper/cli.py` with a `--validate-input` flag and `--input-dir` argument. This keeps the CLI surface small and reuses the existing `CatalogFile` Pydantic model for schema validation.

## Data Flow

```
User changes Settings field
  → onChannelChange / onConfigChange
  → invoke('save_setting')  (auto-save)
  → click "Save" → saveAll() → saved=true (2s feedback)

DashboardCell mount
  → CSS class="dashboard-cell glassmorphism"
  → @supports backdrop-filter → blur(12px)
  → @supports not → solid rgba fallback

PriceBadge receives props
  → builds tooltipLines[] reactively
  → omits line if prop is undefined
  → title={tooltipLines.join('\n')}

Collection store updates
  → $collectionStore.items changes
  → +page.svelte $derived: collectionGainLoss recalculates
  → Cell 8 re-renders gain/loss with color class
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `src/lib/components/DashboardCell.svelte` | Modify | Add `.glassmorphism` class; add `@supports` fallback for `backdrop-filter` |
| `src/lib/components/Settings.svelte` | Modify | Fix save button no-op; add `saving`/`saved` state with 2s feedback and disabled attr |
| `src/lib/components/PriceBadge.svelte` | Modify | Add optional props (`cnt_30d`, `source_count_30d`, `last_recorded_at`, `min_30d`, `avg_90d`, `current`); build 3-line `title` and `aria-label` |
| `src/routes/+page.svelte` | Modify | Add `$derived` gain/loss in Cell 8; add gain/loss color/formatting markup |
| `src/lib/components/__tests__/ProductCard.test.ts` | Create | Rendering, price badge, add-to-collection action, hide-when-in-collection |
| `src/lib/components/__tests__/Settings.test.ts` | Create | Form rendering, save feedback, disabled state |
| `src/lib/components/__tests__/PriceChart.test.ts` | Create | Chart rendering with data, empty state, loading state |
| `src/routes/__tests__/+page.test.ts` | Create | 9 cells render, loading states, empty states |
| `scraper/cli.py` | Modify | Add `--validate-input` and `--input-dir` flags |
| `.github/workflows/scrape.yml` | Modify | Add `validate-input` step after upload and before any publish boundary |
| `CHANGELOG.md` | Modify | Replace `[Unreleased]` with dated `## [0.1.0] - YYYY-MM-DD` |
| `README.md` | Modify | Add v0.1.0 badge/line |

## Interfaces / Contracts

### PriceBadge new optional props
```ts
interface PriceBadgeProps {
  level: 'green' | 'amber' | 'hidden';
  pct: number;
  confidence: number;
  cnt_30d?: number;
  source_count_30d?: number;
  last_recorded_at?: number; // days ago
  min_30d?: number;
  avg_90d?: number;
  current?: number;
}
```

### Scraper CLI extension
```bash
python -m scraper --validate-input --input-dir incoming/
```
- Exit code 0 if all JSON files in `incoming/` are valid `CatalogFile` schema.
- Exit code 1 if any file is malformed, with `logger.error` naming the file and error.
- Idempotent: read-only, no side effects.

### Settings internal state
```ts
let saved = $state(false);
let saving = $state(false);
```
- `saveAll()` sets `saving=true`, re-invokes both `save_setting` calls, then `saving=false; saved=true`.
- `setTimeout(() => saved = false, 2000)` reverts label.

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit | `PriceBadge` tooltip lines | Render with all/new props; assert `title` attribute and `aria-label` |
| Unit | `Settings` save feedback | Mock `invoke` to resolve; click button; assert "Saved" text; assert `disabled` during save |
| Unit | `ProductCard` add action | Mock `collectionStore` with `collectedSkus`; assert button visibility; mock `addToCollection` IPC |
| Unit | `PriceChart` empty/loading | Mock `invoke('get_price_history')` to return `[]` or throw; assert empty/loading text |
| Unit | `+page` cells & gain/loss | Mock `collectionStore`, `dashboardStats`, `syncResult`; render page; assert 9 `.dashboard-cell` elements and gain/loss text |
| Integration | `scraper/cli.py --validate-input` | Run CLI against valid/invalid JSON fixtures in `scraper/tests/`; assert exit codes 0 and 1 |
| E2E | Not required for this change | — |

All frontend tests use the existing `setupTests.ts` global `vi.mock('@tauri-apps/api/core')`. Store mocks are done by importing the writable store and calling `.set(mockState)` before `render()`.

## Migration / Rollout

No migration required. No database schema changes. Rollout is four standalone commits:

1. `ui-polish` — Settings, DashboardCell, PriceBadge, +page gain/loss
2. `frontend-tests` — 4 new test files
3. `ci-docs` — `scrape.yml`, CHANGELOG, README
4. `collection-gain-loss` — Cell 8 markup (can be squashed into commit 1 if preferred)

Rollback: `git revert <commit>` for any failing phase. No data loss risk.

## Open Questions

- The `scrape.yml` spec references `download-artifact` and `--publish-index` steps that do not exist in the current workflow. The design adds the `validate-input` step as a gate before the artifact upload (treating upload as the publish boundary). If a true `publish-index` job is added later, the `validate-input` step should be moved there.
- `settingsStore` is referenced in the spec but does not exist in the codebase; the design keeps the existing per-field `invoke('save_setting')` pattern and adds visual feedback only.
