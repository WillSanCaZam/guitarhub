<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import GearCard from '$lib/components/GearCard.svelte';
  import SkeletonLoader from '$lib/components/ui/SkeletonLoader.svelte';
  import HeroSection from '$lib/components/discovery/HeroSection.svelte';
  import FeaturedRig from '$lib/components/discovery/FeaturedRig.svelte';
  import FeedSection from '$lib/components/discovery/FeedSection.svelte';
  import TrendingPill from '$lib/components/discovery/TrendingPill.svelte';
  import { collectionState } from '$lib/stores/collection.svelte';
  import { filterState, restoreFiltersFromUrl } from '$lib/stores/filter.svelte';
  import type { RawProduct } from '$lib/types/search';
  import type { Connection } from '$lib/types/stores';

  let featuredProducts = $state<RawProduct[]>([]);
  let priceDropProducts = $state<RawProduct[]>([]);
  let newArrivals = $state<RawProduct[]>([]);
  let loading = $state(true);

  const trending = [
    'John Mayer Strat',
    'Polyphia Tone',
    'Blues Jr',
    'Klon Centaur',
    'Neural DSP',
    'Strymon BigSky',
  ];

  onMount(async () => {
    // Restore filters from URL
    const restored = restoreFiltersFromUrl();
    Object.assign(filterState, restored);

    // Load connections to pass user_id to discovery commands
    let connections: Connection[] = []
    try {
      const result = await invoke<Connection[]>('list_connections')
      connections = result ?? []
    } catch {
      // No connections — ignore
    }
    // Use the first active connection's id as user context
    const userId = connections.length > 0 ? String(connections[0].id) : null

    // Load discovery data
    try {
      const [featured, drops, newItems] = await Promise.all([
        invoke<RawProduct[]>('get_featured_products', { limit: 6, userId }).catch(() => []),
        invoke<RawProduct[]>('get_price_drops', { limit: 6, userId }).catch(() => []),
        invoke<RawProduct[]>('get_new_arrivals', { limit: 6, userId }).catch(() => []),
      ]);
      featuredProducts = featured;
      priceDropProducts = drops;
      newArrivals = newItems;
    } catch (e) {
      console.error('Failed to load discovery feed:', e);
    } finally {
      loading = false;
    }
  });

  function handleHeroSearch(query: string) {
    window.location.href = `/explore?q=${encodeURIComponent(query)}`;
  }
</script>

<div class="discovery-feed">
  <!-- Hero Section -->
  <HeroSection onSearch={handleHeroSearch} />

  <!-- Featured Rig of the Week -->
  <FeaturedRig
    artist="Tim Henson"
    band="Polyphia"
    rigName="Studio Dream Rig"
    artistPhoto=""
    quote="This rig captures every texture I need for Polyphia's sound — from crystal cleans to crushing leads."
  />

  <!-- Trending Searches -->
  <FeedSection title="Trending Now">
    <TrendingPill {trending} onSearch={handleHeroSearch} />
  </FeedSection>

  <!-- Price Drops -->
  {#if priceDropProducts.length > 0}
    <FeedSection title="Price Drops" seeAllHref="/explore?sort=price_desc">
      <div class="product-scroll">
        {#each priceDropProducts as product (product.sku)}
          <GearCard
            {product}
            inCollection={collectionState.collectedSkus.has(product.sku)}
          />
        {/each}
      </div>
    </FeedSection>
  {/if}

  <!-- New Arrivals -->
  {#if newArrivals.length > 0}
    <FeedSection title="New Arrivals" seeAllHref="/explore?sort=newest">
      <div class="product-scroll">
        {#each newArrivals as product (product.sku)}
          <GearCard
            {product}
            inCollection={collectionState.collectedSkus.has(product.sku)}
          />
        {/each}
      </div>
    </FeedSection>
  {/if}

  <!-- Featured Products -->
  {#if featuredProducts.length > 0}
    <FeedSection title="Because You Viewed">
      <div class="product-scroll">
        {#each featuredProducts as product (product.sku)}
          <GearCard
            {product}
            inCollection={collectionState.collectedSkus.has(product.sku)}
          />
        {/each}
      </div>
    </FeedSection>
  {/if}

  <!-- Loading skeleton -->
  {#if loading}
    <div class="loading-state">
      <SkeletonLoader variant="card-grid" count={4} />
    </div>
  {/if}
</div>

<style>
  .discovery-feed {
    max-width: 1440px;
    margin: 0 auto;
    padding: 0 var(--space-6);
  }

  .product-scroll {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: var(--space-5);
  }

  /* Loading */
  .loading-state {
    padding: var(--space-8) 0;
  }
</style>
