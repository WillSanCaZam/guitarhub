# Design: Bento Grid Dashboard

## Technical Approach

Replace the vertical-stack landing page with a responsive CSS Grid bento layout. Nine cells surface search results, sync status, catalog stats, wishlist, recent searches, a featured product, settings, a placeholder, and version info. Existing search and settings behavior remain unchanged; only the layout wrapper and data surfacing change.

## Architecture Decisions

| Decision | Options | Tradeoffs | Chosen |
|---|---|---|---|
| Grid spans | CSS classes vs inline styles | Classes are reusable and mobile overrides are simpler | CSS classes (`.cell-hero`, `.cell-wide`, `.cell-tall`, `.cell-standard`) |
| Cell wrapper | Dedicated `DashboardCell.svelte` vs no wrapper | Wrapper centralizes glassmorphism, empty states, and loading spinners | `DashboardCell.svelte` with `<slot>` |
| Backend commands | New `dashboard_command.rs` vs extending existing modules | Keeps dashboard-specific queries isolated; COUNT queries are too small for services | `dashboard_command.rs` with direct `sqlx::query_scalar` |
| Data flow | Parallel `invoke` on mount vs staggered | Faster initial paint; 3 queries are cheap | Parallel invoke into `dashboard.ts` writable store |
| Delivery | 2 PRs vs 1 PR | Stay under 400-line review budget | 2 chained PRs (layout + content) |

## Data Flow

```
+page.svelte (onMount)
  ├─ invoke get_total_products()  ──→ dashboard.ts store
  ├─ invoke get_wishlist_count()  ──→ dashboard.ts store
  ├─ invoke get_recent_searches() ──→ dashboard.ts store
  └─ $syncResult (existing store) ──→ Sync cell
```

`DashboardCell` reads store values and renders children. Search results remain fetched on-demand.

## File Changes

| File | Action | Description |
|---|---|---|
| `src/lib/components/DashboardCell.svelte` | Create | Wrapper: glassmorphism, empty state, loading spinner |
| `src/routes/+page.svelte` | Modify | Replace vertical stack with grid container + 9 cells |
| `src/lib/stores/dashboard.ts` | Create | Writable store for dashboard stats |
| `src-tauri/src/commands/dashboard_command.rs` | Create | 3 IPC commands: total products, wishlist count, recent searches |
| `src-tauri/src/commands/mod.rs` | Modify | Add `pub mod dashboard_command;` |
| `src-tauri/src/main.rs` | Modify | Register 3 commands in `generate_handler!` |

## Interfaces / Contracts

**Rust IPC commands** (`dashboard_command.rs`):

```rust
#[tauri::command]
pub async fn get_total_products(state: State<'_, AppState>) -> Result<u32, crate::AppError>;

#[tauri::command]
pub async fn get_wishlist_count(state: State<'_, AppState>) -> Result<u32, crate::AppError>;

#[tauri::command]
pub async fn get_recent_searches() -> Result<Vec<String>, crate::AppError>;
```

`get_total_products` and `get_wishlist_count` use `sqlx::query_scalar` with `SELECT COUNT(*) FROM products_meta` and `SELECT COUNT(*) FROM wishlist`. `get_recent_searches` reads from the Tauri store plugin (or an in-memory cache if the plugin is unavailable).

**TypeScript store** (`dashboard.ts`):

```ts
export interface DashboardStats {
  totalProducts: number;
  wishlistCount: number;
  recentSearches: string[];
  loading: boolean;
  error: string | null;
}
export const dashboardStats = writable<DashboardStats>({ ...default });
```

**Grid CSS** (scoped in `+page.svelte`):

```css
.bento-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 16px; }
.cell-hero { grid-column: span 2; grid-row: span 2; }
.cell-wide { grid-column: span 2; }
.cell-tall { grid-row: span 2; }
.cell-standard { grid-column: span 1; grid-row: span 1; }
@media (max-width: 768px) { .bento-grid { grid-template-columns: 1fr; } .cell-hero, .cell-wide, .cell-tall, .cell-standard { grid-column: span 1; grid-row: span 1; } }
```

**Glassmorphism** (`DashboardCell.svelte`):

```css
.glass { background: rgba(255,255,255,0.7); backdrop-filter: blur(10px); border: 1px solid rgba(255,255,255,0.3); border-radius: 12px; }
```

## Cell Content Mapping

| Cell | Span | Content | Data Source |
|---|---|---|---|
| 1 | Hero | ProductCard grid (search results) | Existing `search_products` + `results` state |
| 2 | Wide | Sync status bar + drop count | `$syncResult` store |
| 3 | Standard | "{count} products in catalog" | `dashboardStats.totalProducts` |
| 4 | Standard | "{count} items in wishlist" (0 → empty state) | `dashboardStats.wishlistCount` |
| 5 | Standard | Recent searches list | `dashboardStats.recentSearches` |
| 6 | Wide | Featured product (random from last results) | `results` array |
| 7 | Standard | Settings shortcut button | Inline `Settings` component |
| 8 | Standard | Placeholder "Price trends coming soon" | Static |
| 9 | Standard | App version + GitHub link | Static |

## Implementation Order

**PR 1: Layout + foundation (≤250 LoC)**
- `DashboardCell.svelte` component
- `+page.svelte` grid layout
- 3 backend commands + registration
- `dashboard.ts` store
- Cells 1–5 (search, sync, total, wishlist, recent searches)

**PR 2: Content + polish (≤250 LoC)**
- Cells 6–9 (featured, settings, placeholder, version)
- Empty states for all 9 cells
- Keyboard navigation
- CSS polish (glassmorphism, transitions)
- Mobile testing

## Testing Strategy

| Layer | What to Test | Approach |
|---|---|---|
| Rust unit | `dashboard_command` COUNT queries | None (simple `sqlx::query_scalar`, no logic) |
| Frontend unit | Component rendering | Manual visual testing (no test framework yet) |
| Build | Type correctness | `npx tsc --noEmit` |
| Build | Bundle | `npm run build` |
| E2E | Full suite | `make test` — no regression in search, sync, or settings |

## Edge Cases

- **Empty catalog**: `totalProducts` = 0 triggers "No products yet" in cell 3.
- **No recent searches**: Cell 5 shows "Start searching to see history".
- **Mobile (<768px)**: Single column; all spans reset to 1 via media query.
- **Dark mode**: Glassmorphism uses `rgba(255,255,255,0.7)` for light backgrounds; if dark mode is introduced later, switch to `rgba(0,0,0,0.4)` with a CSS variable.

## Migration / Rollout

No migration required. New components and commands are additive. Rollback: revert `+page.svelte` to the previous vertical-stack version.

## Open Questions

- None.
