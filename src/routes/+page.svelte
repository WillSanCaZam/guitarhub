<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import ProductCard from '$lib/components/ProductCard.svelte';
  import DashboardCell from '$lib/components/DashboardCell.svelte';
  import Settings from '$lib/components/Settings.svelte';
  import { pageFromOffset } from '$lib/types/search';
  import type { SearchResult } from '$lib/types/search';
  import { syncResult } from '$lib/stores/sync';
  import { dashboardStats } from '$lib/stores/dashboard';
  import pkg from '../../package.json';

  let query = $state('');
  let results = $state([]);
  let total = $state(0);
  let page = $state(1);
  let pageSize = $state(20);
  let loading = $state(false);
  let error = $state(null);
  let searched = $state(false);

  async function loadDashboard() {
    dashboardStats.update(s => ({ ...s, loading: true, error: null }));
    try {
      const [totalProducts, wishlistCount, recentSearches] = await Promise.all([
        invoke<number>('get_total_products'),
        invoke<number>('get_wishlist_count'),
        invoke<string[]>('get_recent_searches')
      ]);
      dashboardStats.set({
        totalProducts,
        wishlistCount,
        recentSearches,
        loading: false,
        error: null,
      });
    } catch (e) {
      dashboardStats.update(s => ({ ...s, loading: false, error: String(e) }));
    }
  }

  onMount(() => {
    loadDashboard();
  });

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

  let drops = $derived($syncResult?.drops ?? []);
  let dropsSent = $derived($syncResult?.drops_sent ?? 0);

  let featuredProduct = $derived(results.length > 0 ? results[0] : null);
  let appVersion = pkg.version;
</script>

<div class="page">
  <div class="bento-grid">
    <!-- Cell 1: Hero (Search Results) -->
    <div class="cell cell-hero">
      <DashboardCell title="Search" icon="🔍" loading={false} empty={!searched} emptyMessage="Search to find guitar deals" emptyIcon="🔍">
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
          <div class="product-grid">
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
        {/if}
      </DashboardCell>
    </div>

    <!-- Cell 2: Wide (Sync Status) -->
    <div class="cell cell-wide">
      <DashboardCell title="Sync Status" icon="🔄" loading={false} empty={drops.length === 0} emptyMessage="Sync catalog to see price drops" emptyIcon="🔄">
        {#if drops.length > 0}
          <div class="sync-toast">
            {drops.length} price drop(s) detected
            {#if dropsSent > 0}
              , {dropsSent} sent
            {/if}
          </div>
          <ul class="drop-list">
            {#each drops.slice(0, 3) as drop}
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
          <p class="sync-idle">No recent price drops. Last sync: {$syncResult?.state ?? 'idle'}</p>
        {/if}
      </DashboardCell>
    </div>

    <!-- Cell 3: Standard (Total Products) -->
    <div class="cell cell-standard">
      <DashboardCell title="Products" icon="🎸" loading={$dashboardStats.loading} empty={$dashboardStats.totalProducts === 0} emptyMessage="No products in catalog yet" emptyIcon="🎸">
        {#if !$dashboardStats.loading}
          <div class="stat-value">{$dashboardStats.totalProducts.toLocaleString()}</div>
          <div class="stat-label">in catalog</div>
        {/if}
      </DashboardCell>
    </div>

    <!-- Cell 4: Standard (Wishlist Count) -->
    <div class="cell cell-standard">
      <DashboardCell title="Wishlist" icon="❤️" loading={$dashboardStats.loading} empty={$dashboardStats.wishlistCount === 0} emptyMessage="Wishlist is empty" emptyIcon="❤️">
        {#if !$dashboardStats.loading}
          <div class="stat-value">{$dashboardStats.wishlistCount.toLocaleString()}</div>
          <div class="stat-label">items saved</div>
        {/if}
      </DashboardCell>
    </div>

    <!-- Cell 5: Standard (Recent Searches) -->
    <div class="cell cell-standard">
      <DashboardCell title="Recent Searches" icon="🕓" loading={$dashboardStats.loading} empty={$dashboardStats.recentSearches.length === 0} emptyMessage="Start searching to see history" emptyIcon="🕓">
        {#if !$dashboardStats.loading && $dashboardStats.recentSearches.length > 0}
          <ul class="recent-list">
            {#each $dashboardStats.recentSearches as search}
              <li class="recent-item">{search}</li>
            {/each}
          </ul>
        {/if}
      </DashboardCell>
    </div>

    <!-- Cell 6: Wide (Featured Deal) -->
    <div class="cell cell-wide">
      <DashboardCell title="Featured Deal" icon="⭐" loading={false} empty={!featuredProduct} emptyMessage="No featured deal available" emptyIcon="⭐">
        {#if featuredProduct}
          <ProductCard product={featuredProduct} />
        {/if}
      </DashboardCell>
    </div>

    <!-- Cell 7: Standard (Settings Shortcut) -->
    <div class="cell cell-standard">
      <DashboardCell title="Quick Settings" icon="⚙️" loading={false} empty={false}>
        <button class="settings-shortcut" onclick={() => document.getElementById('settings')?.scrollIntoView({ behavior: 'smooth' })}>
          <span class="shortcut-icon" aria-hidden="true">⚙️</span>
          <span>Open Settings</span>
        </button>
      </DashboardCell>
    </div>

    <!-- Cell 8: Standard (Price Trends Placeholder) -->
    <div class="cell cell-standard">
      <DashboardCell title="Price Trends" icon="📈" loading={false} empty={true} emptyMessage="Price trends coming soon" emptyIcon="📈"></DashboardCell>
    </div>

    <!-- Cell 9: Standard (App Info) -->
    <div class="cell cell-standard">
      <DashboardCell title="About" icon="ℹ️" loading={false} empty={false}>
        <div class="app-info">
          <p class="app-name">GuitarHub</p>
          <p class="app-version">v{appVersion}</p>
          <a class="app-link" href="https://github.com/user/guitarhub" target="_blank" rel="noopener noreferrer">GitHub</a>
          <p class="app-stack">Built with Tauri + Svelte 5</p>
        </div>
      </DashboardCell>
    </div>
  </div>

  <section id="settings">
    <Settings />
  </section>
</div>

<style>
  .page {
    max-width: 1200px;
    margin: 0 auto;
    padding: 16px;
  }

  .bento-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 16px;
    margin-bottom: 24px;
  }

  .cell {
    min-width: 0;
  }

  .cell-hero {
    grid-column: span 2;
    grid-row: span 2;
  }

  .cell-wide {
    grid-column: span 2;
  }

  .cell-standard {
    grid-column: span 1;
    grid-row: span 1;
  }

  @media (max-width: 768px) {
    .bento-grid {
      grid-template-columns: 1fr;
    }

    .cell-hero,
    .cell-wide,
    .cell-standard {
      grid-column: span 1;
      grid-row: span 1;
    }
  }

  /* Search styles within hero cell */
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

  .results-meta {
    display: flex;
    justify-content: space-between;
    margin-bottom: 12px;
    font-size: 0.85rem;
    color: #666;
  }

  .product-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
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

  /* Sync cell styles */
  .sync-toast {
    padding: 8px 12px;
    background: #d4edda;
    color: #155724;
    border: 1px solid #c3e6cb;
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
    color: #1a1a2e;
  }

  .drop-price {
    color: #28a745;
    font-family: monospace;
  }

  .drop-reason {
    color: #666;
    font-size: 0.8rem;
  }

  .sync-idle {
    margin: 0;
    color: #888;
    font-size: 0.9rem;
  }

  /* Stat cell styles */
  .stat-value {
    font-size: 2rem;
    font-weight: 700;
    color: #1a1a2e;
    line-height: 1;
  }

  .stat-label {
    font-size: 0.85rem;
    color: #666;
    margin-top: 4px;
  }

  /* Recent searches */
  .recent-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .recent-item {
    font-size: 0.9rem;
    color: #333;
    padding: 4px 8px;
    background: rgba(0, 0, 0, 0.03);
    border-radius: 4px;
  }

  /* Settings shortcut */
  .settings-shortcut {
    padding: 10px 16px;
    background: #1a1a2e;
    color: #fff;
    border: none;
    border-radius: 6px;
    font-size: 0.9rem;
    cursor: pointer;
    margin-top: auto;
  }

  .settings-shortcut:hover {
    background: #2a2a4e;
  }

  .shortcut-icon {
    margin-right: 6px;
  }

  /* App info */
  .app-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .app-name {
    font-weight: 700;
    font-size: 1rem;
    color: #1a1a2e;
    margin: 0;
  }

  .app-version {
    font-size: 0.85rem;
    color: #666;
    margin: 0;
  }

  .app-link {
    font-size: 0.85rem;
    color: #4a90d9;
    text-decoration: none;
    margin: 0;
  }

  .app-link:hover {
    text-decoration: underline;
  }

  .app-stack {
    font-size: 0.8rem;
    color: #999;
    margin: 0;
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

    .drop-item {
      background: rgba(255, 255, 255, 0.05);
    }

    .drop-sku {
      color: #e8e8f0;
    }

    .stat-value {
      color: #e8e8f0;
    }

    .recent-item {
      color: #ccc;
      background: rgba(255, 255, 255, 0.05);
    }

    .sync-idle {
      color: #aaa;
    }

    .app-name {
      color: #e8e8f0;
    }

    .app-version {
      color: #aaa;
    }

    .app-link {
      color: #7ab8e8;
    }

    .app-stack {
      color: #888;
    }
  }

  @media (max-width: 768px) {
    .search-btn,
    .load-more-btn,
    .settings-shortcut {
      min-height: 44px;
    }

    .recent-item {
      padding: 10px 8px;
      min-height: 44px;
      display: flex;
      align-items: center;
    }

    .drop-item {
      padding: 10px 8px;
      min-height: 44px;
    }
  }
</style>
