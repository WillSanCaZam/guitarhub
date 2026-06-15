<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import ProductCard from '$lib/components/ProductCard.svelte';
  import HeroSection from '$lib/components/discovery/HeroSection.svelte';
  import FeaturedRig from '$lib/components/discovery/FeaturedRig.svelte';
  import FeedSection from '$lib/components/discovery/FeedSection.svelte';
  import TrendingPill from '$lib/components/discovery/TrendingPill.svelte';
  import { collectionState } from '$lib/stores/collection.svelte';
  import { filterState, restoreFiltersFromUrl } from '$lib/stores/filter.svelte';
  import type { RawProduct } from '$lib/types/search';

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

    // Load discovery data
    try {
      const [featured, drops, newItems] = await Promise.all([
        invoke<RawProduct[]>('get_featured_products', { limit: 6 }).catch(() => []),
        invoke<RawProduct[]>('get_price_drops', { limit: 6 }).catch(() => []),
        invoke<RawProduct[]>('get_new_arrivals', { limit: 6 }).catch(() => []),
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
          <ProductCard
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
          <ProductCard
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
          <ProductCard
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
      <div class="skeleton-scroll">
        {#each Array(4) as _, i}
          <div class="skeleton-card" style="animation-delay: {i * 60}ms">
            <div class="skeleton-image"></div>
            <div class="skeleton-text"></div>
            <div class="skeleton-text short"></div>
          </div>
        {/each}
      </div>
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

  .skeleton-scroll {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: var(--space-5);
  }

  .skeleton-card {
    border: 1px solid var(--color-outline-variant);
    border-radius: var(--radius-lg);
    overflow: hidden;
    background: var(--void-mid);
    animation: fadeIn 300ms var(--ease-plug) both;
  }

  .skeleton-image {
    width: 100%;
    aspect-ratio: 16 / 10;
    background: linear-gradient(90deg, var(--void-raised) 25%, var(--void-hover) 50%, var(--void-raised) 75%);
    background-size: 200% 100%;
    animation: shimmer 1.5s infinite;
  }

  .skeleton-text {
    height: 14px;
    margin: 12px 16px 0;
    background: var(--void-raised);
    border-radius: 4px;
  }

  .skeleton-text.short {
    width: 60%;
    margin-bottom: 16px;
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(8px); }
    to { opacity: 1; transform: translateY(0); }
  }

  @keyframes shimmer {
    0% { background-position: 200% 0; }
    100% { background-position: -200% 0; }
  }
</style>
