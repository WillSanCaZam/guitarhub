<script lang="ts">
  import { collectionState, removeFromCollection } from '$lib/stores/collection.svelte';
  import type { CollectionItem } from '$lib/types/collection';

  function gain(item: CollectionItem): number {
    const ev = item.estimated_value ?? 0;
    const pp = item.purchase_price ?? 0;
    return ev - pp;
  }

  function pct(item: CollectionItem): number {
    const pp = item.purchase_price ?? 0;
    if (pp === 0) return 0;
    const g = gain(item);
    return (g / pp) * 100;
  }

  async function handleRemove(id: number) {
    if (!confirm('Remove this item from your collection?')) return;
    await removeFromCollection(id);
  }
</script>

{#if collectionState.loading}
  <div class="loading-state" aria-busy="true">
    <span class="spinner"></span>
    <span>Loading collection...</span>
  </div>
{:else if collectionState.items.length === 0}
  <div class="empty-state">
    <p>Your collection is empty.</p>
    <p class="empty-hint">Search for gear to add!</p>
  </div>
{:else}
  <div class="collection-grid">
    {#each collectionState.items as item (item.id)}
      <div class="collection-card">
        <div class="card-header">
          <h4 class="item-name">{item.name}</h4>
          {#if item.brand}
            <p class="item-brand">{item.brand}</p>
          {/if}
        </div>
        <div class="card-body">
          <div class="price-row">
            <span class="price-label">Purchase</span>
            <span class="price-value">
              {#if item.purchase_price != null}
                ${item.purchase_price.toFixed(2)}
              {:else}
                —
              {/if}
            </span>
          </div>
          <div class="price-row">
            <span class="price-label">Estimated</span>
            <span class="price-value">
              {#if item.estimated_value != null}
                ${item.estimated_value.toFixed(2)}
              {:else}
                —
              {/if}
            </span>
          </div>
          <div class="price-row">
            <span class="price-label">Gain / Loss</span>
            <span class="gain-loss {gain(item) >= 0 ? 'gain' : 'loss'}">
              {gain(item) >= 0 ? '+' : ''}{gain(item).toFixed(2)}
              ({pct(item).toFixed(1)}%)
            </span>
          </div>
        </div>
        <div class="card-footer">
          <button class="remove-btn" onclick={() => handleRemove(item.id)}>
            Remove
          </button>
        </div>
      </div>
    {/each}
  </div>
{/if}

<style>
  .loading-state {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    padding: 48px 0;
    color: var(--color-on-surface-muted);
  }

  .spinner {
    width: 20px;
    height: 20px;
    border: 2px solid var(--color-outline);
    border-top-color: var(--color-secondary);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .empty-state {
    text-align: center;
    padding: 48px 0;
    color: var(--color-on-surface-muted);
  }

  .empty-state p {
    margin: 0 0 8px;
    font-size: 1.1rem;
  }

  .empty-hint {
    font-size: 0.9rem;
    color: var(--color-on-surface-variant);
  }

  .collection-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: var(--spacing-md);
  }

  .collection-card {
    background: rgba(255, 255, 255, 0.55);
    backdrop-filter: blur(12px);
    border-radius: var(--radius-lg);
    padding: var(--spacing-md);
    border: 1px solid rgba(255, 255, 255, 0.25);
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.05);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-3);
    transition: transform var(--transition-base), box-shadow var(--transition-base);
  }

  .collection-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 8px 16px rgba(0, 0, 0, 0.1);
  }

  .card-header {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .item-name {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--color-secondary);
  }

  .item-brand {
    margin: 0;
    font-size: 0.85rem;
    color: var(--color-on-surface-muted);
  }

  .card-body {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .price-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.9rem;
  }

  .price-label {
    color: var(--color-on-surface-muted);
  }

  .price-value {
    font-weight: 600;
    color: var(--color-secondary);
  }

  .gain-loss {
    font-weight: 600;
  }

  .gain-loss.gain {
    color: var(--color-success);
  }

  .gain-loss.loss {
    color: var(--color-error);
  }

  .card-footer {
    margin-top: auto;
  }

  .remove-btn {
    width: 100%;
    padding: 8px 12px;
    background: transparent;
    color: var(--color-error);
    border: 1px solid var(--color-error);
    border-radius: var(--radius-md);
    font-size: 0.85rem;
    cursor: pointer;
    transition: background var(--transition-fast), color var(--transition-fast);
  }

  .remove-btn:hover {
    background: var(--color-error);
    color: var(--color-on-surface);
  }

  @media (prefers-color-scheme: dark) {
    .collection-card {
      background: rgba(20, 20, 30, 0.55);
      border-color: rgba(255, 255, 255, 0.1);
      box-shadow: 0 4px 6px rgba(0, 0, 0, 0.25);
    }

    .item-name {
      color: var(--color-on-surface);
    }

    .price-value {
      color: var(--color-on-surface);
    }

    .gain-loss.gain {
      color: var(--color-success);
    }

    .gain-loss.loss {
      color: var(--color-error);
    }

    .remove-btn {
      color: var(--color-error);
      border-color: var(--color-error);
    }

    .remove-btn:hover {
      background: var(--color-error);
      color: var(--color-secondary);
    }

    .spinner {
      border-color: var(--color-outline-variant);
      border-top-color: var(--color-on-surface);
    }

    .loading-state {
      color: var(--color-on-surface-variant);
    }

    .empty-state {
      color: var(--color-on-surface-variant);
    }

    .empty-hint {
      color: var(--color-on-surface-muted);
    }
  }
</style>
