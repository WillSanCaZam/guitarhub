<script lang="ts">
  import { wishlistState } from '$lib/stores/wishlist.svelte';

  interface Props {
    currentPath: string;
    serverReachable: boolean;
    syncing?: boolean;
    collapsed?: boolean;
    onSync?: () => void;
    onToggleCollapse?: () => void;
  }

  let {
    currentPath,
    serverReachable,
    syncing = false,
    collapsed = false,
    onSync,
    onToggleCollapse,
  }: Props = $props();

  const navItems = [
    { path: '/explore', label: 'Buscar', icon: 'search', community: true },
    { path: '/collection', label: 'Colección', icon: 'collection', community: false },
    { path: '/', label: 'My Gear', icon: 'guitar', community: false },
    { path: '/wishlist', label: 'Wishlist', icon: 'heart', community: false, badge: wishlistState.items.length },
    { path: '/feed', label: 'Feed', icon: 'feed', community: true },
    { path: '/lessons', label: 'Lessons', icon: 'lessons', community: true, badge: 'off' },
    { path: '/saved-riffs', label: 'Saved Riffs', icon: 'riffs', community: true, badge: 'off' },
    { path: '/profile', label: 'Profile', icon: 'profile', community: true },
  ];

  function isActive(path: string): boolean {
    if (path === '/') return currentPath === '/';
    return currentPath.startsWith(path);
  }

  function getIcon(name: string): string {
    const icons: Record<string, string> = {
      search: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/></svg>',
      collection: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/></svg>',
      guitar: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>',
      heart: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/></svg>',
      feed: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 11a9 9 0 0 1 9 9"/><path d="M4 4a16 16 0 0 1 16 16"/><circle cx="5" cy="19" r="1"/></svg>',
      lessons: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 10v6M2 10l10-5 10 5-10 5z"/><path d="M6 12v5c3 3 9 3 12 0v-5"/></svg>',
      riffs: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>',
      profile: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>',
      sync: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/></svg>',
      chevron: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="15 18 9 12 15 6"/></svg>',
    };
    return icons[name] || '';
  }
</script>

<aside class="sidebar" class:collapsed data-testid="sidebar">
  <div class="sidebar-header">
    <a href="/" class="logo" title={collapsed ? 'GuitarHub' : undefined}>
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="logo-icon"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>
      {#if !collapsed}
        <span class="logo-text">GuitarHub</span>
      {/if}
    </a>
    <button
      class="collapse-toggle"
      onclick={onToggleCollapse}
      aria-label={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
      title={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
    >
      <span class="collapse-icon" class:rotated={!collapsed}>{@html getIcon('chevron')}</span>
    </button>
  </div>

  <nav class="sidebar-nav">
    {#each navItems as item}
      {@const isOff = item.badge === 'off'}
      {@const active = isActive(item.path)}
      <a
        href={isOff ? undefined : item.path}
        class="nav-item"
        class:active
        class:disabled={isOff}
        aria-current={active ? 'page' : undefined}
        aria-disabled={isOff || undefined}
        tabindex={isOff ? -1 : 0}
        title={collapsed ? item.label : undefined}
      >
        <span class="active-indicator" class:visible={active}></span>
        <span class="nav-icon">{@html getIcon(item.icon)}</span>
        {#if !collapsed}
          <span class="nav-label">{item.label}</span>
        {/if}
        {#if !collapsed && item.badge && !isOff}
          <span class="badge">{item.badge}</span>
        {/if}
        {#if !collapsed && isOff}
          <span class="off-badge">PRÓXIMO</span>
        {/if}
        {#if !collapsed && item.community && !serverReachable}
          <span class="offline-badge" title="Connect to enable">OFF</span>
        {/if}
      </a>
    {/each}
  </nav>

  <div class="sidebar-footer">
    <button class="nav-item sync-button" onclick={onSync} disabled={syncing} title={collapsed ? (syncing ? 'Syncing…' : 'Sync') : undefined}>
      <span class="nav-icon">{@html getIcon('sync')}</span>
      {#if !collapsed}
        <span class="nav-label">{syncing ? 'Syncing…' : 'Sync'}</span>
      {/if}
    </button>
    <a href="/settings" class="nav-item settings-link" title={collapsed ? 'Settings' : undefined}>
      <span class="nav-icon">{@html getIcon('profile')}</span>
      {#if !collapsed}
        <span class="nav-label">Settings</span>
      {/if}
    </a>
  </div>
</aside>

<style>
  .sidebar {
    width: var(--sidebar-expanded);
    height: 100vh;
    background: var(--void-mid);
    border-right: 1px solid var(--color-outline-variant);
    display: flex;
    flex-direction: column;
    position: fixed;
    top: 0;
    left: 0;
    z-index: var(--z-sticky);
    transition: width var(--sidebar-transition);
    overflow: hidden;
  }

  .sidebar.collapsed {
    width: var(--sidebar-collapsed);
  }

  @media (prefers-reduced-motion: reduce) {
    .sidebar {
      transition: none;
    }
  }

  .sidebar-header {
    padding: var(--space-lg) var(--space-md);
    border-bottom: 1px solid var(--color-outline-variant);
    display: flex;
    align-items: center;
    justify-content: space-between;
    min-height: 64px;
  }

  .sidebar.collapsed .sidebar-header {
    padding: var(--space-lg) var(--space-sm);
    justify-content: center;
  }

  .logo {
    color: var(--color-primary);
    text-decoration: none;
    font-family: var(--font-display);
    font-size: 1.25rem;
    font-weight: 800;
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    overflow: hidden;
    white-space: nowrap;
  }

  .logo-icon {
    width: 24px;
    height: 24px;
    flex-shrink: 0;
  }

  .logo-text {
    opacity: 1;
    transition: opacity 150ms var(--ease-fade);
  }

  .sidebar.collapsed .logo-text {
    opacity: 0;
    width: 0;
    overflow: hidden;
  }

  .collapse-toggle {
    background: transparent;
    border: none;
    cursor: pointer;
    padding: var(--space-1);
    border-radius: var(--radius-sm);
    color: var(--text-dim);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 150ms var(--ease-snap), color 150ms var(--ease-snap);
    flex-shrink: 0;
  }

  .collapse-toggle:hover {
    background: var(--void-hover);
    color: var(--text-bright);
  }

  .collapse-icon {
    width: 16px;
    height: 16px;
    display: flex;
    transition: transform 200ms var(--ease-plug);
  }

  .collapse-icon :global(svg) {
    width: 100%;
    height: 100%;
  }

  .collapse-icon.rotated {
    transform: rotate(180deg);
  }

  @media (prefers-reduced-motion: reduce) {
    .collapse-icon {
      transition: none;
    }
  }

  .sidebar-nav {
    flex: 1;
    padding: var(--spacing-sm);
    display: flex;
    flex-direction: column;
    gap: 2px;
    overflow-y: auto;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    border-radius: var(--radius-md);
    color: var(--color-on-surface-variant);
    text-decoration: none;
    font-family: var(--font-sans);
    font-size: 0.875rem;
    font-weight: 400;
    transition: background var(--transition-fast),
                color var(--transition-fast);
    position: relative;
    overflow: hidden;
  }

  .sidebar.collapsed .nav-item {
    justify-content: center;
    padding: var(--spacing-sm);
  }

  .nav-item:hover {
    background: var(--color-surface-container);
    color: var(--color-on-surface);
  }

  .nav-item.active {
    background: var(--color-primary-container);
    color: var(--color-on-primary-container);
    font-weight: 500;
  }

  .nav-item.disabled {
    opacity: 0.5;
    cursor: not-allowed;
    pointer-events: none;
  }

  .active-indicator {
    position: absolute;
    left: 0;
    top: 20%;
    bottom: 20%;
    width: 3px;
    border-radius: 0 2px 2px 0;
    background: var(--glow-primary);
    opacity: 0;
    transform: scaleY(0);
    transition: opacity 200ms var(--ease-plug), transform 200ms var(--ease-plug);
  }

  .active-indicator.visible {
    opacity: 1;
    transform: scaleY(1);
  }

  @media (prefers-reduced-motion: reduce) {
    .active-indicator {
      transition: none;
    }
  }

  .nav-icon {
    width: 20px;
    height: 20px;
    flex-shrink: 0;
  }

  .nav-icon :global(svg) {
    width: 100%;
    height: 100%;
  }

  .nav-label {
    flex: 1;
    opacity: 1;
    transition: opacity 150ms var(--ease-fade);
    overflow: hidden;
    white-space: nowrap;
  }

  .sidebar.collapsed .nav-label {
    opacity: 0;
    width: 0;
    overflow: hidden;
  }

  .offline-badge {
    font-size: 0.5625rem;
    font-family: var(--font-mono);
    font-weight: 600;
    padding: 1px 4px;
    border-radius: var(--radius-xs);
    background: var(--color-warning-container);
    color: var(--color-on-warning-container);
    letter-spacing: 0.05em;
    opacity: 1;
    transition: opacity 150ms var(--ease-fade);
  }

  .sidebar.collapsed .offline-badge {
    opacity: 0;
    width: 0;
    overflow: hidden;
  }

  .badge {
    font-size: 0.625rem;
    font-family: var(--font-mono);
    font-weight: 600;
    padding: 1px 5px;
    border-radius: var(--radius-pill);
    background: var(--color-primary);
    color: var(--color-on-primary);
    min-width: 18px;
    text-align: center;
    opacity: 1;
    transition: opacity 150ms var(--ease-fade);
  }

  .sidebar.collapsed .badge {
    opacity: 0;
    width: 0;
    overflow: hidden;
  }

  .off-badge {
    font-size: 0.5rem;
    font-family: var(--font-mono);
    font-weight: 700;
    padding: 1px 4px;
    border-radius: var(--radius-xs);
    background: var(--void-active);
    color: var(--text-dim);
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }

  .sidebar-footer {
    padding: var(--spacing-sm);
    border-top: 1px solid var(--color-outline-variant);
  }

  .settings-link {
    color: var(--color-on-surface-muted);
  }

  .sync-button {
    width: 100%;
    background: transparent;
    border: none;
    cursor: pointer;
    text-align: left;
  }

  .sidebar.collapsed .sync-button {
    text-align: center;
  }

  .sync-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
