# Delta for frontend-scaffolding

> **Change**: mvp-completion  
> **Status**: Modified — empty route shells upgraded to live IPC-connected search UI

## ADDED Requirements

### Requirement: +page.svelte MUST provide search UI

The system MUST provide a search bar and result grid in `+page.svelte`. Typing a search query SHALL invoke `search_products` via `@tauri-apps/api/core`. Results SHALL render as a responsive grid of `ProductCard` components.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Search triggers IPC | 3+ characters typed | User presses Enter/clicks Search | `invoke('search_products', { query })` called |
| Results render | 4 products returned | IPC resolves | 4 `ProductCard` components in grid |
| Empty results | No matches | Search executed | "No results found" message shown |
| Loading state | IPC in-flight | Search submitted | Spinner or skeleton shown during fetch |

### Requirement: +layout.svelte MUST include navigation

The system MUST provide a nav bar in `+layout.svelte` with a sync button and optional settings link. The sync button SHALL invoke `sync_catalog` and display progress.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Sync button triggers IPC | Layout renders | User clicks Sync | `invoke('sync_catalog', { url })` called |
| Sync progress shown | Syncing | Sync completes | Nav shows progress indicator during sync |

### Requirement: Error states MUST be displayed

IPC errors from `search_products` or `sync_catalog` SHALL be surfaced to the user as inline error messages, not silent failures.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Search error | Backend returns `AppError` | Search fails | Error message rendered above results |
| Network error | Tauri IPC fails | invoke rejected | "Connection error" message shown |

### Requirement: Search results MUST paginate

The result grid SHALL support pagination via `limit` and `offset` params passed to `search_products`. A "Load more" button SHALL fetch the next page.

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Load more | 20 results total, 10 displayed | Click "Load more" | Next 10 products appended |
| No more results | All 10 results displayed | Click "Load more" | Button disabled, no duplicate fetch |

## MODIFIED Requirements

### Requirement: Routes MUST reference IPC-connected components

`+page.svelte` and `+layout.svelte` SHALL import `@tauri-apps/api/core` and wire Tauri commands. The existing component imports are retained but now receive real data instead of static props.
(Previously: Routes imported components with static/mock props only)

| Case | Precondition | Action | Outcome |
|------|-------------|--------|---------|
| Components render with real data | Search returns 10 products | Page renders | `ProductCard` receives real product data from IPC |
