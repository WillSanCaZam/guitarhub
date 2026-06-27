<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'
  import { onMount } from 'svelte'
  import { storeDefs } from '$lib/data/store-defs'
  import StoresGrid from '$lib/components/stores/StoresGrid.svelte'
  import ConnectModal from '$lib/components/stores/ConnectModal.svelte'
  import type { Connection } from '$lib/types/stores'

  let connections = $state<Connection[]>([])
  let loading = $state(true)
  let error = $state<string | null>(null)
  let connectingStoreId = $state<string | null>(null)

  onMount(() => {
    loadConnections()
  })

  async function loadConnections() {
    loading = true
    error = null
    try {
      connections = await invoke<Connection[]>('list_connections')
    } catch (e) {
      error = String(e)
    } finally {
      loading = false
    }
  }

  function handleConnect(storeId: string) {
    connectingStoreId = storeId
  }

  function handleCloseModal() {
    connectingStoreId = null
  }

  async function handleConnected(connection: Connection) {
    connectingStoreId = null
    // Refresh the connections list
    await loadConnections()
  }

  async function handleDisconnect(storeId: string) {
    await invoke('disconnect_store', { storeId })
    await loadConnections()
  }

  let activeStore = $derived(
    connectingStoreId ? storeDefs.find((s) => s.id === connectingStoreId) ?? null : null,
  )
</script>

<div class="stores-page">
  <header class="page-header">
    <a href="/" class="back-link">← Dashboard</a>
    <h1>Connected Stores</h1>
    <p class="subtitle">Connect your store accounts to see your own listings in the catalog</p>
  </header>

  <main class="page-content">
    {#if loading}
      <div class="loading-state">
        <p>Loading connections...</p>
      </div>
    {:else if error}
      <div class="error-banner" role="alert">
        Failed to load connections: {error}
      </div>
      <StoresGrid
        storeDefs={storeDefs}
        connections={[]}
        onConnect={handleConnect}
        onDisconnect={handleDisconnect}
      />
    {:else}
      <StoresGrid
        {storeDefs}
        {connections}
        onConnect={handleConnect}
        onDisconnect={handleDisconnect}
      />
    {/if}
  </main>
</div>

{#if activeStore}
  <ConnectModal
    store={activeStore}
    onClose={handleCloseModal}
    onConnected={handleConnected}
  />
{/if}

<style>
  .stores-page {
    max-width: 960px;
    margin: 0 auto;
    padding: var(--space-6);
    min-height: 100vh;
    display: flex;
    flex-direction: column;
  }

  .page-header {
    margin-bottom: var(--space-8);
  }

  .back-link {
    display: inline-block;
    margin-bottom: var(--space-2);
    color: var(--text-dim);
    text-decoration: none;
    font-size: 0.85rem;
    transition: color var(--transition-fast);
  }

  .back-link:hover {
    color: var(--glow-primary);
  }

  h1 {
    margin: 0 0 var(--space-2);
    font-family: var(--font-display);
    font-size: 1.75rem;
    font-weight: 700;
    color: var(--text-bright);
  }

  .subtitle {
    margin: 0;
    color: var(--text-dim);
    font-size: 0.95rem;
  }

  .page-content {
    flex: 1;
  }

  .loading-state {
    text-align: center;
    padding: var(--space-10) 0;
    color: var(--text-dim);
  }

  .error-banner {
    padding: var(--space-3) var(--space-4);
    background: var(--glow-danger);
    color: var(--danger);
    border-radius: var(--radius-sm);
    margin-bottom: var(--space-5);
    font-size: 0.9rem;
  }
</style>
