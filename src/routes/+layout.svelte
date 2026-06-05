<script>
  import { invoke } from '@tauri-apps/api/core';
  import { syncResult } from '$lib/stores/sync';

  let { children } = $props();
  let syncing = $state(false);
  let syncError = $state(null);

  async function handleSync() {
    syncing = true;
    syncError = null;
    try {
      const result = await invoke('sync_catalog', {
        url: 'https://pages.guitarhub.app/catalog.json'
      });
      syncResult.set(result);
    } catch (e) {
      syncError = String(e);
      setTimeout(() => { syncError = null; }, 5000);
    } finally {
      syncing = false;
    }
  }
</script>

<nav class="nav">
  <a href="/" class="nav-title">GuitarHub</a>
  <div class="nav-actions">
    <a href="#settings" class="nav-link">Settings</a>
    <button onclick={handleSync} disabled={syncing} class="sync-btn">
      {syncing ? 'Syncing\u2026' : 'Sync Catalog'}
    </button>
    {#if syncError}
      <span class="sync-error">{syncError}</span>
    {/if}
  </div>
</nav>

<div class="content">
  {@render children()}
</div>

<style>
  :global(body) {
    margin: 0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen,
      Ubuntu, Cantarell, sans-serif;
    background: #f5f5f5;
    color: #333;
  }
  .nav {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 16px;
    height: 48px;
    background: #1a1a2e;
    color: #fff;
    position: sticky;
    top: 0;
    z-index: 100;
  }
  .nav-title {
    color: #fff;
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
    gap: 12px;
  }
  .nav-link {
    color: #aaa;
    text-decoration: none;
    font-size: 0.85rem;
    transition: color 0.15s;
  }
  .nav-link:hover {
    color: #fff;
  }
  .sync-btn {
    padding: 4px 12px;
    border: 1px solid rgba(255,255,255,0.3);
    border-radius: 4px;
    background: transparent;
    color: #fff;
    cursor: pointer;
    font-size: 0.8rem;
    transition: background 0.15s;
  }
  .sync-btn:hover:not(:disabled) {
    background: rgba(255,255,255,0.1);
  }
  .sync-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .sync-error {
    color: #f88;
    font-size: 0.8rem;
  }
  .content {
    min-height: calc(100vh - 48px);
  }
</style>
