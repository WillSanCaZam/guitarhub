<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import ProductCard from '$lib/components/ProductCard.svelte';
  import Settings from '$lib/components/Settings.svelte';
  import { pageFromOffset } from '$lib/types/search';
  import type { SearchResult } from '$lib/types/search';

  let query = $state('');
  let results = $state([]);
  let total = $state(0);
  let page = $state(1);
  let pageSize = $state(20);
  let loading = $state(false);
  let error = $state(null);
  let searched = $state(false);

  async function search(reset) {
    const q = query.trim();
    if (q.length < 3) return;

    const targetPage = reset ? 1 : page + 1;
    const offset = (targetPage - 1) * pageSize;
    const limit = pageSize;

    loading = true;
    error = null;

    try {
      const res = await invoke<SearchResult>('search_products', {
        query: q,
        filters: { category: null, price_min: null, price_max: null, source: null },
        sort: 'relevance',
        page: targetPage,
        pageSize
      });

      results = reset ? res.products : [...results, ...res.products];
      total = res.total;
      page = pageFromOffset(res.offset, res.limit);
      searched = true;
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  function handleSearch() {
    search(true);
  }

  function handleKeydown(e) {
    if (e.key === 'Enter') {
      search(true);
    }
  }

  function loadMore() {
    if (!loading) {
      search(false);
    }
  }

  let hasMore = $derived(total > page * pageSize);
</script>

<div class="page">
  <div class="search-section">
    <h1>Search Products</h1>
    <div class="search-bar">
      <input
        type="text"
        bind:value={query}
        onkeydown={handleKeydown}
        placeholder="Search guitars, basses, amps... (min. 3 characters)"
        disabled={loading}
        class="search-input"
      />
      <button
        onclick={handleSearch}
        disabled={loading || query.trim().length < 3}
        class="search-btn"
      >
        Search
      </button>
    </div>
  </div>

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
    <section class="results-section">
      <div class="results-meta">
        <span>{total} result{total !== 1 ? 's' : ''} found</span>
        <span>Page {page}</span>
      </div>
      <div class="grid">
        {#each results as item (item.sku)}
          <ProductCard product={item} />
        {/each}
      </div>
      {#if hasMore}
        <div class="load-more-wrap">
          <button onclick={loadMore} disabled={loading} class="load-more-btn">
            {loading ? 'Loading...' : 'Load More'}
          </button>
        </div>
      {/if}
    </section>
  {:else if !searched}
    <div class="welcome-state">
      <p>Enter a search term above to find guitar gear listings.</p>
    </div>
  {/if}

  <section id="settings">
    <Settings />
  </section>
</div>

<style>
  .page {
    max-width: 960px;
    margin: 0 auto;
    padding: 24px 16px;
  }
  .search-section {
    margin-bottom: 24px;
  }
  .search-section h1 {
    margin: 0 0 12px;
    font-size: 1.5rem;
  }
  .search-bar {
    display: flex;
    gap: 8px;
  }
  .search-input {
    flex: 1;
    padding: 10px 14px;
    border: 1px solid #ccc;
    border-radius: 6px;
    font-size: 1rem;
    box-sizing: border-box;
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
  .spinner {
    width: 20px;
    height: 20px;
    border: 2px solid #ddd;
    border-top-color: #1a1a2e;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }
  @keyframes spin {
    to { transform: rotate(360deg); }
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
  .welcome-state {
    text-align: center;
    padding: 48px 0;
    color: #999;
    font-size: 1rem;
  }
  .results-section {
    margin-bottom: 32px;
  }
  .results-meta {
    display: flex;
    justify-content: space-between;
    margin-bottom: 12px;
    font-size: 0.85rem;
    color: #666;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 16px;
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
</style>
