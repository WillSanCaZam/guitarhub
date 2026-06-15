# Exploration: Community Hub Integration

## Current State

GuitarHub is a **product aggregator** desktop app (Tauri 2 + Svelte 5 + SQLite) that scrapes guitar/amp/pedal listings from online stores into a unified catalog. The current UI is a **bento-grid dashboard** with:

- **Navigation**: Simple top navbar with Wishlist, Settings, and Sync Catalog
- **Pages**: Home (dashboard), Collection, Wishlist, Settings
- **Backend**: Tauri commands → services → SQLite repository (hexagonal architecture)
- **Styling**: System fonts, CSS custom properties, `prefers-color-scheme` dark mode
- **No community features**: No user profiles, no social interactions, no lessons, no user-generated content

The new design adds a **social learning platform** layer: user profiles, practice streaks, riff sharing, lesson library, and community interaction — all with a distinctive "Acoustic Dark Modern" aesthetic (amber accents, tonal layering, Hanken Grotesk + JetBrains Mono).

## Affected Areas

### Frontend (`src/`)
- `src/routes/+layout.svelte` — Navigation must change from top navbar to sidebar (desktop) + bottom nav (mobile)
- `src/routes/+page.svelte` — Home dashboard must be reimagined as feed
- `src/lib/styles/` — Complete overhaul: new design tokens, typography, color system
- `src/lib/components/` — All 11 existing components need redesign; ~15-20 new components needed
- `src/lib/stores/` — New stores for user profile, lessons, riffs, social features

### New Routes Needed
- `/feed` — Main community feed (current home redesign)
- `/explore` — Discovery/discovery grid
- `/lessons` — Lesson library
- `/lessons/[id]` — Lesson detail with video, materials, comments
- `/profile` — User profile with gear, riffs, streaks
- `/upload` — Content upload flow

### Backend (`src-tauri/`)
- `src-tauri/src/commands/` — New commands for user profiles, lessons, riffs, comments, follows
- `src-tauri/src/services/` — New services for community logic
- `src-tauri/src/domain/` — New domain models (User, Lesson, Riff, Comment, PracticeStreak)
- `src-tauri/src/repository/sqlite/` — New migrations for community tables
- `src-tauri/capabilities/` — May need new permissions for file uploads

### Design System
- `docs/design/` — Design tokens already defined in DESIGN.md; need to create `tokens.json`
- New component library required (15-20 components)

## Approaches

### Approach 1: Full Integration (Recommended)

**Description**: Integrate community hub as a first-class module within the existing GuitarHub app. The current product aggregator becomes one "tab" (Gear/Catalog) alongside Feed, Explore, Lessons, and Profile.

| Aspect | Detail |
|--------|--------|
| **Pros** | Single app, single codebase, shared design system, users get everything in one place |
| **Cons** | Larger scope, more complex navigation, must migrate existing UI |
| **Effort** | High (6-8 weeks for full implementation) |

**Architecture**:
```
GuitarHub App
├── Gear (existing catalog/aggregator) ← current home becomes a tab
├── Feed (new) ← community content feed
├── Explore (new) ← discovery/search for riffs, lessons, creators
├── Lessons (new) ← structured learning content
└── Profile (new) ← user profile, gear, practice streaks
```

### Approach 2: Modular Extraction

**Description**: Extract the catalog aggregator into a separate "Gear" module and build the community hub as the new primary shell. The two modules communicate via Tauri IPC.

| Aspect | Detail |
|--------|--------|
| **Pros** | Clean separation, easier to maintain independently, can ship community hub first |
| **Cons** | Duplication of design system, navigation, and shared utilities; more complex state management |
| **Effort** | Medium-High (5-7 weeks) |

### Approach 3: Gradual Migration (Phased)

**Description**: Start with the community hub as a separate route/page within the existing app, then gradually migrate the catalog features into the new design system over multiple sprints.

| Aspect | Detail |
|--------|--------|
| **Pros** | Lowest risk, incremental value delivery, easier to review |
| **Cons** | Two design systems coexist temporarily, user experience inconsistency during transition |
| **Effort** | Medium (4-6 weeks, but spread across phases) |

## Recommendation

**Approach 1: Full Integration** is recommended because:

1. **Design Cohesion**: The design kit already specifies a complete design system that replaces the current one. Having two design systems (current light + new dark) would be confusing.
2. **Navigation**: The design shows a unified navigation (sidebar on desktop, bottom nav on mobile) that includes both catalog and community features.
3. **Existing Architecture**: The hexagonal architecture (commands → services → repository) scales well to new domain models.
4. **User Experience**: GuitarHub should feel like one app, not two apps bolted together.

**Migration Strategy**: Phase the work as:
1. **Phase 1**: Create design system (tokens, typography, colors) + new layout shell (sidebar/bottom nav)
2. **Phase 2**: Build community backend (user profiles, lessons, riffs, comments)
3. **Phase 3**: Build frontend pages (feed, explore, lessons, profile)
4. **Phase 4**: Migrate existing catalog features into new design system
5. **Phase 5**: Polish, test, and ship

## Risks

1. **Scope Creep**: Community features are complex (auth, content moderation, media uploads). Define MVP clearly.
2. **Backend Complexity**: User accounts, auth, content storage — this is a significant backend addition to an offline-first app. Consider if community features require a server (violates "zero server costs" constraint).
3. **Design Migration**: Existing components (ProductCard, PriceChart, FilterBar) need complete redesign, not just reskinning.
4. **Performance**: Community features (feeds, images, videos) may impact the "offline-first" and "< 15MB" constraints.
5. **Legal/Privacy**: User profiles and social features raise GDPR/privacy concerns not present in the current catalog-only app.

## Ready for Proposal

**Yes** — the design kit is comprehensive, the codebase architecture supports extension, and the migration path is clear. The orchestrator should proceed with proposal creation, but MUST address the **server dependency question** first: community features (user accounts, content storage, social interactions) typically require a backend server, which conflicts with GuitarHub's "zero server costs" constraint.

**Key question for the user**: Should community features be:
- **Local-only** (user data stored locally, no social features beyond personal profiles/streaks)?
- **P2P** (peer-to-peer sharing via Tauri, no central server)?
- **Server-backed** (requires a lightweight server for auth and content storage)?

This decision fundamentally affects the architecture and implementation scope.
