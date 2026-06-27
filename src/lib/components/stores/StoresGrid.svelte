<script lang="ts">
  import type { StoreDef, Connection } from '$lib/types/stores'
  import StoreCard from './StoreCard.svelte'

  let {
    storeDefs,
    connections,
    onConnect,
    onDisconnect,
  }: {
    storeDefs: StoreDef[]
    connections: Connection[]
    onConnect: (storeId: string) => void
    onDisconnect: (storeId: string) => Promise<void>
  } = $props()

  function getConnection(storeId: string): Connection | null {
    return connections.find((c) => c.store_id === storeId) ?? null
  }
</script>

<div class="stores-grid">
  {#each storeDefs as store (store.id)}
    <StoreCard
      {store}
      connection={getConnection(store.id)}
      {onConnect}
      {onDisconnect}
    />
  {/each}
</div>

{#if storeDefs.length === 0}
  <p class="empty-state">No stores available yet.</p>
{/if}

<style>
  .stores-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: var(--space-5);
  }

  .empty-state {
    text-align: center;
    color: var(--text-dim);
    font-size: 1rem;
    padding: var(--space-10) 0;
  }
</style>
