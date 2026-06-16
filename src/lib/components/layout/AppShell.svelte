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
    ondrawerClose: () => void;
    ondrawerToggle: () => void;
    children: Snippet;
  }

  let { currentPath, serverReachable, syncing = false, onSync, drawerOpen, ondrawerClose, ondrawerToggle, children }: Props = $props();
</script>

<div class="app-shell">
  <!-- Desktop: Sidebar (hidden on mobile via CSS) -->
  <div class="sidebar-container">
    <Sidebar {currentPath} {serverReachable} {syncing} {onSync} />
  </div>

  <!-- Content area -->
  <main class="content" aria-hidden={drawerOpen} inert={drawerOpen}>
    {@render children()}
  </main>

  <!-- Mobile: Bottom Nav (hidden on desktop via CSS) -->
  <div class="bottomnav-container">
    <BottomNav {currentPath} {serverReachable} {drawerOpen} ondrawerClose={ondrawerClose} ondrawerToggle={ondrawerToggle} />
  </div>

  <!-- Drawer -->
  <DrawerOverlay open={drawerOpen} onclose={ondrawerClose} />
  <DrawerPanel
    open={drawerOpen}
    {currentPath}
    {serverReachable}
    {syncing}
    {onSync}
    onclose={ondrawerClose}
  />
</div>

<style>
  .app-shell {
    display: flex;
    min-height: 100vh;
  }

  .sidebar-container {
    display: none;
  }

  .content {
    flex: 1;
    min-height: 100vh;
    padding-bottom: 0;
  }

  .bottomnav-container {
    display: block;
  }

  /* Desktop: ≥768px */
  @media (min-width: 768px) {
    .sidebar-container {
      display: block;
    }

    .content {
      margin-left: 240px;
      padding-bottom: 0;
    }

    .bottomnav-container {
      display: none;
    }
  }

  /* Mobile: <768px — add bottom padding for bottom nav */
  @media (max-width: 767px) {
    .content {
      padding-bottom: 64px;
    }
  }
</style>
