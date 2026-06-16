<script lang="ts">
  import { wishlistState } from '$lib/stores/wishlist.svelte';

  interface Props {
    currentPath: string;
    serverReachable: boolean;
    syncing?: boolean;
    onSync?: () => void;
  }

  let { currentPath, serverReachable, syncing = false, onSync }: Props = $props();

  const navItems = [
    { path: '/explore', label: 'Buscar', icon: 'search', community: true },
    { path: '/collection', label: 'Colección', icon: 'collection', community: false },
    { path: '/', label: 'My Gear', icon: 'guitar', community: false },
    { path: '/wishlist', label: 'Wishlist', icon: 'heart', community: false, badge: wishlistState.items.length },
    { path: '/feed', label: 'Feed', icon: 'feed', community: true },
    { path: '/lessons', label: 'Lessons', icon: 'lessons', community: true },
    { path: '/saved-riffs', label: 'Saved Riffs', icon: 'riffs', community: true },
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
    };
    return icons[name] || '';
  }
</script>

<aside class="sidebar">
  <div class="sidebar-header">
    <a href="/" class="logo">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="logo-icon"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>
      GuitarHub
    </a>
  </div>

  <nav class="sidebar-nav">
    {#each navItems as item}
      <a
        href={item.path}
        class="nav-item"
        class:active={isActive(item.path)}
        aria-current={isActive(item.path) ? 'page' : undefined}
      >
        <span class="nav-icon">{@html getIcon(item.icon)}</span>
        <span class="nav-label">{item.label}</span>
        {#if item.badge}
          <span class="badge">{item.badge}</span>
        {/if}
        {#if item.community && !serverReachable}
          <span class="offline-badge" title="Connect to enable">OFF</span>
        {/if}
      </a>
    {/each}
  </nav>

  <div class="sidebar-footer">
    <button class="nav-item sync-button" onclick={onSync} disabled={syncing}>
      <span class="nav-icon">{@html getIcon('sync')}</span>
      <span class="nav-label">{syncing ? 'Syncing…' : 'Sync'}</span>
    </button>
    <a href="/settings" class="nav-item settings-link">
      <span class="nav-icon">{@html getIcon('profile')}</span>
      <span class="nav-label">Settings</span>
    </a>
  </div>
</aside>

<style>
  .sidebar {
    width: 240px;
    height: 100vh;
    background: var(--void-mid);
    border-right: 1px solid var(--color-outline-variant);
    display: flex;
    flex-direction: column;
    position: fixed;
    top: 0;
    left: 0;
    z-index: var(--z-sticky);
  }

  .sidebar-header {
    padding: var(--space-lg) var(--space-md);
    border-bottom: 1px solid var(--color-outline-variant);
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
  }

  .logo-icon {
    width: 24px;
    height: 24px;
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

  .sync-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
