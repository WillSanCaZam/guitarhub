<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount, onDestroy, untrack } from 'svelte';
  import { createVirtualizer } from '@tanstack/svelte-virtual';
  import GearCard from './GearCard.svelte';
  import DashboardCell from './DashboardCell.svelte';
  import FilterBar from './FilterBar.svelte';
  import SkeletonLoader from './ui/SkeletonLoader.svelte';
  import EmptyState from './ui/EmptyState.svelte';
  import { pageFromOffset } from '$lib/types/search';
  import type { SearchResult, RawProduct } from '$lib/types/search';
  import type { FilterState } from '$lib/stores/filter.svelte';
  import { dashboardState } from '$lib/stores/dashboard.svelte';
  import type { CollectionStore } from '$lib/stores/collection.svelte';
  import type { Connection } from '$lib/types/stores';

  interface Props {
    filterState: FilterState;
    collectionStore: CollectionStore;
    onfeaturedChange?: (product: RawProduct | null) => void;
  }

  let { filterState, collectionStore, onfeaturedChange }: Props = $props();

  let query = $state('');
  let results: RawProduct[] = $state([]);
  let total = $state(0);
  let page = $state(1);
  let pageSize = $state(20);
  let loading = $state(false);
  let error: string | null = $state(null);
  let searched = $state(false);

  // Store connection filter
  let connections = $state<Connection[]>([])
  let loadingConnections = $state(true)

  onMount(async () => {
    // Load user connections for the source filter
    try {
      connections = await invoke<Connection[]>('list_connections')
    } catch {
      connections = []
    } finally {
      loadingConnections = false
    }

    if (scrollContainer) {
      resizeObserver = new ResizeObserver((entries) => {
        for (const entry of entries) {
          containerWidth = entry.contentRect.width;
        }
      });
      resizeObserver.observe(scrollContainer);
    }
  });

  // Virtual scrolling state
  let scrollContainer: HTMLElement | undefined = $state(undefined);
  let containerWidth = $state(800);
  let resizeObserver: ResizeObserver | null = null;

  // Calculate column count based on container width (min card width ~220px + 16px gap)
  const CARD_MIN_WIDTH = 220;
  const CARD_GAP = 16;
  const ESTIMATED_ROW_HEIGHT = 340;

  let columnCount = $derived(Math.max(1, Math.floor((containerWidth + CARD_GAP) / (CARD_MIN_WIDTH + CARD_GAP))));
  let rowCount = $derived(Math.ceil(results.length / columnCount));

  let hasMore = $derived(total > page * pageSize);

  // Create virtualizer — options are kept in sync via $effect below
  const virtualizer = createVirtualizer({
    count: 0,
    getScrollElement: () => scrollContainer ?? null,
    estimateSize: () => ESTIMATED_ROW_HEIGHT,
    overscan: 3,
  });

  // Keep virtualizer options in sync with reactive state
  // Untrack setOptions to prevent infinite loop: the virtualizer writes
  // to internal Svelte stores, which would otherwise re-trigger this effect.
  $effect(() => {
    const count = rowCount;
    const container = scrollContainer;
    untrack(() => {
      $virtualizer.setOptions({
        count,
        getScrollElement: () => container ?? null,
        estimateSize: () => ESTIMATED_ROW_HEIGHT,
        overscan: 3,
      });
    });
  });

  // Notify parent when the featured product (first result) changes
  $effect(() => {
    onfeaturedChange?.(results.length > 0 ? results[0] : null);
  });

  onDestroy(() => {
    resizeObserver?.disconnect();
  });

  async function search(reset: boolean) {
    const q = query.trim();
    if (q.length < 3) return;

    const targetPage = reset ? 1 : page + 1;
    const offset = (targetPage - 1) * pageSize;

    loading = true;
    error = null;

    try {
      const currentFilters = filterState;

      const res = await invoke<SearchResult>('search_products', {
        query: q,
        filters: {
          category: currentFilters.category,
          price_min: currentFilters.price_min,
          price_max: currentFilters.price_max,
          source: currentFilters.source,
          condition: currentFilters.condition,
          listing_currency: currentFilters.listing_currency,
          store_connection_id: currentFilters.store_connection_id,
        },
        sort: currentFilters.sort,
        page: targetPage,
        pageSize
      });

      results = reset ? res.products : [...results, ...res.products];
      total = res.total;
      page = pageFromOffset(res.offset, res.limit);
      searched = true;

      // Reset scroll position on new search (not on load more)
      if (reset && scrollContainer) {
        scrollContainer.scrollTop = 0;
      }
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  function handleSearch() {
    search(true);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      search(true);
    }
  }

  function loadMore() {
    if (!loading) {
      search(false);
    }
  }

  /** Get the results items for a given virtual row (handles partial last row) */
  function getRowItems(rowIndex: number): RawProduct[] {
    const start = rowIndex * columnCount;
    return results.slice(start, start + columnCount);
  }
</script>

<div class="search-panel">
  <div class="search-bar">
    <input
      type="text"
      bind:value={query}
      onkeydown={handleKeydown}
      placeholder="Search guitars, basses, amps... (min. 3 characters)"
      disabled={loading}
      class="search-input"
      data-testid="search-input"
    />
    <button
      onclick={handleSearch}
      disabled={loading || query.trim().length < 3}
      class="search-btn"
      data-testid="search-button"
    >
      Search
    </button>
  </div>

  {#if dashboardState.recentSearches.length > 0}
    <div class="recent-searches">
      <span class="recent-label">Recent:</span>
      {#each dashboardState.recentSearches as recent}
        <button
          class="recent-chip"
          onclick={() => { query = recent; search(true); }}
        >
          {recent}
        </button>
      {/each}
    </div>
  {/if}

  <!-- Source / Store Filter -->
  {#if !loadingConnections}
    <div class="source-filter">
      <span class="source-filter-label">Source:</span>
      <div class="source-filter-options">
        <button
          class="source-chip"
          class:active={filterState.store_connection_id === null}
          onclick={() => { filterState.store_connection_id = null }}
        >
          All
        </button>
        <button
          class="source-chip"
          class:active={filterState.store_connection_id === 'public'}
          onclick={() => { filterState.store_connection_id = 'public' }}
        >
          Public
        </button>
        {#each connections as conn (conn.id)}
          <button
            class="source-chip"
            class:active={filterState.store_connection_id === String(conn.id)}
            onclick={() => { filterState.store_connection_id = String(conn.id) }}
          >
            My Stores
          </button>
        {/each}
      </div>
    </div>
  {/if}

  <FilterBar />

  <DashboardCell title="Search" icon="🔍" loading={false} empty={false}>
    {#if error}
      <div class="error-banner" role="alert">
        Search failed: {error}
      </div>
    {/if}

    {#if loading && results.length === 0}
      <SkeletonLoader variant="card-grid" count={6} />
    {:else if searched && results.length === 0 && !loading}
      <EmptyState
        variant="search"
        title="No products found"
        description="Try a different search term or browse all products."
      />
    {:else if results.length > 0}
      <div class="results-meta">
        <span>{total} result{total !== 1 ? 's' : ''} found</span>
        <span>Page {page}</span>
      </div>
      <div
        class="virtual-scroll-container"
        bind:this={scrollContainer}
      >
        <div
          class="virtual-scroll-inner"
          style="height: {$virtualizer.getTotalSize()}px;"
        >
          {#each $virtualizer.getVirtualItems() as virtualRow (virtualRow.key)}
            <div
              class="virtual-row"
              style="height: {virtualRow.size}px; transform: translateY({virtualRow.start}px);"
            >
              <div class="product-grid">
                {#each getRowItems(virtualRow.index) as item (item.sku)}
                  <GearCard
                    product={item}
                    inCollection={collectionStore.collectedSkus.has(item.sku)}
                  />
                {/each}
              </div>
            </div>
          {/each}
        </div>
        {#if hasMore}
          <div class="load-more-wrap">
            <button onclick={loadMore} disabled={loading} class="load-more-btn">
              {loading ? 'Loading...' : 'Load More'}
            </button>
          </div>
        {/if}
      </div>
    {:else if !searched}
      <EmptyState
        variant="search"
        title="Search to find guitar deals"
        description="Type at least 3 characters and press Enter or click Search."
      />
    {/if}
  </DashboardCell>
</div>

<style>
  .search-panel {
    display: flex;
    flex-direction: column;
  }

  .source-filter {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-3);
    flex-wrap: wrap;
  }

  .source-filter-label {
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--text-dim);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .source-filter-options {
    display: flex;
    gap: var(--space-1);
    flex-wrap: wrap;
  }

  .source-chip {
    padding: var(--space-1) var(--space-3);
    border-radius: var(--radius-pill);
    border: 1px solid var(--border-subtle);
    background: transparent;
    color: var(--text-warm);
    font-size: 0.8rem;
    cursor: pointer;
    transition: background var(--transition-fast), color var(--transition-fast), border-color var(--transition-fast);
  }

  .source-chip:hover {
    background: var(--void-hover);
    border-color: var(--border-hover);
  }

  .source-chip.active {
    background: var(--glow-primary);
    color: var(--void-deep);
    border-color: var(--glow-primary);
  }

  .recent-searches {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-md);
    flex-wrap: wrap;
  }

  .recent-label {
    font-size: 0.85rem;
    color: var(--color-on-surface-muted);
  }

  .recent-chip {
    padding: var(--spacing-2xs) var(--spacing-sm);
    border-radius: var(--radius-pill);
    border: 1px solid var(--color-outline);
    background: transparent;
    color: var(--color-on-surface-variant);
    font-size: 0.8rem;
    cursor: pointer;
    transition: background var(--transition-fast), color var(--transition-fast);
  }

  .recent-chip:hover {
    background: var(--color-surface-container);
    color: var(--color-on-surface);
  }

  .search-bar {
    display: flex;
    gap: 8px;
    margin-bottom: 16px;
  }

  .search-input {
    flex: 1;
    padding: 10px 14px;
    border: 1px solid var(--text-muted);
    border-radius: var(--radius-sm);
    font-size: 1rem;
    box-sizing: border-box;
    background: var(--void-mid);
    color: var(--text-bright);
  }

  .search-input:focus {
    outline: none;
    border-color: var(--glow-primary);
    box-shadow: 0 0 0 2px var(--glow-soft);
  }

  .search-btn {
    padding: 10px 20px;
    background: var(--color-secondary);
    color: var(--color-on-surface);
    border: none;
    border-radius: 6px;
    font-size: 0.95rem;
    cursor: pointer;
    white-space: nowrap;
  }

  .search-btn:hover:not(:disabled) {
    background: var(--color-secondary);
  }

  .search-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .error-banner {
    padding: 12px 16px;
    background: var(--color-error-container);
    color: var(--color-error);
    border-radius: 6px;
    margin-bottom: 16px;
    font-size: 0.9rem;
  }

  .results-meta {
    display: flex;
    justify-content: space-between;
    margin-bottom: 12px;
    font-size: 0.85rem;
    color: var(--color-on-surface-muted);
  }

  .virtual-scroll-container {
    height: 600px;
    overflow-y: auto;
    overflow-x: hidden;
    position: relative;
  }

  .virtual-scroll-inner {
    position: relative;
    width: 100%;
  }

  .virtual-row {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    display: flex;
    align-items: stretch;
  }

  .product-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: var(--space-5);
    width: 100%;
    padding: 0;
    box-sizing: border-box;
  }

  .load-more-wrap {
    display: flex;
    justify-content: center;
    margin-top: 24px;
  }

  .load-more-btn {
    padding: 10px 32px;
    background: var(--color-on-surface);
    color: var(--color-surface-container-high);
    border: 1px solid var(--color-on-surface-variant);
    border-radius: 6px;
    font-size: 0.9rem;
    cursor: pointer;
    transition: background 0.15s;
  }

  .load-more-btn:hover:not(:disabled) {
    background: var(--color-surface);
  }

  .load-more-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  @media (prefers-color-scheme: dark) {
    .search-input {
      background: var(--void-deep);
      border-color: var(--text-muted);
      color: var(--text-bright);
    }

    .search-input::placeholder {
      color: var(--text-dim);
    }

    .search-input:focus {
      border-color: var(--glow-primary);
      box-shadow: 0 0 0 2px var(--glow-soft);
    }

    .load-more-btn {
      background: var(--void-raised);
      color: var(--text-bright);
      border-color: var(--text-muted);
    }

    .load-more-btn:hover:not(:disabled) {
      background: var(--void-hover);
    }
  }

  @media (max-width: 768px) {
    .search-btn,
    .load-more-btn {
      min-height: 44px;
    }
  }
</style>
