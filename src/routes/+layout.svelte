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

<AppShell {currentPath} serverReachable={authState.serverReachable}>
  <div class="legacy-header">
    <a href="/" class="nav-title">GuitarHub</a>
    <div class="nav-actions">
      <a href="/wishlist" class="nav-link">
        Wishlist{#if $wishlistState.items.length > 0}
          <span class="badge">{$wishlistState.items.length}</span>
        {/if}
      </a>
      <a href="/settings" class="nav-link">Settings</a>
      <button onclick={handleSync} disabled={syncing} class="sync-btn" data-testid="sync-button">
        {syncing ? 'Syncing\u2026' : 'Sync Catalog'}
      </button>
      {#if syncError}
        <span class="sync-error">{syncError}</span>
      {/if}
    </div>
  </div>

  {@render children()}
</AppShell>

<style>
  :global(body) {
    margin: 0;
    font-family: var(--font-sans, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen,
      Ubuntu, Cantarell, sans-serif);
    background: var(--color-surface, #121218);
    color: var(--color-on-surface, #e8e8f0);
  }
  .legacy-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 var(--spacing-md, 16px);
    height: 48px;
    background: var(--color-surface-container, #1c1c26);
    color: var(--color-on-surface, #e8e8f0);
  }
  .nav-title {
    color: var(--color-on-surface, #fff);
    text-decoration: none;
    font-size: 1.1rem;
    font-weight: 700;
  }
  .nav-title:hover {
    opacity: 0.85;
  }
  .nav-actions {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm, 12px);
  }
  .nav-link {
    color: var(--color-on-surface-variant, #aaa);
    text-decoration: none;
    font-size: 0.85rem;
    transition: color var(--transition-fast, 0.15s);
  }
  .nav-link:hover {
    color: var(--color-on-surface, #fff);
  }
  .badge {
    display: inline-block;
    min-width: 18px;
    height: 18px;
    line-height: 18px;
    padding: 0 5px;
    border-radius: 9px;
    background: var(--color-secondary, #4a90d9);
    color: var(--color-on-secondary, #fff);
    font-size: 0.7rem;
    font-weight: 600;
    text-align: center;
    margin-left: var(--spacing-2xs, 4px);
    vertical-align: middle;
  }
  .sync-btn {
    padding: var(--spacing-2xs, 4px) var(--spacing-sm, 12px);
    border: 1px solid var(--color-outline, rgba(255,255,255,0.3));
    border-radius: var(--radius-sm, 4px);
    background: transparent;
    color: var(--color-on-surface, #fff);
    cursor: pointer;
    font-size: 0.8rem;
    transition: background var(--transition-fast, 0.15s);
  }
  .sync-btn:hover:not(:disabled) {
    background: var(--color-surface-container-high, rgba(255,255,255,0.1));
  }
  .sync-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .sync-error {
    color: var(--color-error, #f88);
    font-size: 0.8rem;
  }
</style>
