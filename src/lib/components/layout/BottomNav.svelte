<script lang="ts">
  interface Props {
    currentPath: string;
    serverReachable: boolean;
    drawerOpen: boolean;
    ondrawerClose: () => void;
    ondrawerToggle: () => void;
  }

  let { currentPath, serverReachable, drawerOpen, ondrawerClose, ondrawerToggle }: Props = $props();

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
  <button
    class="hamburger"
    aria-expanded={drawerOpen}
    aria-label="Toggle navigation menu"
    onclick={ondrawerToggle}
  >
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M3 12h18M3 6h18M3 18h18" />
    </svg>
  </button>
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

  .hamburger {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 44px;
    height: 44px;
    background: transparent;
    border: none;
    cursor: pointer;
    color: var(--text-warm);
    border-radius: var(--radius-md);
    transition: color var(--transition-fast);
  }

  .hamburger:hover {
    color: var(--text-bright);
  }

  .hamburger svg {
    width: 24px;
    height: 24px;
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

  /* Hide hamburger on desktop */
  @media (min-width: 768px) {
    .hamburger {
      display: none;
    }
  }
</style>
