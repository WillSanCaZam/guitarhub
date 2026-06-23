<script>
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { fade } from 'svelte/transition';
  import { invoke } from '@tauri-apps/api/core';
  import { syncState } from '$lib/stores/sync.svelte';
  import { wishlistState, loadWishlist } from '$lib/stores/wishlist.svelte';
  import { authState } from '$lib/stores/auth.svelte';
  import { drawerState } from '$lib/stores/drawerState.svelte';
  import HealthCheck from '$lib/components/community/HealthCheck.svelte';
  import AppShell from '$lib/components/layout/AppShell.svelte';
  import '$lib/styles/design-tokens.css';
  import '$lib/styles/global.css';
  import '$lib/styles/animations.css';

  let { children } = $props();
  let syncing = $state(false);
  let syncError = $state(null);
  let catalogUrl = $state('https://pages.guitarhub.app/catalog.json');
  let currentPath = $derived($page.url.pathname);
  let drawer = drawerState();

  onMount(async () => {
    loadWishlist();
    try {
      const saved = await invoke('get_setting', { key: 'catalog_url' });
      if (saved) {
        catalogUrl = saved;
      } else {
        await invoke('save_setting', {
          key: 'catalog_url',
          value: 'https://pages.guitarhub.app/catalog.json'
        });
      }
    } catch (e) {
      // Fallback already set in initial state
    }
  });

  async function handleSync() {
    syncing = true;
    syncError = null;
    try {
      const result = await invoke('sync_catalog', { url: catalogUrl });
      Object.assign(syncState, result);
    } catch (e) {
      syncError = String(e);
      setTimeout(() => { syncError = null; }, 5000);
    } finally {
      syncing = false;
    }
  }

  // Auto-close drawer on route change
  $effect(() => {
    if (currentPath) {
      drawer.close();
    }
  });

  // Auto-close drawer on breakpoint crossing
  $effect(() => {
    const mql = matchMedia('(min-width: 768px)');
    function handleChange(e) {
      if (e.matches) {
        drawer.close();
      }
    }
    mql.addEventListener('change', handleChange);
    return () => mql.removeEventListener('change', handleChange);
  });
</script>

<HealthCheck />

<AppShell {currentPath} serverReachable={authState.serverReachable} syncing={syncing} onSync={handleSync} drawerOpen={drawer.open} ondrawerClose={drawer.close} ondrawerToggle={drawer.toggle}>
  {#key currentPath}
    <div class="page-transition" in:fade={{ duration: 150, delay: 50 }} out:fade={{ duration: 100 }}>
      {@render children()}
    </div>
  {/key}
</AppShell>

<style>
  :global(body) {
    margin: 0;
    font-family: var(--font-body);
    background: var(--void-deep);
    color: var(--text-warm);
  }

  .page-transition {
    min-height: 100%;
  }

  @media (prefers-reduced-motion: reduce) {
    .page-transition {
      animation: none !important;
    }
  }
</style>
