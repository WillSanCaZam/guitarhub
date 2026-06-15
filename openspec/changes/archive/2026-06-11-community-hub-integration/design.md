# Design: Community Hub Integration

## Technical Approach

Hybrid architecture: offline-first core (zero server) + optional community module (server-backed with local cache). Design system "Acoustic Dark Modern" applied globally. All new state uses Svelte 5 runes. Backend follows existing command → service → repository pattern. Community features activate only when server endpoint is reachable.

## Architecture Decisions

| Decision | Option A | Option B | Option C | Tradeoff | Decision |
|----------|----------|----------|----------|----------|----------|
| State management | Svelte 5 runes ($state) | Legacy writable stores | Zustand | A: native, tree-shakeable; B: deprecated; C: extra dep | **A — runes** |
| Sync strategy | CRDT (PowerSync) | Simple last-write-wins + local cache | REST polling | A: complex, heavy dep; B: simpler,足够 for MVP; C: stale data | **B — local cache + LWW** |
| Auth token storage | Tauri secure store | SQLite settings table | localStorage | A: OS keychain, secure; B: readable from DB; C: XSS risk | **A — Tauri secure store** |
| Component architecture | Atomic design (atoms/molecules/organisms) | Flat component dir | Feature-based dirs | A: proven, matches design system; B: unscalable; C: hard to share | **A — atomic** |
| Navigation shell | CSS media query (sidebar/bottom nav) | JS viewport detection | Tauri window mode | A: pure CSS, no JS overhead; B: reactive; C: Tauri API coupling | **A — CSS media query** |
| Community DB | Separate SQLite file | Additive migrations to existing DB | In-memory only | A: isolation; B: simpler, matches existing pattern; C: lost on restart | **B — additive migrations** |
| Content cache | SQLite cache table | Filesystem cache | Both | A: queryable, transactional; B: large files; C: complexity | **A — SQLite cache** |

## Data Flow

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   Svelte 5   │────▶│  Tauri Cmds  │────▶│   Services   │
│   Runes      │     │  (IPC glue)  │     │  (business)  │
│   Stores     │◀────│              │◀────│              │
└──────────────┘     └──────────────┘     └──────┬───────┘
                                                 │
                                          ┌──────▼───────┐
                                          │  Repository  │
                                          │  (SQLite)    │
                                          └──────┬───────┘
                                                 │
                              ┌───────────────────┼───────────────────┐
                              │                   │                   │
                       ┌──────▼───────┐   ┌──────▼───────┐   ┌──────▼───────┐
                       │  Local DB    │   │  HTTP Client │   │  Secure Store│
                       │  (offline)   │   │  (server)    │   │  (JWT keys)  │
                       └──────────────┘   └──────────────┘   └──────────────┘
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `src/lib/styles/tokens.css` | Create | Acoustic Dark Modern CSS custom properties |
| `src/lib/styles/typography.css` | Create | Hanken Grotesk + JetBrains Mono font faces + type scale |
| `src/lib/components/layout/AppShell.svelte` | Create | Sidebar (desktop) + bottom nav (mobile) wrapper |
| `src/lib/components/layout/Sidebar.svelte` | Create | Desktop navigation sidebar |
| `src/lib/components/layout/BottomNav.svelte` | Create | Mobile bottom navigation |
| `src/lib/components/ui/Button.svelte` | Create | Design system button (primary/secondary/ghost) |
| `src/lib/components/ui/Card.svelte` | Create | Design system card (tonal layering) |
| `src/lib/components/ui/Avatar.svelte` | Create | User avatar with fallback |
| `src/lib/components/ui/Badge.svelte` | Create | Status/role badges |
| `src/lib/components/ui/Chip.svelte` | Create | Genre/difficulty tags |
| `src/lib/components/ui/Input.svelte` | Create | Dark-themed input with amber focus |
| `src/lib/components/ui/ProgressBar.svelte` | Create | Amber progress indicator |
| `src/lib/components/community/FeedCard.svelte` | Create | Community feed item |
| `src/lib/components/community/LessonCard.svelte` | Create | Lesson preview card |
| `src/lib/components/community/ProfileHeader.svelte` | Create | User profile header |
| `src/lib/components/community/StreakCounter.svelte` | Create | Practice streak display |
| `src/lib/components/auth/LoginForm.svelte` | Create | OAuth/JWT login form |
| `src/lib/components/auth/AuthGuard.svelte` | Create | Route guard for authenticated content |
| `src/lib/stores/auth.svelte.ts` | Create | Auth state (user, token, serverReachable) |
| `src/lib/stores/community.svelte.ts` | Create | Feed, lessons, riffs state |
| `src/lib/stores/profile.svelte.ts` | Create | User profile + streaks state |
| `src/lib/types/community.ts` | Create | TypeScript interfaces for community entities |
| `src/routes/feed/+page.svelte` | Create | Feed route |
| `src/routes/explore/+page.svelte` | Create | Explore route |
| `src/routes/lessons/+page.svelte` | Create | Lessons route |
| `src/routes/lessons/[id]/+page.svelte` | Create | Lesson detail route |
| `src/routes/profile/+page.svelte` | Create | Profile route |
| `src/routes/my-gear/+page.svelte` | Create | My Gear route |
| `src/routes/saved-riffs/+page.svelte` | Create | Saved Riffs route |
| `src/routes/+layout.svelte` | Modify | Replace top nav with AppShell |
| `src/lib/styles/page.css` | Modify | Migrate to design tokens |
| `src-tauri/src/commands/auth_command.rs` | Create | Login, register, token refresh commands |
| `src-tauri/src/commands/community_command.rs` | Create | Feed, lessons, riffs, comments commands |
| `src-tauri/src/commands/profile_command.rs` | Create | Profile CRUD, streak commands |
| `src-tauri/src/services/auth_service.rs` | Create | OAuth flow, JWT management |
| `src-tauri/src/services/community_service.rs` | Create | Content CRUD, feed aggregation |
| `src-tauri/src/services/profile_service.rs` | Create | Profile + streak logic |
| `src-tauri/src/repository/sqlite/migrations/010_community_schema.sql` | Create | Users, profiles, lessons, riffs, comments, streaks, follows tables |
| `src-tauri/src/repository/sqlite/migrations/010_community_schema.down.sql` | Create | Rollback |
| `src-tauri/capabilities/community.json` | Create | Network + file upload permissions |

## Interfaces / Contracts

```rust
// Community entities (Rust)
pub struct UserProfile {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub gear_list: Vec<String>,  // SKUs
    pub streak_days: u32,
    pub joined_at: i64,
}

pub struct Lesson {
    pub id: String,
    pub author_id: String,
    pub title: String,
    pub description: String,
    pub content_url: String,     // embed URL (YouTube/Vimeo)
    pub difficulty: String,      // beginner/intermediate/advanced
    pub tags: Vec<String>,
    pub likes: u32,
    pub created_at: i64,
}

pub struct Riff {
    pub id: String,
    pub author_id: String,
    pub title: String,
    pub tablature: String,       // tab notation
    pub bpm: u32,
    pub tuning: String,
    pub tags: Vec<String>,
    pub likes: u32,
    pub created_at: i64,
}

pub struct FeedItem {
    pub id: String,
    pub author: UserProfile,
    pub item_type: String,       // lesson/riff/comment
    pub content: serde_json::Value,
    pub created_at: i64,
}
```

```typescript
// TypeScript interfaces (frontend)
interface UserProfile {
  id: string
  username: string
  displayName: string
  avatarUrl?: string
  bio?: string
  gearList: string[]
  streakDays: number
  joinedAt: number
}

interface Lesson {
  id: string
  authorId: string
  title: string
  description: string
  contentUrl: string
  difficulty: 'beginner' | 'intermediate' | 'advanced'
  tags: string[]
  likes: number
  createdAt: number
}
```

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit | Rust services (auth, community, profile) | In-memory SQLite pool, mock HTTP client |
| Unit | Svelte stores (runes state) | Vitest + Svelte component testing |
| Unit | Command handlers | Separate core logic from Tauri IPC, test with `memory_pool()` |
| Integration | Auth flow (register → login → token) | Mock server + real Tauri commands |
| Integration | Content CRUD (lesson create → fetch) | Mock server + SQLite |
| Integration | Offline degradation | Mock unreachable server, verify graceful fallback |
| E2E | Full community flow | tauri-driver: login → create lesson → view feed → like |
| E2E | Design system tokens | Visual regression (screenshots) |

## Migration / Rollout

**Phased delivery (5 phases):**

1. **Design system + layout shell** — tokens.css, typography, AppShell, Sidebar, BottomNav. All existing routes wrapped in new shell. No functional changes.
2. **Auth layer** — auth_command, auth_service, secure store, LoginForm. Feature-flag: community nav shows "Connect to enable" when server unreachable.
3. **Content management** — lessons, riffs CRUD. Local cache in SQLite. Server-side storage.
4. **Social features** — feed, follows, comments, likes. Feed aggregation from followed users.
5. **Challenges + leaderboards** — streak tracking, daily challenges, community rankings.

**Database migration**: Additive only (010_community_schema.sql). No existing tables modified. Rollback drops community tables.

**Feature flag**: `community.enabled` in settings. Controls whether community nav items are interactive or show disabled state.

**Server endpoint**: User configures in Settings. Default: none (offline mode). Community features activate on successful health check.

## Open Questions

- [ ] Server backend ownership: self-hosted vs. managed? (Affects auth flow complexity)
- [ ] Rate limiting strategy for community content uploads
- [ ] Content moderation approach for public lessons/riffs (manual vs. automated)
- [ ] Avatar storage: server-side or external URL (e.g., Gravatar)?
- [ ] Should streaks persist offline and sync when reconnected?
