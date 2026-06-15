<script lang="ts">
  import { onMount } from 'svelte';
  import CollectionView from '$lib/components/CollectionView.svelte';
  import { collectionState, loadCollection, loadCollectionStats } from '$lib/stores/collection.svelte';

  let viewMode = $state<'grid' | 'list'>('grid');

  onMount(() => {
    loadCollection();
    loadCollectionStats();
  });

  function exportCSV() {
    const headers = ['Name', 'Brand', 'Purchase Price', 'Currency', 'Condition', 'Notes'];
    const rows = collectionState.items.map(item => [
      item.name,
      item.brand ?? '',
      item.purchase_price?.toFixed(2) ?? '',
      item.purchase_currency,
      item.condition,
      item.notes ?? ''
    ]);
    const csv = [headers.join(','), ...rows.map(r => r.join(','))].join('\n');
    const blob = new Blob([csv], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'guitarhub-collection.csv';
    a.click();
    URL.revokeObjectURL(url);
  }
</script>

<div class="page">
  <header class="collection-header">
    <a href="/" class="back-link">← Dashboard</a>
    <div class="header-content">
      <h1>My Collection</h1>
      {#if collectionState.stats}
        <div class="stats-bar">
          <span class="stat">{collectionState.stats.total_items} items</span>
          <span class="stat-sep">·</span>
          <span class="stat">{collectionState.stats.total_value.toLocaleString()} USD</span>
        </div>
      {/if}
    </div>
    <div class="actions">
      <div class="toggle-group">
        <button
          class="toggle-btn {viewMode === 'grid' ? 'active' : ''}"
          onclick={() => viewMode = 'grid'}
          aria-label="Grid view"
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <rect x="1" y="1" width="6" height="6" rx="1"/>
            <rect x="9" y="1" width="6" height="6" rx="1"/>
            <rect x="1" y="9" width="6" height="6" rx="1"/>
            <rect x="9" y="9" width="6" height="6" rx="1"/>
          </svg>
        </button>
        <button
          class="toggle-btn {viewMode === 'list' ? 'active' : ''}"
          onclick={() => viewMode = 'list'}
          aria-label="List view"
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <rect x="1" y="2" width="14" height="3" rx="1"/>
            <rect x="1" y="7" width="14" height="3" rx="1"/>
            <rect x="1" y="12" width="14" height="3" rx="1"/>
          </svg>
        </button>
      </div>
      <button class="export-btn" onclick={exportCSV}>
        Export CSV
      </button>
    </div>
  </header>

  <CollectionView {viewMode} />
</div>

<style>
  .page {
    max-width: 1200px;
    margin: 0 auto;
    padding: var(--spacing-lg);
  }

  .collection-header {
    margin-bottom: var(--spacing-xl);
  }

  .back-link {
    display: inline-block;
    margin-bottom: var(--spacing-sm);
    color: var(--color-on-surface-muted);
    text-decoration: none;
    font-size: 0.85rem;
    transition: color var(--transition-fast);
  }

  .back-link:hover {
    color: var(--color-amber);
  }

  .header-content {
    display: flex;
    align-items: baseline;
    gap: var(--spacing-md);
    flex-wrap: wrap;
  }

  h1 {
    margin: 0;
    font-family: var(--font-display);
    font-size: 1.75rem;
    font-weight: 700;
    color: var(--color-on-surface);
  }

  .stats-bar {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    font-size: 0.9rem;
    color: var(--color-on-surface-muted);
  }

  .stat-sep {
    color: var(--color-outline);
  }

  .actions {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    margin-top: var(--spacing-md);
  }

  .toggle-group {
    display: flex;
    border: 1px solid var(--color-outline-variant);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .toggle-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    background: transparent;
    color: var(--color-on-surface-muted);
    border: none;
    cursor: pointer;
    transition: background var(--transition-fast), color var(--transition-fast);
  }

  .toggle-btn.active {
    background: var(--color-amber);
    color: var(--color-on-amber);
  }

  .toggle-btn:hover:not(.active) {
    background: var(--color-surface-container);
  }

  .export-btn {
    padding: var(--spacing-xs) var(--spacing-md);
    background: var(--color-surface-container);
    color: var(--color-on-surface);
    border: 1px solid var(--color-outline-variant);
    border-radius: var(--radius-md);
    font-size: 0.85rem;
    cursor: pointer;
    transition: background var(--transition-fast), border-color var(--transition-fast);
  }

  .export-btn:hover {
    background: var(--color-surface-container-high);
    border-color: var(--color-outline);
  }
</style>
