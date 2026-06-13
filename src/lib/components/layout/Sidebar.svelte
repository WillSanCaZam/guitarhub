<script lang="ts">
  interface Props {
    currentPath: string;
    serverReachable: boolean;
  }

  let { currentPath, serverReachable }: Props = $props();

  const navItems = [
    { path: '/feed', label: 'Feed', icon: '📡', community: true },
    { path: '/explore', label: 'Explore', icon: '🔍', community: true },
    { path: '/lessons', label: 'Lessons', icon: '🎓', community: true },
    { path: '/', label: 'My Gear', icon: '🎸', community: false },
    { path: '/saved-riffs', label: 'Saved Riffs', icon: '🎵', community: true },
    { path: '/profile', label: 'Profile', icon: '👤', community: true },
  ];

  function isActive(path: string): boolean {
    if (path === '/') return currentPath === '/';
    return currentPath.startsWith(path);
  }
</script>

<aside class="sidebar">
  <div class="sidebar-header">
    <a href="/" class="logo">🎸 GuitarHub</a>
  </div>

  <nav class="sidebar-nav">
    {#each navItems as item}
      <a
        href={item.path}
        class="nav-item"
        class:active={isActive(item.path)}
        aria-current={isActive(item.path) ? 'page' : undefined}
      >
        <span class="nav-icon">{item.icon}</span>
        <span class="nav-label">{item.label}</span>
        {#if item.community && !serverReachable}
          <span class="offline-badge" title="Connect to enable">OFF</span>
        {/if}
      </a>
    {/each}
  </nav>

  <div class="sidebar-footer">
    <a href="/settings" class="nav-item settings-link">
      <span class="nav-icon">⚙️</span>
      <span class="nav-label">Settings</span>
    </a>
  </div>
</aside>

<style>
  .sidebar {
    width: 240px;
    height: 100vh;
    background: var(--color-surface-container-low, #161620);
    border-right: 1px solid var(--color-outline-variant, #2a2a38);
    display: flex;
    flex-direction: column;
    position: fixed;
    top: 0;
    left: 0;
    z-index: var(--z-sticky, 200);
  }

  .sidebar-header {
    padding: var(--spacing-lg, 24px) var(--spacing-md, 16px);
    border-bottom: 1px solid var(--color-outline-variant, #2a2a38);
  }

  .logo {
    color: var(--color-on-surface, #e8e8f0);
    text-decoration: none;
    font-family: var(--font-sans, system-ui, sans-serif);
    font-size: 1.25rem;
    font-weight: 700;
  }

  .sidebar-nav {
    flex: 1;
    padding: var(--spacing-sm, 8px);
    display: flex;
    flex-direction: column;
    gap: 2px;
    overflow-y: auto;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm, 8px);
    padding: var(--spacing-sm, 8px) var(--spacing-md, 16px);
    border-radius: var(--radius-md, 8px);
    color: var(--color-on-surface-variant, #a0a0b0);
    text-decoration: none;
    font-family: var(--font-sans, system-ui, sans-serif);
    font-size: 0.875rem;
    font-weight: 400;
    transition: background var(--transition-fast, 100ms ease),
                color var(--transition-fast, 100ms ease);
  }

  .nav-item:hover {
    background: var(--color-surface-container, #1c1c26);
    color: var(--color-on-surface, #e8e8f0);
  }

  .nav-item.active {
    background: var(--color-primary-container, #3d3520);
    color: var(--color-on-primary-container, #f5e6b8);
    font-weight: 500;
  }

  .nav-icon {
    font-size: 1.1rem;
    width: 24px;
    text-align: center;
    flex-shrink: 0;
  }

  .nav-label {
    flex: 1;
  }

  .offline-badge {
    font-size: 0.5625rem;
    font-family: var(--font-mono, monospace);
    font-weight: 600;
    padding: 1px 4px;
    border-radius: var(--radius-xs, 2px);
    background: var(--color-warning-container, #3d2a10);
    color: var(--color-on-warning-container, #ffcc80);
    letter-spacing: 0.05em;
  }

  .sidebar-footer {
    padding: var(--spacing-sm, 8px);
    border-top: 1px solid var(--color-outline-variant, #2a2a38);
  }

  .settings-link {
    color: var(--color-on-surface-muted, #666680);
  }
</style>
