# Design: guitarhub-offcanvas-menu

## 1. Component Architecture

```
+layout.svelte
├── drawerOpen: boolean (state)
├── handleToggleDrawer()
├── handleCloseDrawer()
│
├── AppShell
│   ├── Sidebar (desktop only, ≥768px)
│   ├── DrawerOverlay (mobile only, conditional)
│   ├── DrawerPanel (mobile only, conditional)
│   │   └── Sidebar content (reused)
│   ├── main.content
│   └── BottomNav
│       └── HamburgerButton (mobile only)
```

### State Flow
```
+layout.svelte owns drawerOpen state
  ↓
AppShell receives: drawerOpen, onCloseDrawer
  ↓
BottomNav receives: drawerOpen, onToggleDrawer
  ↓
DrawerOverlay receives: onClick (close)
  ↓
DrawerPanel receives: drawerOpen, onCloseDrawer
```

## 2. State Management

### +layout.svelte
```svelte
<script>
  import { page } from '$app/stores';
  
  // Reactive currentPath — fixes stale bug
  let currentPath = $derived($page.url.pathname);
  
  // Drawer state
  let drawerOpen = $state(false);
  
  function handleToggleDrawer() {
    drawerOpen = !drawerOpen;
  }
  
  function handleCloseDrawer() {
    drawerOpen = false;
  }
  
  // Auto-close on route change
  $effect(() => {
    $page.url.pathname; // subscribe to changes
    drawerOpen = false;
  });
  
  // Auto-close on breakpoint crossing to desktop
  $effect(() => {
    const mql = window.matchMedia('(min-width: 768px)');
    function handleChange(e) {
      if (e.matches) drawerOpen = false;
    }
    mql.addEventListener('change', handleChange);
    return () => mql.removeEventListener('change', handleChange);
  });
</script>
```

### AppShell.svelte
```svelte
<script lang="ts">
  import Sidebar from './Sidebar.svelte';
  import BottomNav from './BottomNav.svelte';
  import DrawerOverlay from './DrawerOverlay.svelte';
  import DrawerPanel from './DrawerPanel.svelte';
  import type { Snippet } from 'svelte';

  interface Props {
    currentPath: string;
    serverReachable: boolean;
    syncing?: boolean;
    onSync?: () => void;
    drawerOpen: boolean;
    onToggleDrawer: () => void;
    onCloseDrawer: () => void;
    children: Snippet;
  }

  let { 
    currentPath, serverReachable, syncing = false, onSync,
    drawerOpen, onToggleDrawer, onCloseDrawer,
    children 
  }: Props = $props();
  
  // Focus trap refs
  let drawerPanel: HTMLElement;
  let previousFocus: HTMLElement | null = null;
  
  // Focus management
  $effect(() => {
    if (drawerOpen) {
      previousFocus = document.activeElement as HTMLElement;
      // Focus first focusable element in drawer
      requestAnimationFrame(() => {
        const firstFocusable = drawerPanel?.querySelector('a, button, [tabindex]');
        firstFocusable?.focus();
      });
    } else if (previousFocus) {
      previousFocus.focus();
      previousFocus = null;
    }
  });
</script>
```

## 3. CSS Architecture

### DrawerOverlay.svelte
```svelte
<div 
  class="drawer-overlay"
  class:visible={drawerOpen}
  onclick={onClose}
  aria-hidden="true"
></div>

<style>
  .drawer-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    z-index: var(--z-overlay);
    opacity: 0;
    pointer-events: none;
    transition: opacity 200ms var(--ease-fade);
  }
  
  .drawer-overlay.visible {
    opacity: 1;
    pointer-events: auto;
  }
</style>
```

### DrawerPanel.svelte
```svelte
<div 
  class="drawer-panel"
  class:open={drawerOpen}
  bind:this={drawerPanel}
  role="dialog"
  aria-modal="true"
  aria-label="Navigation menu"
  onkeydown={handleKeydown}
>
  <div class="drawer-header">
    <a href="/" class="logo" onclick={onClose}>
      <svg ...>...</svg>
      GuitarHub
    </a>
    <button class="close-btn" onclick={onClose} aria-label="Close menu">
      <svg ...>X</svg>
    </button>
  </div>
  
  <nav class="drawer-nav">
    <!-- Reuse navItems from Sidebar -->
  </nav>
  
  <div class="drawer-footer">
    <!-- Sync + Settings -->
  </div>
</div>

<style>
  .drawer-panel {
    position: fixed;
    top: 0;
    left: 0;
    width: 280px;
    max-width: 85vw;
    height: 100vh;
    background: var(--void-mid);
    border-right: 1px solid var(--color-outline-variant);
    z-index: var(--z-modal);
    transform: translateX(-100%);
    transition: transform 200ms var(--ease-plug);
    display: flex;
    flex-direction: column;
  }
  
  .drawer-panel.open {
    transform: translateX(0);
  }
  
  @media (prefers-reduced-motion: reduce) {
    .drawer-panel {
      transition: none;
    }
  }
</style>
```

## 4. Animation Strategy

### Keyframes (add to animations.css)
```css
@keyframes drawerSlideIn {
  from { transform: translateX(-100%); }
  to { transform: translateX(0); }
}

@keyframes drawerSlideOut {
  from { transform: translateX(0); }
  to { transform: translateX(-100%); }
}

@keyframes overlayFadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes overlayFadeOut {
  from { opacity: 1; }
  to { opacity: 0; }
}
```

### CSS Transitions (preferred over keyframes)
- Overlay: `opacity 200ms var(--ease-fade)`
- Panel: `transform 200ms var(--ease-plug)` — cubic-bezier(0.16, 1, 0.3, 1) for natural slide

### prefers-reduced-motion
```css
@media (prefers-reduced-motion: reduce) {
  .drawer-panel, .drawer-overlay {
    transition: none;
  }
}
```

## 5. Keyboard Handling

### Escape Key
```svelte
<script>
  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      onClose();
    }
    
    // Tab trap
    if (e.key === 'Tab') {
      const focusables = drawerPanel.querySelectorAll('a, button, [tabindex]:not([tabindex="-1"])');
      const first = focusables[0];
      const last = focusables[focusables.length - 1];
      
      if (e.shiftKey && document.activeElement === first) {
        e.preventDefault();
        last.focus();
      } else if (!e.shiftKey && document.activeElement === last) {
        e.preventDefault();
        first.focus();
      }
    }
  }
</script>
```

### Tauri Compatibility
- Escape key is NOT reserved by Tauri — safe to use
- Test on all 3 platforms (Linux/macOS/Windows)

## 6. Responsive Strategy

### Breakpoint Listener
```svelte
$effect(() => {
  const mql = window.matchMedia('(min-width: 768px)');
  function handleChange(e: MediaQueryListEvent) {
    if (e.matches) {
      // Crossed to desktop — close drawer
      drawerOpen = false;
    }
  }
  mql.addEventListener('change', handleChange);
  return () => mql.removeEventListener('change', handleChange);
});
```

### CSS Media Queries
```css
/* Drawer only visible on mobile */
@media (min-width: 768px) {
  .drawer-overlay, .drawer-panel {
    display: none;
  }
}

/* Hamburger only visible on mobile */
@media (min-width: 768px) {
  .hamburger-btn {
    display: none;
  }
}
```

## 7. Accessibility

### ARIA Attributes
| Element | Attribute | Value |
|---------|-----------|-------|
| Drawer panel | `role` | `dialog` |
| Drawer panel | `aria-modal` | `true` |
| Drawer panel | `aria-label` | `Navigation menu` |
| Hamburger button | `aria-expanded` | `{drawerOpen}` |
| Hamburger button | `aria-label` | `Toggle navigation menu` |
| Main content | `aria-hidden` | `{drawerOpen}` |
| Overlay | `aria-hidden` | `true` |

### Focus Management Lifecycle
1. **Drawer opens**: Save `document.activeElement`, focus first focusable in drawer
2. **While open**: Tab cycles through focusable elements (trap)
3. **Drawer closes**: Restore focus to saved element

### Screen Reader Announcements
- Drawer open: `aria-live="polite"` region announces "Navigation menu opened"
- Drawer close: announces "Navigation menu closed"

## 8. Testing Strategy

### Functional Tests
- [ ] Hamburger button visible only on mobile
- [ ] Click hamburger opens drawer
- [ ] Click overlay closes drawer
- [ ] Click close button closes drawer
- [ ] Escape key closes drawer
- [ ] Tab cycles through drawer elements
- [ ] Focus returns to hamburger on close
- [ ] Drawer closes on route navigation
- [ ] Drawer closes when viewport crosses 768px

### Accessibility Tests
- [ ] `role="dialog"` present on drawer
- [ ] `aria-modal="true"` present
- [ ] `aria-expanded` toggles on hamburger
- [ ] `aria-hidden` on main content when open
- [ ] Focus trap works correctly
- [ ] Screen reader announces open/close

### Visual Tests
- [ ] Slide animation smooth (200ms)
- [ ] Overlay fades in/out
- [ ] Reduced motion respected
- [ ] Drawer width correct (280px / 85vw)
- [ ] Z-index layering correct (overlay=300, modal=400)
