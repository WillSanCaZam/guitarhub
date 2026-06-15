# Proposal: Community Hub Integration

## Intent

GuitarHub is currently a solo product aggregator. Users browse gear alone. The goal is to transform it into a community platform where guitarists learn, share, and connect — without sacrificing the offline-first core that defines the app. Community features are **optional** and require a server backend; the core app remains zero-server, zero-tracking.

## Scope

### In Scope
- "Acoustic Dark Modern" design system applied to entire app (tokens, typography, colors, layout)
- New navigation shell: sidebar (desktop) + bottom nav (mobile)
- Optional community module: activates only when server is available, degrades gracefully offline
- User authentication (OAuth/JWT) via Tauri commands → server
- User profiles with practice streaks, gear lists, and contribution history
- Content storage: lessons, riffs, videos — server-backed with local cache
- Social features: follows, comments, likes, content contributions
- Daily challenges and leaderboards
- New routes: Feed, Explore, Lessons, Profile, My Gear, Saved Riffs

### Out of Scope
- Real-time chat or DMs (future iteration)
- Video hosting (embed from YouTube/Vimeo, no self-hosted video)
- Content moderation system (manual moderation for MVP)
- Payment/subscription system
- Mobile apps (Tauri desktop only for now)

## Capabilities

### New Capabilities
- `community-auth`: User registration, login, OAuth providers, JWT token management
- `user-profiles`: Profile creation, practice streaks, gear lists, contribution history
- `content-management`: Lesson/riff upload, storage, retrieval, local caching
- `social-features`: Follows, comments, likes, feed aggregation
- `challenges-leaderboards`: Daily challenges, streak tracking, community leaderboards
- `design-system`: Acoustic Dark Modern tokens, typography, component library
- `adaptive-navigation`: Sidebar (desktop) + bottom nav (mobile) with route-aware active states

### Modified Capabilities
- `scrape-workflow`: May need content adapters for lesson/riff sources (future)
- `ui`: Complete redesign — all existing components migrate to new design system

## Approach

**Hybrid Architecture** — two layers, one app:

```
┌─────────────────────────────────────────────────┐
│                 GuitarHub App                    │
├──────────────────────┬──────────────────────────┤
│   OFFLINE CORE       │   COMMUNITY LAYER        │
│   (zero server)      │   (server required)      │
│                      │                          │
│  • Product catalog   │  • Auth (OAuth/JWT)      │
│  • Search/filter     │  • User profiles         │
│  • Collections       │  • Lessons/riffs/videos  │
│  • Wishlist          │  • Social (follow/like)   │
│  • Price tracking    │  • Challenges            │
│  • Local SQLite      │  • Server + local cache  │
└──────────────────────┴──────────────────────────┘
```

**Key principles:**
1. Core app works 100% offline — no community feature blocks catalog access
2. Community module lazy-loads only when server is reachable
3. Graceful degradation: community nav items show "Connect to enable" state
4. Server can be self-hosted or managed (user configures endpoint in Settings)
5. Local cache for community content — recent lessons/riffs available offline after first fetch

**Phased delivery:**
1. Design system + layout shell (tokens, sidebar/bottom nav)
2. Auth layer + user profiles
3. Content management (lessons, riffs)
4. Social features (feed, follows, comments)
5. Challenges + leaderboards

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src/lib/styles/` | Modified | New design tokens, typography, color system |
| `src/routes/+layout.svelte` | Modified | Sidebar/bottom nav replaces top navbar |
| `src/routes/` | Modified | New routes: feed, explore, lessons, profile, my-gear, saved-riffs |
| `src/lib/components/` | Modified | 11 existing components redesigned; ~15-20 new components |
| `src/lib/stores/` | Modified | New stores: user, lessons, riffs, social, auth |
| `src-tauri/src/commands/` | Modified | New commands for auth, profiles, content, social |
| `src-tauri/src/services/` | Modified | New services: AuthService, ProfileService, ContentService, SocialService |
| `src-tauri/src/repository/sqlite/` | Modified | New migrations for community tables |
| `src-tauri/capabilities/` | Modified | Permissions for file uploads, network access |
| `docs/design/community-hub/` | New | Design system reference (already exists) |
| `src/lib/components/community/` | New | Community-specific components |
| `src/lib/components/auth/` | New | Auth UI components |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Server dependency breaks offline-first ethos | High | Community features are explicitly optional; core works without server |
| Scope creep into real-time features | Medium | MVP excludes chat/DMs; define clear feature boundary |
| Design migration breaks existing UX | Medium | Phase migration last; keep old components until new ones are verified |
| Content moderation at scale | Low for MVP | Manual moderation; limit public content to verified users initially |
| GDPR/privacy with user profiles | Medium | Privacy-first: minimal data, clear consent, export/delete account |

## Rollback Plan

1. **Design rollback**: Git revert the design system commit; existing components are preserved in `src/lib/components/` until Phase 5
2. **Community module rollback**: Feature-flag the community module in `tauri.conf.json`; disable network commands if server issues arise
3. **Database rollback**: New migrations are additive only; rollback by dropping community tables (no existing data affected)
4. **Full rollback**: `git revert` to pre-community-hub commit; all changes are in additive commits

## Dependencies

- Server backend (can be self-hosted or third-party) — NOT included in this change, only the client integration
- OAuth provider credentials (GitHub, Google) for auth
- Hanken Grotesk + JetBrains Mono fonts (Google Fonts or self-hosted)

## Success Criteria

- [ ] App works fully offline with zero community features active
- [ ] Community features activate only when server endpoint is configured and reachable
- [ ] "Connect to enable" state shown for community nav items when offline
- [ ] Auth flow completes: register → login → profile created
- [ ] At least one content type (lessons) uploadable and viewable
- [ ] Design system matches "Acoustic Dark Modern" spec (tokens, typography, colors)
- [ ] `make lint && make test` pass with no regressions
- [ ] Bundle size stays under 15MB (desktop app)
