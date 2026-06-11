<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import ProductCard from '$lib/components/ProductCard.svelte';
  import DashboardCell from '$lib/components/DashboardCell.svelte';
  import Settings from '$lib/components/Settings.svelte';
  import SearchPanel from '$lib/components/SearchPanel.svelte';
  import SyncStatusCell from '$lib/components/SyncStatusCell.svelte';
  import CollectionStatsCell from '$lib/components/CollectionStatsCell.svelte';
  import type { RawProduct } from '$lib/types/search';
  import { syncResult } from '$lib/stores/sync';
  import { dashboardStats } from '$lib/stores/dashboard';
  import { collectionStore, loadCollection, loadCollectionStats } from '$lib/stores/collection';
  import { filterStore, restoreFiltersFromUrl } from '$lib/stores/filter';
  import pkg from '../../package.json';
  import '$lib/styles/dashboard.css';

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
    loadCollectionStats();
    loadCollection();
  }

  onMount(() => {
    loadDashboard();

    // Restore filters from URL on mount
    const restored = restoreFiltersFromUrl();
    const hasFilters = restored.category !== null
      || restored.price_min !== null
      || restored.price_max !== null
      || restored.source !== null
      || restored.condition !== null
      || restored.listing_currency !== null
      || restored.sort !== 'relevance';
    if (hasFilters) {
      filterStore.set(restored);
    }
  });

  let featuredProduct: RawProduct | null = $state(null);

  let appVersion = pkg.version;
</script>

<div class="page">
  <div class="bento-grid">
    <!-- Cell 1: Hero (Search Results) -->
    <div class="cell cell-hero">
      <SearchPanel
        {filterStore}
        collectionStore={$collectionStore}
        onfeaturedchange={(product) => { featuredProduct = product; }}
      />
    </div>

    <!-- Cell 2: Wide (Sync Status) -->
    <div class="cell cell-wide">
      <SyncStatusCell
        drops={$syncResult?.drops ?? []}
        dropsSent={$syncResult?.drops_sent ?? 0}
        syncState={$syncResult?.state ?? 'idle'}
      />
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
          <ProductCard product={featuredProduct} inCollection={$collectionStore.collectedSkus.has(featuredProduct.sku)} />
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

    <!-- Cell 8: Standard (Collection Stats) -->
    <div class="cell cell-standard" data-testid="collection-cell">
      <CollectionStatsCell
        stats={$collectionStore.stats}
        items={$collectionStore.items}
        loading={$collectionStore.loading}
      />
    </div>

    <!-- Cell 9: Standard (App Info) -->
    <div class="cell cell-standard">
      <DashboardCell title="About" icon="ℹ️" loading={false} empty={false}>
        <div class="app-info">
          <p class="app-name">GuitarHub</p>
          <p class="app-version">v{appVersion}</p>
          <a class="app-link" href="https://github.com/WillSanCaZam/guitarhub" target="_blank" rel="noopener noreferrer">GitHub</a>
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
    border-radius: 12px;
    overflow: hidden;
  }

  .cell:focus-within {
    outline: 2px solid #4a90d9;
    outline-offset: 2px;
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
    .recent-item {
      color: #ccc;
      background: rgba(255, 255, 255, 0.05);
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
    .settings-shortcut {
      min-height: 44px;
    }

    .recent-item {
      padding: 10px 8px;
      min-height: 44px;
      display: flex;
      align-items: center;
    }
  }
</style>
