<script lang="ts">
  import DashboardCell from './DashboardCell.svelte';

  interface PriceDrop {
    sku: string;
    previous_price: number;
    new_price: number;
    channel: string;
    reason: string;
  }

  interface Props {
    drops: PriceDrop[];
    dropsSent: number;
    syncState: string;
  }

  let { drops, dropsSent, syncState }: Props = $props();
</script>

<DashboardCell
  title="Sync Status"
  icon="🔄"
  loading={false}
  empty={drops.length === 0}
  emptyMessage="Sync catalog to see price drops"
  emptyIcon="🔄"
>
  {#if drops.length > 0}
    <div class="sync-toast">
      {drops.length} price drop(s) detected
      {#if dropsSent > 0}
        , {dropsSent} sent
      {/if}
    </div>
    <ul class="drop-list">
      {#each drops.slice(0, 3) as drop (drop.sku)}
        <li class="drop-item">
          <span class="drop-sku">{drop.sku}</span>
          <span class="drop-price">
            ${drop.previous_price.toFixed(2)} → ${drop.new_price.toFixed(2)}
          </span>
          <span class="drop-reason">{drop.reason}</span>
        </li>
      {/each}
    </ul>
  {:else}
    <p class="sync-idle">No recent price drops. Last sync: {syncState}</p>
  {/if}
</DashboardCell>

<style>
  .sync-toast {
    padding: 8px 12px;
    background: var(--color-success-container);
    color: var(--color-success);
    border: 1px solid var(--color-success-container);
    border-radius: 6px;
    margin-bottom: 10px;
    font-size: 0.9rem;
  }

  .drop-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .drop-item {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 0.85rem;
    padding: 6px 8px;
    background: rgba(0, 0, 0, 0.03);
    border-radius: 6px;
  }

  .drop-sku {
    font-weight: 600;
    color: var(--color-secondary);
  }

  .drop-price {
    color: var(--color-success);
    font-family: monospace;
  }

  .drop-reason {
    color: var(--color-on-surface-muted);
    font-size: 0.8rem;
  }

  .sync-idle {
    margin: 0;
    color: var(--color-on-surface-muted);
    font-size: 0.9rem;
  }

  @media (prefers-color-scheme: dark) {
    .drop-item {
      background: rgba(255, 255, 255, 0.05);
    }

    .drop-sku {
      color: var(--color-on-surface);
    }

    .sync-idle {
      color: var(--color-on-surface-variant);
    }
  }

  @media (max-width: 768px) {
    .drop-item {
      padding: 10px 8px;
      min-height: 44px;
    }
  }
</style>
