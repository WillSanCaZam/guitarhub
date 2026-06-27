# Components Registry — GuitarHub

Last updated: 2026-06-22

## UI Atoms (`src/lib/components/ui/`)

| Component | File | Purpose |
|-----------|------|---------|
| Avatar | `Avatar.svelte` | User avatar display |
| Badge | `Badge.svelte` | Status/label badge |
| Button | `Button.svelte` | Primary action button |
| Card | `Card.svelte` | Container card |
| CategoryPills | `CategoryPills.svelte` | Multi-select category filter pills |
| Chip | `Chip.svelte` | Filter/tag chip |
| EmptyState | `EmptyState.svelte` | Contextual empty state with illustration |
| Input | `Input.svelte` | Text input field |
| PriceDisplay | `PriceDisplay.svelte` | Formatted price display |
| PriceRangeSlider | `PriceRangeSlider.svelte` | Price range filter slider |
| ProgressBar | `ProgressBar.svelte` | Progress indicator |
| SearchBar | `SearchBar.svelte` | Search input with autocomplete dropdown |
| SkeletonLoader | `SkeletonLoader.svelte` | Animated shimmer placeholder |
| StarRating | `StarRating.svelte` | Star rating display/input |

## Layout (`src/lib/components/layout/`)

| Component | File | Purpose |
|-----------|------|---------|
| AppShell | `AppShell.svelte` | Main app layout wrapper |
| BottomNav | `BottomNav.svelte` | Mobile bottom navigation |
| DrawerOverlay | `DrawerOverlay.svelte` | Off-canvas drawer backdrop |
| DrawerPanel | `DrawerPanel.svelte` | Off-canvas drawer panel |
| Sidebar | `Sidebar.svelte` | Desktop sidebar navigation (collapsible) |

## Auth (`src/lib/components/auth/`)

| Component | File | Purpose |
|-----------|------|---------|
| AuthGuard | `AuthGuard.svelte` | Route protection wrapper |
| LoginForm | `LoginForm.svelte` | Login form |

## Community (`src/lib/components/community/`)

| Component | File | Purpose |
|-----------|------|---------|
| FeedCard | `FeedCard.svelte` | Community feed item |
| HealthCheck | `HealthCheck.svelte` | System health indicator |
| LessonCard | `LessonCard.svelte` | Lesson preview card |
| ProfileHeader | `ProfileHeader.svelte` | User profile header |
| StreakCounter | `StreakCounter.svelte` | Activity streak display |

## Discovery (`src/lib/components/discovery/`)

| Component | File | Purpose |
|-----------|------|---------|
| FeaturedRig | `FeaturedRig.svelte` | Featured rig showcase with glow effects |
| FeedSection | `FeedSection.svelte` | Feed section wrapper |
| HeroSection | `HeroSection.svelte` | Landing hero section |
| TrendingPill | `TrendingPill.svelte` | Trending item pills with horizontal scroll |

## Product (`src/lib/components/product/`)

| Component | File | Purpose |
|-----------|------|---------|
| PriceHistory | `PriceHistory.svelte` | Price history chart |
| ProductDetail | `ProductDetail.svelte` | Full product detail view |
| StoreComparison | `StoreComparison.svelte` | Cross-store price comparison |

## Stores (`src/lib/components/stores/`)

| Component | File | Purpose |
|-----------|------|---------|
| ConnectModal | `ConnectModal.svelte` | Modal for connecting a store account via PAT |
| StoreCard | `StoreCard.svelte` | Individual store card with status and connect/disconnect |
| StoreIcon | `StoreIcon.svelte` | SVG icon per store (Reverb "R", generic fallback) |
| StoresGrid | `StoresGrid.svelte` | Grid of store cards with connection state |

## Shared (`src/lib/components/`)

| Component | File | Purpose |
|-----------|------|---------|
| SourceBadge | `SourceBadge.svelte` | Source store pill badge (Reverb, Guitar Center, Your Listing) |

## Root Components (`src/lib/components/`)

| Component | File | Purpose |
|-----------|------|---------|
| CollectionStatsCell | `CollectionStatsCell.svelte` | Collection statistics |
| CollectionView | `CollectionView.svelte` | Collection grid/list view |
| DashboardCell | `DashboardCell.svelte` | Dashboard metric cell |
| FilterBar | `FilterBar.svelte` | Search filters bar |
| GearCard | `GearCard.svelte` | Product card with transitions and micro-interactions |
| PriceBadge | `PriceBadge.svelte` | Price badge overlay |
| PriceChart | `PriceChart.svelte` | Price chart visualization |
| SearchPanel | `SearchPanel.svelte` | Search panel |
| Settings | `Settings.svelte` | Settings page |
| SyncStatusCell | `SyncStatusCell.svelte` | Sync status indicator |

---

## Component Details

### SearchBar

**Props**: `placeholder?: string`, `onSearch: (query: string) => void`
**Module exports**: `SEARCH_SUGGESTIONS` — static array of `{ label, category }` for autocomplete
**Events**: Invokes `onSearch` on Enter or suggestion selection
**Accessibility**: `role="combobox"` wrapper, `aria-expanded`, `aria-controls`, `aria-activedescendant`, keyboard navigation (ArrowUp/Down, Enter, Escape)
**Behavior**: Recent searches stored in `localStorage` (`guitarhub:recent-searches`, max 10, FIFO). Filters `SEARCH_SUGGESTIONS` when 2+ chars typed.

### CategoryPills

**Props**: `categories: CategoryPill[]`, `selected: string[]`, `onToggle: (id: string) => void`, `multiSelect?: boolean`
**Events**: `onToggle` callback with category id
**Accessibility**: `role="group"`, `aria-label`, `aria-pressed` on each pill
**Behavior**: Toggle selection. Active pills use `--glow-soft` background with orange glow.

### SkeletonLoader

**Props**: `variant: 'card-grid' | 'card-list' | 'text' | 'hero' | 'detail'`, `count?: number`
**Accessibility**: `aria-busy="true"`, `aria-label` per variant
**Behavior**: Uses canonical `@keyframes shimmer` from `animations.css`. Respects `prefers-reduced-motion`.

### EmptyState

**Props**: `variant: 'collection' | 'search' | 'wishlist' | 'feed' | 'lessons' | 'riffs' | 'default'`, `title: string`, `description?: string`, `actionLabel?: string`
**Events**: `action` callback when action button clicked
**Accessibility**: Semantic `h2` heading, descriptive text
**Behavior**: Renders contextual SVG illustration per variant. Uses `--empty-state-*` tokens.

### GearCard (renamed from ProductCard)

**Props**: `product: GearCardProduct`, `inCollection?: boolean`
**Events**: `add` (to collection), `wishlistToggle`, `openUrl`
**Accessibility**: `role="article"`, `tabindex="0"`
**Behavior**: Enter/exit transitions (fade + slide). Wishlist heart bounce animation. Price pulse on update. Uses `SkeletonLoader` for image loading.

### FeaturedRig (modified)

**Changes**: Added left orange border eyebrow, circular artist photo with glow border, gear preview skeleton cards, arrow-right CTA icon. Responsive vertical stack below 768px.

### TrendingPill (modified)

**Changes**: Horizontal scroll with `scroll-snap-type: x mandatory`, hidden scrollbar, 🔥 icon before label, orange hover glow. Uses `--border-subtle` and `--border-glow` tokens.

### Sidebar (modified)

**Changes**: Collapsible between 240px and 64px. Collapse state persisted in `localStorage`. Tooltips on collapsed icons. Width transition via `--sidebar-transition` token.
