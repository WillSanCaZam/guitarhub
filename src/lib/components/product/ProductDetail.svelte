<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import PriceDisplay from '$lib/components/ui/PriceDisplay.svelte';
  import StarRating from '$lib/components/ui/StarRating.svelte';
  import StoreComparison from '$lib/components/product/StoreComparison.svelte';
  import PriceHistory from '$lib/components/product/PriceHistory.svelte';
  import { addToWishlist, removeFromWishlist, wishlistState } from '$lib/stores/wishlist.svelte';
  import type { RawProduct } from '$lib/types/search';

  interface Props {
    product: RawProduct;
  }

  let { product }: Props = $props();

  let imageData = $state<string>('');
  let activeImage = $state(0);
  let loading = $state(true);

  const isInWishlist = $derived(
    wishlistState.items.some(item => item.sku === product.sku)
  );

  const specs = $derived.by(() => {
    try {
      return product.specs_json ? JSON.parse(product.specs_json) : {};
    } catch {
      return {};
    }
  });

  onMount(async () => {
    try {
      imageData = await invoke<string>('get_product_image', { imageUrl: product.image_url });
    } catch (e) {
      console.error('Failed to load image:', e);
    } finally {
      loading = false;
    }
  });

  async function handleWishlistToggle() {
    if (isInWishlist) {
      const item = wishlistState.items.find(i => i.sku === product.sku);
      if (item) await removeFromWishlist(item.id);
    } else {
      await addToWishlist({
        sku: product.sku,
        name: product.name,
        brand: product.brand,
        price: product.price,
        currency: product.currency,
        image_url: product.image_url,
        product_url: product.url,
      });
    }
  }

  function handleOpenUrl() {
    if (product.url) {
      invoke('open_url', { url: product.url });
    }
  }
</script>

<div class="product-detail">
  <!-- Main Content -->
  <div class="detail-grid">
    <!-- Image Gallery -->
    <div class="gallery">
      <div class="main-image">
        {#if imageData}
          <img src={imageData} alt={product.name} />
        {:else}
          <div class="shimmer-placeholder"></div>
        {/if}
      </div>
    </div>

    <!-- Product Info -->
    <div class="info">
      <div class="info-header">
        <span class="brand">{product.brand}</span>
        <h1 class="product-name">{product.name}</h1>
        {#if product.model}
          <p class="model">{product.model}</p>
        {/if}
      </div>

      <StarRating rating={4.5} reviewCount={342} size="md" />

      <div class="price-section">
        <PriceDisplay
          price={product.price}
          originalPrice={product.price * 1.15}
          currency={product.currency}
          size="lg"
        />
        <div class="stock-status">
          <span class="badge badge-stock">In Stock</span>
          <span class="badge badge-best">Best Price</span>
        </div>
      </div>

      <!-- Actions -->
      <div class="actions">
        {#if product.url}
          <button class="action-btn primary" onclick={handleOpenUrl}>
            View at Store →
          </button>
        {/if}
        <button class="action-btn secondary" onclick={handleWishlistToggle}>
          <svg viewBox="0 0 24 24" fill={isInWishlist ? 'var(--glow-hot)' : 'none'} stroke="currentColor" stroke-width="2" width="20" height="20">
            <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/>
          </svg>
          {isInWishlist ? 'Remove from Wishlist' : 'Add to Wishlist'}
        </button>
      </div>

      <!-- Store Comparison -->
      <StoreComparison stores={[
        { name: 'Sweetwater', price: product.price, url: product.url || '#' },
        { name: 'Guitar Center', price: product.price * 1.03, url: '#' },
        { name: 'Reverb', price: product.price * 0.97, url: '#' },
      ]} />
    </div>
  </div>

  <!-- Price History -->
  <section class="section">
    <h2 class="section-title">Price History</h2>
    <PriceHistory history={[
      { date: '2024-01', price: product.price * 1.1 },
      { date: '2024-03', price: product.price * 1.05 },
      { date: '2024-06', price: product.price * 1.08 },
      { date: '2024-09', price: product.price },
      { date: '2024-12', price: product.price * 0.98 },
      { date: '2025-01', price: product.price },
    ]} />
  </section>

  <!-- Specs -->
  {#if Object.keys(specs).length > 0}
    <section class="section">
      <h2 class="section-title">Specifications</h2>
      <div class="specs-grid">
        {#each Object.entries(specs) as [key, value]}
          <div class="spec-card">
            <span class="spec-label">{key}</span>
            <span class="spec-value">{value}</span>
          </div>
        {/each}
      </div>
    </section>
  {/if}
</div>

<style>
  .product-detail {
    display: flex;
    flex-direction: column;
    gap: var(--space-8);
  }

  .detail-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-8);
    align-items: start;
  }

  /* Gallery */
  .gallery {
    position: sticky;
    top: var(--space-6);
  }

  .main-image {
    aspect-ratio: 4 / 3;
    border-radius: var(--radius-lg);
    overflow: hidden;
    background: var(--void-raised);
  }

  .main-image img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .shimmer-placeholder {
    width: 100%;
    height: 100%;
    background: linear-gradient(90deg, var(--void-raised) 25%, var(--void-hover) 50%, var(--void-raised) 75%);
    background-size: 200% 100%;
    animation: shimmer 1.5s infinite;
  }

  /* Info */
  .info {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .brand {
    font-family: var(--font-mono);
    font-size: 0.8rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--glow-primary);
  }

  .product-name {
    margin: 0;
    font-family: var(--font-display);
    font-size: 2rem;
    font-weight: 700;
    color: var(--text-bright);
    line-height: 1.2;
  }

  .model {
    margin: 0;
    color: var(--text-warm);
    font-size: 1rem;
  }

  .price-section {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .stock-status {
    display: flex;
    gap: var(--space-2);
  }

  .badge {
    font-size: 0.7rem;
    font-weight: 700;
    padding: 2px var(--space-2);
    border-radius: var(--radius-sm);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .badge-stock {
    background: rgba(0, 230, 118, 0.12);
    color: var(--success);
  }

  .badge-best {
    background: rgba(255, 215, 0, 0.12);
    color: var(--glow-gold);
  }

  /* Actions */
  .actions {
    display: flex;
    gap: var(--space-3);
    flex-wrap: wrap;
  }

  .action-btn {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-5);
    border-radius: var(--radius-md);
    font-size: 0.9rem;
    font-weight: 600;
    cursor: pointer;
    border: none;
    transition: background 150ms var(--ease-snap), transform 150ms var(--ease-snap);
  }

  .action-btn:active {
    transform: scale(0.96);
  }

  .action-btn.primary {
    background: var(--glow-primary);
    color: var(--void-deep);
  }

  .action-btn.primary:hover {
    background: var(--glow-warm);
  }

  .action-btn.secondary {
    background: var(--void-raised);
    color: var(--text-bright);
    border: 1px solid var(--text-muted);
  }

  .action-btn.secondary:hover {
    border-color: var(--glow-primary);
  }

  /* Sections */
  .section {
    padding: var(--space-6);
    background: var(--void-mid);
    border: 1px solid rgba(255, 122, 61, 0.06);
    border-radius: var(--radius-lg);
  }

  .section-title {
    margin: 0 0 var(--space-4);
    font-family: var(--font-display);
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--text-bright);
  }

  /* Specs */
  .specs-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: var(--space-3);
  }

  .spec-card {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    padding: var(--space-3);
    border-radius: var(--radius-sm);
    background: var(--void-raised);
    transition: background 150ms var(--ease-snap);
  }

  .spec-card:hover {
    background: var(--void-hover);
  }

  .spec-label {
    font-size: 0.75rem;
    color: var(--text-dim);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .spec-value {
    font-family: var(--font-mono);
    font-size: 0.9rem;
    font-weight: 500;
    color: var(--text-bright);
  }

  @keyframes shimmer {
    0% { background-position: 200% 0; }
    100% { background-position: -200% 0; }
  }

  @media (max-width: 768px) {
    .detail-grid {
      grid-template-columns: 1fr;
    }

    .gallery {
      position: static;
    }
  }
</style>
