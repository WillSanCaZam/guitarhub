<script lang="ts">
  import DashboardCell from './DashboardCell.svelte';
  import type { CollectionStats, CollectionItem } from '$lib/types/collection';
  import { calculateCollectionGainLoss, formatGainLoss } from '$lib/utils/collectionValue';

  interface Props {
    stats: CollectionStats | null;
    items: CollectionItem[];
    loading: boolean;
  }

  let { stats, items, loading }: Props = $props();

  let gainLossFormatted = $derived(formatGainLoss(calculateCollectionGainLoss(items)));
</script>

<a href="/collection" class="cell-link">
  <DashboardCell
    title="Collection"
    icon="🎸"
    {loading}
    empty={!stats || stats.total_items === 0}
    emptyMessage="Start adding gear to track your collection value"
    emptyIcon="🎸"
  >
    {#if stats && stats.total_items > 0}
      <div class="collection-stats">
        <div class="stat">
          <span class="stat-value">{stats.total_items}</span>
          <span class="stat-label">items</span>
        </div>
        <div class="stat">
          <span class="stat-value">${stats.total_value.toFixed(0)}</span>
          <span class="stat-label">total value</span>
        </div>
        <div class="stat">
          <span class="stat-value gain-loss-{gainLossFormatted.colorClass}">{gainLossFormatted.text}</span>
          <span class="stat-label">gain/loss</span>
        </div>
        {#if stats.top_item_name}
          <div class="top-item">
            Top: {stats.top_item_name} (${stats.top_item_value.toFixed(0)})
          </div>
        {/if}
      </div>
    {/if}
  </DashboardCell>
</a>

<style>
  .cell-link {
    display: block;
    text-decoration: none;
    color: inherit;
  }

  .collection-stats {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .top-item {
    font-size: 0.8rem;
    color: var(--color-on-surface-muted);
    margin-top: 4px;
  }

  .gain-loss-gain {
    color: var(--color-success);
  }

  .gain-loss-loss {
    color: var(--color-error);
  }

  .gain-loss-neutral {
    color: var(--color-on-surface-muted);
  }

  @media (prefers-color-scheme: dark) {
    .top-item {
      color: var(--color-on-surface-variant);
    }

    .gain-loss-gain {
      color: var(--color-success);
    }

    .gain-loss-loss {
      color: var(--color-error);
    }

    .gain-loss-neutral {
      color: var(--color-on-surface-variant);
    }
  }
</style>
