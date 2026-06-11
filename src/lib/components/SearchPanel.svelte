<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { get } from 'svelte/store';
  import { onMount, onDestroy, untrack } from 'svelte';
  import type { Writable } from 'svelte/store';
  import { createVirtualizer } from '@tanstack/svelte-virtual';
  import ProductCard from './ProductCard.svelte';
  import DashboardCell from './DashboardCell.svelte';
  import FilterBar from './FilterBar.svelte';
  import { pageFromOffset } from '$lib/types/search';
  import type { SearchResult, RawProduct } from '$lib/types/search';
  import type { FilterState } from '$lib/stores/filter';
  import type { CollectionStore } from '$lib/stores/collection';

  interface Props {
    filterStore: Writable<FilterState>;
    collectionStore: CollectionStore;
    onfeaturedChange?: (product: RawProduct | null) => void;
  }

  let { filterStore, collectionStore, onfeaturedChange }: Props = $props();

  let query = $state('');
  let results: RawProduct[] = $state([]);
  let total = $state(0);
  let page = $state(1);
  let pageSize = $state(20);
  let loading = $state(false);
  let error: string | null = $state(null);
  let searched = $state(false);

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

  // Observe container width for responsive column calculation
  onMount(() => {
    if (scrollContainer) {
      resizeObserver = new ResizeObserver((entries) => {
        for (const entry of entries) {
          containerWidth = entry.contentRect.width;
        }
      });
      resizeObserver.observe(scrollContainer);
    }
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
      const currentFilters = get(filterStore);

      const res = await invoke<SearchResult>('search_products', {
        query: q,
        filters: {
          category: currentFilters.category,
          price_min: currentFilters.price_min,
          price_max: currentFilters.price_max,
          source: currentFilters.source,
          condition: currentFilters.condition,
          listing_currency: currentFilters.listing_currency,
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

  <FilterBar />

  <DashboardCell title="Search" icon="🔍" loading={false} empty={false}>
    {#if error}
      <div class="error-banner" role="alert">
        Search failed: {error}
      </div>
    {/if}

    {#if loading && results.length === 0}
      <div class="loading-state" aria-busy="true">
        <span class="spinner"></span>
        Searching...
      </div>
    {:else if searched && results.length === 0 && !loading}
      <div class="empty-state" role="status">
        <p>No products found for "{query.trim()}"</p>
        <p class="empty-hint">Try a different search term or browse all products.</p>
      </div>
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
                  <ProductCard
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
      <div class="empty-state" role="status">
        <span class="search-hint-icon" aria-hidden="true">🔍</span>
        <p>Search to find guitar deals</p>
        <p class="empty-hint">Type at least 3 characters and press Enter or click Search.</p>
      </div>
    {/if}
  </DashboardCell>
</div>

<style>
  .search-panel {
    display: flex;
    flex-direction: column;
  }

  .search-bar {
    display: flex;
    gap: 8px;
    margin-bottom: 16px;
  }

  .search-input {
    flex: 1;
    padding: 10px 14px;
    border: 1px solid #ccc;
    border-radius: 6px;
    font-size: 1rem;
    box-sizing: border-box;
    background: rgba(255, 255, 255, 0.8);
  }

  .search-input:focus {
    outline: none;
    border-color: #1a1a2e;
    box-shadow: 0 0 0 2px rgba(26,26,46,0.15);
  }

  .search-btn {
    padding: 10px 20px;
    background: #1a1a2e;
    color: #fff;
    border: none;
    border-radius: 6px;
    font-size: 0.95rem;
    cursor: pointer;
    white-space: nowrap;
  }

  .search-btn:hover:not(:disabled) {
    background: #2a2a4e;
  }

  .search-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .error-banner {
    padding: 12px 16px;
    background: #f8d7da;
    color: #721c24;
    border-radius: 6px;
    margin-bottom: 16px;
    font-size: 0.9rem;
  }

  .loading-state {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    padding: 48px 0;
    color: #666;
    font-size: 1rem;
  }

  .empty-state {
    text-align: center;
    padding: 48px 0;
    color: #666;
  }

  .empty-state p {
    margin: 0 0 8px;
    font-size: 1.1rem;
  }

  .empty-hint {
    font-size: 0.9rem !important;
    color: #999;
  }

  .search-hint-icon {
    font-size: 1.5rem;
    display: block;
    margin-bottom: 8px;
    opacity: 0.6;
  }

  .results-meta {
    display: flex;
    justify-content: space-between;
    margin-bottom: 12px;
    font-size: 0.85rem;
    color: #666;
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
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: 16px;
    width: 100%;
    padding: 0 4px;
    box-sizing: border-box;
  }

  .load-more-wrap {
    display: flex;
    justify-content: center;
    margin-top: 24px;
  }

  .load-more-btn {
    padding: 10px 32px;
    background: #fff;
    color: #333;
    border: 1px solid #ccc;
    border-radius: 6px;
    font-size: 0.9rem;
    cursor: pointer;
    transition: background 0.15s;
  }

  .load-more-btn:hover:not(:disabled) {
    background: #f0f0f0;
  }

  .load-more-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  @media (prefers-color-scheme: dark) {
    .search-input {
      background: rgba(30, 30, 40, 0.6);
      border-color: #444;
      color: #e8e8f0;
    }

    .search-input::placeholder {
      color: #888;
    }

    .search-input:focus {
      border-color: #e8e8f0;
      box-shadow: 0 0 0 2px rgba(232, 232, 240, 0.15);
    }

    .load-more-btn {
      background: rgba(30, 30, 40, 0.6);
      color: #e8e8f0;
      border-color: #444;
    }

    .load-more-btn:hover:not(:disabled) {
      background: rgba(50, 50, 65, 0.7);
    }
  }

  @media (max-width: 768px) {
    .search-btn,
    .load-more-btn {
      min-height: 44px;
    }
  }
</style>
