# Proposal: Off-Canvas Drawer Menu for Mobile Navigation

## Intent

Mobile users have no hamburger menu — the BottomNav shows only 3 route links. Desktop Sidebar is hidden on mobile with no replacement. Additionally, `currentPath` in `+layout.svelte` is set once in `onMount` and never updated on navigation, causing stale active-state highlighting. This change fixes the bug and introduces a proper off-canvas drawer menu for mobile (<768px).

## Scope

### In Scope
- Fix `currentPath` bug: replace `onMount` set-once with reactive `$page.url.pathname` from `$app/stores`
- Create off-canvas drawer menu component with slide-in panel + backdrop overlay
- Add hamburger button trigger in BottomNav for mobile
- Focus management: trap focus inside drawer when open, restore on close
- Keyboard handling: Escape to close, Tab/Shift+Tab cycling
- Auto-close on route change and on viewport crossing to desktop (>=768px)
- Animation: `transform: translateX()` slide, respects `prefers-reduced-motion`

### Out of Scope
- Swipe-to-open/close gestures (deferred to v2)
- Desktop sidebar changes (unchanged)
- New navigation items or reorganization of existing links
- ARIA live region announcements beyond basic `aria-label`/`role`

## Capabilities

### New Capabilities
- `mobile-drawer-menu`: Off-canvas drawer panel, backdrop overlay, hamburger trigger, focus trap, keyboard nav, responsive auto-close

### Modified Capabilities
- None — no existing spec covers mobile navigation behavior

## Approach

1. **Fix currentPath**: Replace `onMount(() => { currentPath = ... })` in `+layout.svelte` with reactive `$: currentPath = $page.url.pathname` (Svelte 5 rune: `$derived($page.url.pathname)`).
2. **Drawer state**: New `$state` rune in a `drawerState.svelte.ts` store (`open: boolean`, `close()`, `toggle()`).
3. **AppShell**: Add drawer overlay (`position: fixed`, `z-index: var(--z-overlay)`) and slide panel (`z-index: var(--z-modal)`, `transform: translateX(-100%)` → `translateX(0)` on open). Overflow hidden on `.app-shell` wrapper when open (NOT `document.body`).
4. **BottomNav**: Add hamburger `<button>` visible only `<768px`, calls `drawerState.toggle()`.
5. **Focus trap**: `onMount`-based trap cycling Tab within drawer panel. `Escape` key listener. On close, restore focus to trigger button.
6. **Auto-close**: `$effect` watching `$page.url.pathname` → close drawer. MatchMedia listener for `min-width: 768px` → close drawer.
7. **Animation**: CSS transitions on `transform`. `prefers-reduced-motion: reduce` → instant toggle (no animation).
8. **Accessibility**: `role="dialog"`, `aria-modal="true"`, `aria-label="Navigation menu"`, focus-visible ring on interactive elements.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src/routes/+layout.svelte` | Modified | Fix currentPath reactivity, import drawer state |
| `src/lib/components/layout/AppShell.svelte` | Modified | Add drawer overlay + slide panel, overflow control |
| `src/lib/components/layout/BottomNav.svelte` | Modified | Add hamburger button trigger |
| `src/lib/components/layout/DrawerMenu.svelte` | New | Off-canvas drawer component (backdrop + panel + focus trap) |
| `src/lib/stores/drawerState.svelte.ts` | New | Drawer open/close state with runes |
| `src/lib/styles/animations.css` | Modified | Add drawer slide keyframes if needed |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Focus trap breaks tab navigation for screen readers | Medium | Test with VoiceOver/NVDA; use `inert` on backdrop content; follow WAI-ARIA dialog pattern |
| Tauri webview Escape key conflicts | Low | Investigation confirmed Tauri doesn't reserve Escape; test on all 3 platforms |
| Animation jank on low-end devices | Low | Use `transform` only (GPU-composited); `prefers-reduced-motion` disables; no `will-change` |
| Overflow:hidden on .app-shell causes clipping | Low | Apply only when drawer is open; remove on close; test nested scrollable content |

## Rollback Plan

1. Revert `+layout.svelte` to `onMount` currentPath (or keep the fix — it's a bugfix independent of drawer)
2. Remove `DrawerMenu.svelte` and `drawerState.svelte.ts`
3. Remove hamburger button from `BottomNav.svelte`
4. Revert `AppShell.svelte` to original (remove overlay + panel markup)
5. Run `make test && make lint` — all green

## Dependencies

- None new. Uses existing: `$app/stores`, CSS custom properties (`--z-overlay`, `--z-modal`), Svelte 5 runes.

## Success Criteria

- [ ] `currentPath` updates reactively on every navigation (no stale active state)
- [ ] Hamburger button visible on viewports <768px, hidden on >=768px
- [ ] Drawer slides in from left with backdrop overlay on hamburger click
- [ ] Escape key closes drawer; focus returns to hamburger button
- [ ] Tab key cycles only through drawer elements when open
- [ ] Route change auto-closes drawer
- [ ] Resizing from mobile to desktop auto-closes drawer
- [ ] `prefers-reduced-motion: reduce` disables slide animation
- [ ] `make test && make lint` pass without errors
- [ ] No `document.body` overflow manipulation — `.app-shell` only
