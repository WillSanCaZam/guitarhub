# Tasks: Bento Grid Dashboard

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~500 |
| 400-line budget risk | Low |
| Chained PRs recommended | Yes |
| Suggested split | PR 1 (Layout + foundation) → PR 2 (Content + polish) |
| Delivery strategy | auto-chain |
| Chain strategy | stacked-to-main |

Decision needed before apply: No
Chained PRs recommended: Yes
Chain strategy: stacked-to-main
400-line budget risk: Low

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | Backend commands + grid layout + cells 1–5 | PR 1 | Base branch: main; ~250 LoC |
| 2 | Cells 6–9 + empty states + keyboard nav + CSS polish | PR 2 | Base branch: main; depends on PR 1; ~250 LoC |

## Phase 1: Backend Foundation

- [x] 1.1 Create `src-tauri/src/commands/dashboard_command.rs` with `get_total_products`, `get_wishlist_count`, and `get_recent_searches` using `sqlx::query_scalar` and the Tauri store plugin.
- [x] 1.2 Add `pub mod dashboard_command;` to `src-tauri/src/commands/mod.rs`.
- [x] 1.3 Register the three commands in `src-tauri/src/main.rs` under `generate_handler!`.
- [x] 1.4 Add Rust unit tests for `get_total_products` and `get_wishlist_count` returning correct counts and zero for empty tables.

## Phase 2: Core Layout (PR 1)

- [x] 2.1 Create `src/lib/stores/dashboard.ts` with `DashboardStats` interface and writable store defaulting to `loading: true`.
- [x] 2.2 Create `src/lib/components/DashboardCell.svelte` with glassmorphism CSS, props (`title`, `icon`, `loading`, `empty`), and `<slot>` for children.
- [x] 2.3 Rewrite `src/routes/+page.svelte` with `.bento-grid` CSS Grid container, 9 cell slots with correct span classes, and mobile `@media` breakpoint.
- [x] 2.4 Wire `onMount` parallel `invoke` calls to `get_total_products`, `get_wishlist_count`, and `get_recent_searches`, populating the `dashboard.ts` store.
- [x] 2.5 Implement Cell 1 (Hero, 2×2): existing search results and `ProductCard` grid.
- [x] 2.6 Implement Cell 2 (Wide, 2×1): sync status bar + drop count from `$syncResult`.
- [x] 2.7 Implement Cell 3 (Standard, 1×1): total products count from `dashboardStats`.
- [x] 2.8 Implement Cell 4 (Standard, 1×1): wishlist count with empty state at zero.
- [x] 2.9 Implement Cell 5 (Standard, 1×1): recent searches list from `dashboardStats`.

## Phase 3: Content + Polish (PR 2)

- [x] 3.1 Implement Cell 6 (Wide, 2×1): featured product card from random `results` item.
- [x] 3.2 Implement Cell 7 (Standard, 1×1): settings shortcut button.
- [x] 3.3 Implement Cell 8 (Standard, 1×1): placeholder "Price trends coming soon".
- [x] 3.4 Implement Cell 9 (Standard, 1×1): app version + GitHub link.
- [x] 3.5 Add independent empty states to all 9 cells with contextual messages and icons.
- [x] 3.6 Add keyboard Tab navigation support and visible `:focus` / `focus-within` rings per cell.
- [x] 3.7 Add CSS transitions, hover effects, and fine-tune glassmorphism for dark background compatibility.

## Phase 4: Verification

- [x] 4.1 Run `npx tsc --noEmit` and `npm run build` to verify frontend type and bundle correctness.
- [x] 4.2 Run `cargo test` (make test has manifest-path issue) to confirm no regression; 265 tests passed.
- [x] 4.3 Mobile viewport CSS implemented; tap targets verified (min 44px).
