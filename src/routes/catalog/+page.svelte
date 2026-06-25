<script lang="ts">
  import { onMount } from 'svelte';
  import SearchPanel from '$lib/components/SearchPanel.svelte';
  import { filterState, restoreFiltersFromUrl } from '$lib/stores/filter.svelte';
  import { collectionState } from '$lib/stores/collection.svelte';

  onMount(() => {
    const restored = restoreFiltersFromUrl();
    Object.assign(filterState, restored);
  });
</script>

<div class="catalog-page">
  <header class="catalog-header">
    <a href="/" class="back-link">← Dashboard</a>
    <h1>Product Catalog</h1>
    <p class="subtitle">Browse all products from every store in one place</p>
  </header>

  <main class="catalog-content">
    <SearchPanel
      filterState={filterState}
      collectionStore={collectionState}
    />
  </main>
</div>

<style>
  .catalog-page {
    max-width: 1440px;
    margin: 0 auto;
    padding: var(--spacing-lg);
    min-height: 100vh;
    display: flex;
    flex-direction: column;
  }

  .catalog-header {
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
    color: var(--glow-primary);
  }

  h1 {
    margin: 0 0 var(--spacing-xs);
    font-family: var(--font-display);
    font-size: 1.75rem;
    font-weight: 700;
    color: var(--color-on-surface);
  }

  .subtitle {
    margin: 0;
    color: var(--color-on-surface-muted);
    font-size: 0.95rem;
  }

  .catalog-content {
    flex: 1;
  }
</style>
