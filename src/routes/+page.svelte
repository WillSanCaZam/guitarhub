<script lang="ts">
  import { onMount } from 'svelte';
  import ProductCard from '$lib/components/ProductCard.svelte';
  import DashboardCell from '$lib/components/DashboardCell.svelte';
  import Settings from '$lib/components/Settings.svelte';
  import SearchPanel from '$lib/components/SearchPanel.svelte';
  import SyncStatusCell from '$lib/components/SyncStatusCell.svelte';
  import CollectionStatsCell from '$lib/components/CollectionStatsCell.svelte';
  import type { RawProduct } from '$lib/types/search';
  import { syncState } from '$lib/stores/sync.svelte';
  import { dashboardState, loadDashboard } from '$lib/stores/dashboard.svelte';
  import { collectionState } from '$lib/stores/collection.svelte';
  import { filterState, restoreFiltersFromUrl } from '$lib/stores/filter.svelte';
  import pkg from '../../package.json';
  import '$lib/styles/dashboard.css';
  import '$lib/styles/page.css';

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
      Object.assign(filterState, restored);
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
        filterState={filterState}
        collectionStore={collectionState}
        onfeaturedChange={(product: RawProduct | null) => { featuredProduct = product; }}
      />
    </div>

    <!-- Cell 2: Wide (Sync Status) -->
    <div class="cell cell-wide">
      <SyncStatusCell
        drops={syncState?.drops ?? []}
        dropsSent={syncState?.drops_sent ?? 0}
        syncState={syncState?.state ?? 'idle'}
      />
    </div>

    <!-- Cell 3: Standard (Total Products) -->
    <div class="cell cell-standard">
      <DashboardCell title="Products" icon="🎸" loading={dashboardState.loading} empty={dashboardState.totalProducts === 0} emptyMessage="No products in catalog yet" emptyIcon="🎸">
        {#if !dashboardState.loading}
          <div class="stat-value">{dashboardState.totalProducts.toLocaleString()}</div>
          <div class="stat-label">in catalog</div>
        {/if}
      </DashboardCell>
    </div>

    <!-- Cell 4: Standard (Wishlist Count) -->
    <div class="cell cell-standard">
      <DashboardCell title="Wishlist" icon="❤️" loading={dashboardState.loading} empty={dashboardState.wishlistCount === 0} emptyMessage="Wishlist is empty" emptyIcon="❤️">
        {#if !dashboardState.loading}
          <div class="stat-value">{dashboardState.wishlistCount.toLocaleString()}</div>
          <div class="stat-label">items saved</div>
        {/if}
      </DashboardCell>
    </div>

    <!-- Cell 5: Standard (Recent Searches) -->
    <div class="cell cell-standard">
      <DashboardCell title="Recent Searches" icon="🕓" loading={dashboardState.loading} empty={dashboardState.recentSearches.length === 0} emptyMessage="Start searching to see history" emptyIcon="🕓">
        {#if !dashboardState.loading && dashboardState.recentSearches.length > 0}
          <ul class="recent-list">
            {#each dashboardState.recentSearches as search}
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
          <ProductCard product={featuredProduct} inCollection={collectionState.collectedSkus.has(featuredProduct.sku)} />
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
        stats={collectionState.stats}
        items={collectionState.items}
        loading={collectionState.loading}
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

