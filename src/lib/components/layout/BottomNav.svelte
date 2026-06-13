<script lang="ts">
  interface Props {
    currentPath: string;
    serverReachable: boolean;
  }

  let { currentPath, serverReachable }: Props = $props();

  const navItems = [
    { path: '/feed', label: 'Feed', icon: '📡' },
    { path: '/explore', label: 'Explore', icon: '🔍' },
    { path: '/', label: 'My Gear', icon: '🎸' },
    { path: '/saved-riffs', label: 'Riffs', icon: '🎵' },
    { path: '/profile', label: 'Profile', icon: '👤' },
  ];

  function isActive(path: string): boolean {
    if (path === '/') return currentPath === '/';
    return currentPath.startsWith(path);
  }
</script>

<nav class="bottom-nav" aria-label="Mobile navigation">
  {#each navItems as item}
    <a
      href={item.path}
      class="nav-item"
      class:active={isActive(item.path)}
      aria-current={isActive(item.path) ? 'page' : undefined}
    >
      <span class="nav-icon">{item.icon}</span>
      <span class="nav-label">{item.label}</span>
    </a>
  {/each}
</nav>

<style>
  .bottom-nav {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    height: 64px;
    background: var(--color-surface-container, #1c1c26);
    border-top: 1px solid var(--color-outline-variant, #2a2a38);
    display: flex;
    align-items: center;
    justify-content: space-around;
    z-index: var(--z-sticky, 200);
    padding: 0 var(--spacing-xs, 4px);
  }

  .nav-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    padding: var(--spacing-xs, 4px) var(--spacing-sm, 8px);
    border-radius: var(--radius-md, 8px);
    color: var(--color-on-surface-variant, #a0a0b0);
    text-decoration: none;
    font-family: var(--font-sans, system-ui, sans-serif);
    font-size: 0.625rem;
    font-weight: 400;
    transition: color var(--transition-fast, 100ms ease);
    min-width: 48px;
    min-height: 48px;
    justify-content: center;
  }

  .nav-item:hover {
    color: var(--color-on-surface, #e8e8f0);
  }

  .nav-item.active {
    color: var(--color-primary, #d4a017);
  }

  .nav-item.active .nav-icon {
    transform: scale(1.1);
  }

  .nav-icon {
    font-size: 1.25rem;
    line-height: 1;
  }

  .nav-label {
    line-height: 1;
  }
</style>
