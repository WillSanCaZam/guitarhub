<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import PriceDisplay from '$lib/components/ui/PriceDisplay.svelte';
  import StarRating from '$lib/components/ui/StarRating.svelte';
  import SkeletonLoader from '$lib/components/ui/SkeletonLoader.svelte';
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
      <div class="thumbnails">
        {#each Array(5) as _, i}
          <button
            class="thumbnail"
            class:active={activeImage === i}
            onclick={() => activeImage = i}
            aria-label="View image {i + 1}"
          >
            <div class="thumb-placeholder"></div>
          </button>
        {/each}
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

      <StarRating rating={product.rating ?? 4.5} reviewCount={342} size="md" />

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

  <!-- Reviews -->
  <section class="section">
    <h2 class="section-title">Reviews</h2>
    <div class="reviews-list">
      <div class="review-card">
        <div class="review-header">
          <span class="review-author">Marcus T.</span>
          <div class="review-stars">
            {#each Array(5) as _, i}
              <svg class="star-icon" class:filled={i < 5} viewBox="0 0 20 20" width="16" height="16">
                <path d="M10 1l2.39 4.84 5.34.78-3.87 3.77.91 5.32L10 13.28l-4.77 2.43.91-5.32L2.27 6.62l5.34-.78z"/>
              </svg>
            {/each}
          </div>
        </div>
        <p class="review-text">Absolutely incredible tone. The neck pickup is warm and glassy, perfect for blues and jazz. The build quality is top-notch — worth every penny.</p>
        <div class="review-actions">
          <button class="vote-btn" aria-label="Upvote review">
            <svg viewBox="0 0 20 20" width="14" height="14"><path d="M10 3l-7 7h4v7h6v-7h4z"/></svg>
            <span>12</span>
          </button>
          <button class="vote-btn" aria-label="Downvote review">
            <svg viewBox="0 0 20 20" width="14" height="14"><path d="M10 17l7-7h-4V3H7v7H3z"/></svg>
            <span>1</span>
          </button>
        </div>
      </div>

      <div class="review-card">
        <div class="review-header">
          <span class="review-author">Sarah K.</span>
          <div class="review-stars">
            {#each Array(5) as _, i}
              <svg class="star-icon" class:filled={i < 4} viewBox="0 0 20 20" width="16" height="16">
                <path d="M10 1l2.39 4.84 5.34.78-3.87 3.77.91 5.32L10 13.28l-4.77 2.43.91-5.32L2.27 6.62l5.34-.78z"/>
              </svg>
            {/each}
          </div>
        </div>
        <p class="review-text">Great guitar for the price range. The tremolo system stays in tune well. Only complaint is the finish could be a bit more refined on the back of the neck.</p>
        <div class="review-actions">
          <button class="vote-btn" aria-label="Upvote review">
            <svg viewBox="0 0 20 20" width="14" height="14"><path d="M10 3l-7 7h4v7h6v-7h4z"/></svg>
            <span>8</span>
          </button>
          <button class="vote-btn" aria-label="Downvote review">
            <svg viewBox="0 0 20 20" width="14" height="14"><path d="M10 17l7-7h-4V3H7v7H3z"/></svg>
            <span>0</span>
          </button>
        </div>
      </div>

      <div class="review-card">
        <div class="review-header">
          <span class="review-author">Jake R.</span>
          <div class="review-stars">
            {#each Array(5) as _, i}
              <svg class="star-icon" class:filled={i < 5} viewBox="0 0 20 20" width="16" height="16">
                <path d="M10 1l2.39 4.84 5.34.78-3.87 3.77.91 5.32L10 13.28l-4.77 2.43.91-5.32L2.27 6.62l5.34-.78z"/>
              </svg>
            {/each}
          </div>
        </div>
        <p class="review-text">This is my daily driver now. The pickups are hot enough for rock but clean up beautifully. The roasted maple neck feels incredible. Best purchase I've made this year.</p>
        <div class="review-actions">
          <button class="vote-btn" aria-label="Upvote review">
            <svg viewBox="0 0 20 20" width="14" height="14"><path d="M10 3l-7 7h4v7h6v-7h4z"/></svg>
            <span>15</span>
          </button>
          <button class="vote-btn" aria-label="Downvote review">
            <svg viewBox="0 0 20 20" width="14" height="14"><path d="M10 17l7-7h-4V3H7v7H3z"/></svg>
            <span>2</span>
          </button>
        </div>
      </div>
    </div>
  </section>
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

  .thumbnails {
    display: flex;
    gap: var(--space-2);
    margin-top: var(--space-3);
  }

  .thumbnail {
    width: 80px;
    height: 60px;
    border-radius: var(--radius-sm);
    border: 2px solid transparent;
    background: var(--void-raised);
    cursor: pointer;
    padding: 0;
    overflow: hidden;
    transition: border-color 150ms var(--ease-snap);
  }

  .thumbnail:hover {
    border-color: var(--glow-soft);
  }

  .thumbnail.active {
    border-color: var(--glow-primary);
  }

  .thumb-placeholder {
    width: 100%;
    height: 100%;
    background: linear-gradient(90deg, var(--void-raised) 25%, var(--glow-soft) 50%, var(--void-raised) 75%);
    background-size: 200% 100%;
    animation: shimmer 1.5s infinite;
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
    background: var(--glow-success);
    color: var(--success);
  }

  .badge-best {
    background: var(--glow-gold-soft);
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
    border: 1px solid var(--border-subtle);
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

  /* Reviews */
  .reviews-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .review-card {
    padding: var(--space-4);
    background: var(--void-raised);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-subtle);
  }

  .review-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-2);
  }

  .review-author {
    font-family: var(--font-mono);
    font-weight: 600;
    font-size: 0.9rem;
    color: var(--text-bright);
  }

  .review-stars {
    display: flex;
    gap: 2px;
  }

  .star-icon {
    fill: var(--void-active);
  }

  .star-icon.filled {
    fill: var(--glow-primary);
  }

  .review-text {
    margin: 0 0 var(--space-3);
    font-size: 0.9rem;
    line-height: 1.6;
    color: var(--text-warm);
  }

  .review-actions {
    display: flex;
    gap: var(--space-3);
  }

  .vote-btn {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    background: none;
    border: 1px solid var(--text-muted);
    border-radius: var(--radius-sm);
    padding: var(--space-1) var(--space-2);
    color: var(--text-dim);
    font-size: 0.75rem;
    cursor: pointer;
    transition: border-color 150ms var(--ease-snap), color 150ms var(--ease-snap);
  }

  .vote-btn:hover {
    border-color: var(--glow-primary);
    color: var(--glow-primary);
  }

  .vote-btn svg {
    fill: currentColor;
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
