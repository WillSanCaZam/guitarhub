# Tasks: Community Hub Integration

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 3200–4000 |
| 400-line budget risk | High |
| Chained PRs recommended | Yes |
| Suggested split | PR 1 → PR 2 → PR 3 → PR 4 → PR 5 → PR 6 |
| Delivery strategy | ask-on-risk |
| Chain strategy | pending |

Decision needed before apply: Yes
Chained PRs recommended: Yes
Chain strategy: pending
400-line budget risk: High

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | Design system tokens + UI atoms | PR 1 | base: feature/community-hub; tokens.css, typography.css, 7 UI components, types |
| 2 | Navigation shell + layout wiring | PR 2 | AppShell, Sidebar, BottomNav, +layout.svelte refactor; depends on PR 1 |
| 3 | Backend foundation (SQLite + Rust services) | PR 3 | migration, 3 services, 3 commands, capabilities; independent of frontend |
| 4 | Auth layer (store + UI + guard) | PR 4 | auth store, LoginForm, AuthGuard; depends on PR 1 (UI atoms) + PR 3 (backend) |
| 5 | Community components + stores | PR 5 | community/profile stores, FeedCard, LessonCard, ProfileHeader, StreakCounter; depends on PR 1 + PR 4 |
| 6 | Community pages + routing | PR 6 | 7 new routes, page.css migration; depends on PR 2 (shell) + PR 5 (components) |

## Phase 1: Design System Foundation

- [x] 1.1 Create `src/lib/styles/tokens.css` — Acoustic Dark Modern CSS custom properties (colors, spacing, radii, elevation levels from DESIGN.md)
- [x] 1.2 Create `src/lib/styles/typography.css` — @font-face for Hanken Grotesk + JetBrains Mono, type scale (display-lg through label-sm)
- [x] 1.3 Create `src/lib/types/community.ts` — TypeScript interfaces: UserProfile, Lesson, Riff, FeedItem, Comment, Streak, Follow
- [x] 1.4 Create `src/lib/components/ui/Button.svelte` — primary/secondary/ghost variants, amber focus ring, disabled state
- [x] 1.5 Create `src/lib/components/ui/Card.svelte` — tonal layering (surface-container-low), inner glow top-edge, media slot
- [x] 1.6 Create `src/lib/components/ui/Avatar.svelte` — image with initials fallback, size prop (sm/md/lg)
- [x] 1.7 Create `src/lib/components/ui/Badge.svelte` — status/role badges with semantic color variants
- [x] 1.8 Create `src/lib/components/ui/Chip.svelte` — genre/difficulty tags, secondary-container background
- [x] 1.9 Create `src/lib/components/ui/Input.svelte` — dark input, amber bottom-border on focus, label-sm in JetBrains Mono
- [x] 1.10 Create `src/lib/components/ui/ProgressBar.svelte` — thin amber progress indicator, animated fill
- [x] 1.11 Test: all UI atoms render correctly, tokens.css variables resolve, fonts load

## Phase 2: Navigation Shell

- [x] 2.1 Create `src/lib/components/layout/Sidebar.svelte` — desktop nav, route-aware active states, logo, nav items (Feed, Explore, Lessons, My Gear, Saved Riffs, Profile), settings link
- [x] 2.2 Create `src/lib/components/layout/BottomNav.svelte` — mobile nav, 5-icon layout, active indicator, community items show "Connect to enable" when offline
- [x] 2.3 Create `src/lib/components/layout/AppShell.svelte` — CSS media query shell: sidebar at ≥768px, bottom nav at <768px, content area with proper margins
- [x] 2.4 Modify `src/routes/+layout.svelte` — replace top nav with AppShell, move sync logic into sidebar, preserve existing route content
- [x] 2.5 Test: layout switches between sidebar/bottom nav at breakpoint, all existing routes render inside shell

## Phase 3: Backend Foundation

- [x] 3.1 Create `src-tauri/src/repository/sqlite/migrations/010_community_schema.sql` — users, profiles, lessons, riffs, comments, streaks, follows, community_cache tables (additive only)
- [x] 3.2 Create `src-tauri/src/repository/sqlite/migrations/010_community_schema.down.sql` — DROP all community tables
- [x] 3.3 Create `src-tauri/src/services/auth_service.rs` — OAuth flow, JWT validation, token refresh, Tauri secure store integration
- [x] 3.4 Create `src-tauri/src/services/community_service.rs` — lesson/riff CRUD, feed aggregation, comment management, SQLite cache
- [x] 3.5 Create `src-tauri/src/services/profile_service.rs` — profile CRUD, streak tracking, gear list management
- [x] 3.6 Create `src-tauri/src/commands/auth_command.rs` — login, register, logout, refresh_token, get_current_user IPC commands
- [x] 3.7 Create `src-tauri/src/commands/community_command.rs` — get_feed, create_lesson, get_lesson, like_content, add_comment IPC commands
- [x] 3.8 Create `src-tauri/src/commands/profile_command.rs` — get_profile, update_profile, get_streak, add_gear_to_list IPC commands
- [x] 3.9 Create `src-tauri/capabilities/community.json` — network permissions (http:allow-fetch), file upload permissions, secure store access
- [x] 3.10 Test: `cargo test` passes, migration runs clean, commands register without error

## Phase 4: Auth Layer

- [x] 4.1 Create `src/lib/stores/auth.svelte.ts` — $state for user, token, serverReachable, login/logout actions, token refresh effect
- [x] 4.2 Create `src/lib/components/auth/LoginForm.svelte` — OAuth buttons (GitHub, Google), email/password fallback, loading/error states
- [x] 4.3 Create `src/lib/components/auth/AuthGuard.svelte` — wrapper component, redirects to login if unauthenticated, shows "Connect to enable" for offline state
- [x] 4.4 Modify `src/lib/styles/page.css` — migrate hardcoded colors to design tokens (--color-surface, --color-on-surface, etc.)
- [x] 4.5 Test: auth store state transitions, LoginForm renders, AuthGuard blocks/allows access

## Phase 5: Community Components + Stores

- [x] 5.1 Create `src/lib/stores/community.svelte.ts` — $state for feed items, lessons, riffs, loading states, pagination
- [x] 5.2 Create `src/lib/stores/profile.svelte.ts` — $state for current user profile, streak data, gear list
- [x] 5.3 Create `src/lib/components/community/FeedCard.svelte` — author avatar, content preview, like/comment counts, timestamp
- [x] 5.4 Create `src/lib/components/community/LessonCard.svelte` — thumbnail, title, difficulty chip, author, duration
- [x] 5.5 Create `src/lib/components/community/ProfileHeader.svelte` — avatar, display name, bio, streak counter, gear count, follow button
- [x] 5.6 Create `src/lib/components/community/StreakCounter.svelte` — current streak, longest streak, calendar heatmap
- [x] 5.7 Test: stores initialize correctly, components render with mock data, interactions fire callbacks

## Phase 6: Community Pages + Routing

- [x] 6.1 Create `src/routes/feed/+page.svelte` — infinite scroll feed, FeedCard list, pull-to-refresh
- [x] 6.2 Create `src/routes/explore/+page.svelte` — search/filter grid, category tabs, trending content
- [x] 6.3 Create `src/routes/lessons/+page.svelte` — lesson list with difficulty filter, LessonCard grid
- [x] 6.4 Create `src/routes/lessons/[id]/+page.svelte` — lesson detail, video embed, description, comments, related lessons
- [x] 6.5 Create `src/routes/profile/+page.svelte` — ProfileHeader, practice history, gear list, contribution count
- [x] 6.6 Create `src/routes/my-gear/+page.svelte` — user's gear list, add/remove, link to catalog items
- [x] 6.7 Create `src/routes/saved-riffs/+page.svelte` — saved riffs list, tablature preview, BPM/tuning metadata
- [x] 6.8 Test: all routes navigate correctly, pages render with mock data, community nav items show offline state

## Phase 7: Integration & Polish

- [x] 7.1 Wire auth store to Tauri commands — invoke login/register/logout, persist token to secure store
- [x] 7.2 Wire community store to Tauri commands — invoke get_feed, create_lesson, like_content
- [x] 7.3 Wire profile store to Tauri commands — invoke get_profile, update_streak
- [x] 7.4 Implement server health check — ping endpoint on app start, set serverReachable flag
- [x] 7.5 Implement offline degradation — community nav items disabled when server unreachable, local cache for previously fetched content
- [x] 7.6 Verify `make lint && make test` pass with no regressions
- [x] 7.7 Verify bundle size stays under 15MB
