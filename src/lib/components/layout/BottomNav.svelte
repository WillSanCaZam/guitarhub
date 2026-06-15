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
    background: var(--void-raised);
    border-top: 1px solid var(--color-outline-variant);
    display: flex;
    align-items: center;
    justify-content: space-around;
    z-index: var(--z-sticky);
    padding: 0 var(--space-xs);
  }

  .nav-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    padding: var(--space-xs) var(--space-sm);
    border-radius: var(--radius-md);
    color: var(--text-warm);
    text-decoration: none;
    font-family: var(--font-body);
    font-size: 0.625rem;
    font-weight: 400;
    transition: color var(--transition-fast);
    min-width: 48px;
    min-height: 48px;
    justify-content: center;
  }

  .nav-item:hover {
    color: var(--text-bright);
  }

  .nav-item.active {
    color: var(--glow-primary);
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
