<script lang="ts">
  import { onMount } from 'svelte';
  import ProductCard from '$lib/components/ProductCard.svelte';
  import DashboardCell from '$lib/components/DashboardCell.svelte';
  import SearchPanel from '$lib/components/SearchPanel.svelte';
  import Settings from '$lib/components/Settings.svelte';
  import SyncStatusCell from '$lib/components/SyncStatusCell.svelte';
  import CollectionStatsCell from '$lib/components/CollectionStatsCell.svelte';
  import type { RawProduct } from '$lib/types/search';
  import { syncState } from '$lib/stores/sync.svelte';
  import { dashboardState, loadDashboard } from '$lib/stores/dashboard.svelte';
  import { collectionState } from '$lib/stores/collection.svelte';
  import { filterState, restoreFiltersFromUrl } from '$lib/stores/filter.svelte';
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

    <!-- Cell 3: Standard (Stats KPIs — Products + Wishlist) -->
    <div class="cell cell-standard">
      <DashboardCell title="Stats" icon="📊" loading={dashboardState.loading} empty={dashboardState.totalProducts === 0} emptyMessage="No products yet" emptyIcon="📊">
        {#if !dashboardState.loading}
          <div class="stats-row">
            <div class="stat-group">
              <div class="stat-value">{dashboardState.totalProducts.toLocaleString()}</div>
              <div class="stat-label">Products</div>
            </div>
            <div class="stat-divider"></div>
            <div class="stat-group">
              <div class="stat-value">{dashboardState.wishlistCount.toLocaleString()}</div>
              <div class="stat-label">Wishlist</div>
            </div>
          </div>
        {/if}
      </DashboardCell>
    </div>

    <!-- Cell 4: Wide (Featured Deal) -->
    <div class="cell cell-wide">
      <DashboardCell title="Featured Deal" icon="⭐" loading={false} empty={!featuredProduct} emptyMessage="No featured deal available" emptyIcon="⭐">
        {#if featuredProduct}
          <ProductCard product={featuredProduct} inCollection={collectionState.collectedSkus.has(featuredProduct.sku)} />
        {/if}
      </DashboardCell>
    </div>

    <!-- Cell 5: Standard (Collection Stats) -->
    <div class="cell cell-standard" data-testid="collection-cell">
      <CollectionStatsCell
        stats={collectionState.stats}
        items={collectionState.items}
        loading={collectionState.loading}
      />
    </div>
  </div>

  <section id="settings">
    <Settings />
  </section>
</div>

