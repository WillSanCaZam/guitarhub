<script lang="ts">
  import StoreIcon from './StoreIcon.svelte'
  import type { StoreDef, Connection } from '$lib/types/stores'

  let {
    store,
    connection,
    onConnect,
    onDisconnect,
  }: {
    store: StoreDef
    connection: Connection | null
    onConnect: (storeId: string) => void
    onDisconnect: (storeId: string) => Promise<void>
  } = $props()

  let disconnecting = $state(false)
  let disconnectError = $state('')

  async function handleDisconnect() {
    disconnecting = true
    disconnectError = ''
    try {
      await onDisconnect(store.id)
    } catch (e) {
      disconnectError = String(e)
    } finally {
      disconnecting = false
    }
  }
</script>

<div class="store-card">
  <div class="card-header">
    <StoreIcon storeId={store.id} />
    <div class="store-info">
      <h3 class="store-name">{store.name}</h3>
      <a href={store.website} class="store-website" target="_blank" rel="noopener noreferrer">
        {store.website.replace('https://', '')}
      </a>
    </div>
  </div>

  <div class="card-body">
    {#if connection}
      {#if connection.username}
        <span class="status-badge connected">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor"><circle cx="12" cy="12" r="8"/></svg>
          Connected as @{connection.username}
        </span>
      {:else}
        <span class="status-badge connected">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor"><circle cx="12" cy="12" r="8"/></svg>
          Connected
        </span>
      {/if}

      {#if connection.synced_at}
        <p class="sync-info">Last synced: {new Date(connection.synced_at * 1000).toLocaleDateString()}</p>
      {:else}
        <p class="sync-info">Not yet synced</p>
      {/if}
    {:else}
      <span class="status-badge disconnected">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor"><circle cx="12" cy="12" r="8"/></svg>
        Disconnected
      </span>
    {/if}

    {#if disconnectError}
      <p class="error-msg">{disconnectError}</p>
    {/if}
  </div>

  <div class="card-footer">
    {#if connection}
      <button
        class="btn btn-disconnect"
        onclick={handleDisconnect}
        disabled={disconnecting}
      >
        {disconnecting ? 'Disconnecting...' : 'Disconnect'}
      </button>
    {:else}
      <button
        class="btn btn-connect"
        onclick={() => onConnect(store.id)}
      >
        Connect
      </button>
    {/if}
  </div>
</div>

<style>
  .store-card {
    background: var(--void-raised);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: var(--space-5);
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    transition: border-color var(--transition-fast), box-shadow var(--transition-fast);
  }

  .store-card:hover {
    border-color: var(--border-active);
    box-shadow: var(--shadow-card);
  }

  .card-header {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .store-info {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .store-name {
    margin: 0;
    font-family: var(--font-body);
    font-size: 1.1rem;
    font-weight: 700;
    color: var(--text-bright);
  }

  .store-website {
    font-size: 0.8rem;
    color: var(--text-dim);
    text-decoration: none;
    transition: color var(--transition-fast);
  }

  .store-website:hover {
    color: var(--glow-primary);
  }

  .card-body {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    flex: 1;
  }

  .status-badge {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    font-size: 0.85rem;
    font-weight: 500;
  }

  .status-badge svg {
    flex-shrink: 0;
  }

  .connected {
    color: var(--success);
  }

  .connected svg {
    color: var(--success);
  }

  .disconnected {
    color: var(--text-dim);
  }

  .disconnected svg {
    color: var(--text-muted);
  }

  .sync-info {
    margin: 0;
    font-size: 0.75rem;
    color: var(--text-dim);
  }

  .error-msg {
    margin: 0;
    font-size: 0.8rem;
    color: var(--danger);
  }

  .card-footer {
    display: flex;
    gap: var(--space-2);
  }

  .btn {
    flex: 1;
    padding: var(--space-2) var(--space-4);
    border: none;
    border-radius: var(--radius-sm);
    font-size: 0.85rem;
    font-weight: 600;
    cursor: pointer;
    transition: background var(--transition-fast), opacity var(--transition-fast);
  }

  .btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .btn-connect {
    background: var(--glow-primary);
    color: var(--void-deep);
  }

  .btn-connect:hover:not(:disabled) {
    background: var(--glow-warm);
  }

  .btn-disconnect {
    background: transparent;
    color: var(--danger);
    border: 1px solid var(--danger);
  }

  .btn-disconnect:hover:not(:disabled) {
    background: var(--glow-danger);
  }
</style>
