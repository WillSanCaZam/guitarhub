<script>
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { syncState } from '$lib/stores/sync.svelte';
  import { wishlistState, loadWishlist } from '$lib/stores/wishlist.svelte';
  import { authState } from '$lib/stores/auth.svelte';
  import HealthCheck from '$lib/components/community/HealthCheck.svelte';
  import AppShell from '$lib/components/layout/AppShell.svelte';

  let { children } = $props();
  let syncing = $state(false);
  let syncError = $state(null);
  let catalogUrl = $state('https://pages.guitarhub.app/catalog.json');
  let currentPath = $state('/');

  onMount(async () => {
    loadWishlist();
    currentPath = window.location.pathname;
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
</script>

<HealthCheck />

<AppShell {currentPath} serverReachable={authState.serverReachable} syncing={syncing} onSync={handleSync}>
  {@render children()}
</AppShell>

<style>
  :global(body) {
    margin: 0;
    font-family: var(--font-body);
    background: var(--void-deep);
    color: var(--text-warm);
  }
</style>
