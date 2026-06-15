<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import StarRating from './ui/StarRating.svelte';
  import PriceDisplay from './ui/PriceDisplay.svelte';
  import { addToCollection } from '$lib/stores/collection.svelte';
  import { wishlistState, addToWishlist, removeFromWishlist } from '$lib/stores/wishlist.svelte';
  import type { PriceInsight } from '$lib/types/price';

  interface ProductCardProduct {
    sku: string;
    name: string;
    brand: string;
    model?: string;
    price: number;
    original_price?: number;
    currency?: string;
    image_url?: string;
    url?: string;
    condition?: string;
    store_logo_url?: string;
    category?: string;
    discount_pct?: number;
    artist_badge?: string;
    rating?: number;
    review_count?: number;
    viewers_count?: number;
    in_stock?: boolean;
    is_best_price?: boolean;
  }

  interface Props {
    product: ProductCardProduct;
    inCollection?: boolean;
  }

  let { product, inCollection = false }: Props = $props();

  let imageData = $state<string>('');
  let imageError = $state<boolean>(false);
  let priceInsight = $state<PriceInsight | null>(null);
  let adding = $state<boolean>(false);
  let added = $state<boolean>(false);
  let imageLoaded = $state<boolean>(false);
  let hovered = $state<boolean>(false);

  const isInWishlist = $derived(
    wishlistState.items.some(item => item.sku === product.sku)
  );

  onMount(async () => {
    try {
      imageData = await invoke<string>('get_product_image', { imageUrl: product.image_url });
      imageLoaded = true;
    } catch (e) {
      console.error('Failed to load product image:', e);
      imageError = true;
    }
    try {
      priceInsight = await invoke<PriceInsight | null>('get_price_insight', { sku: product.sku });
    } catch (e) {
      // silent fail — badge is optional
    }
  });

  async function handleAdd() {
    if (adding || added) return;
    adding = true;
    try {
      await addToCollection(product);
      added = true;
      setTimeout(() => { added = false; }, 2000);
    } catch (e) {
      console.error('Failed to add to collection:', e);
    } finally {
      adding = false;
    }
  }

  async function handleWishlistToggle() {
    if (isInWishlist) {
      const item = wishlistState.items.find(i => i.sku === product.sku);
      if (item) {
        await removeFromWishlist(item.id);
      }
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

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div
  class="product-card"
  tabindex="0"
  role="article"
  aria-labelledby="title-{product.sku}"
  onmouseenter={() => hovered = true}
  onmouseleave={() => hovered = false}
>
  <!-- Image Area -->
  <div class="image-container">
    {#if imageData}
      <img src={imageData} alt={product.name} class="product-image" loading="lazy" />
    {:else}
      <div class="shimmer skeleton" aria-label={product.name}></div>
    {/if}

    <!-- Store Logo (top-left) -->
    {#if product.store_logo_url}
      <span class="store-logo">
        <img src={product.store_logo_url} alt="" />
      </span>
    {/if}

    <!-- Category Pill (top-right) -->
    {#if product.category}
      <span class="category-pill">{product.category}</span>
    {/if}

    <!-- Deal Badge (absolute top-right) -->
    {#if product.discount_pct && product.discount_pct > 0}
      <span class="deal-badge">-{product.discount_pct}%</span>
    {/if}

    <!-- Artist Badge -->
    {#if product.artist_badge}
      <span class="artist-badge">{product.artist_badge}</span>
    {/if}

    <!-- Quick Actions (hover) -->
    {#if hovered}
      <div class="quick-actions">
        <button
          class="quick-action"
          onclick={handleWishlistToggle}
          aria-label={isInWishlist ? 'Remove from wishlist' : 'Add to wishlist'}
        >
          <svg viewBox="0 0 24 24" fill={isInWishlist ? 'var(--glow-hot)' : 'none'} stroke="currentColor" stroke-width="2">
            <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/>
          </svg>
        </button>
        {#if product.url}
          <button class="quick-action" onclick={handleOpenUrl} aria-label="View deal">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>
              <polyline points="15 3 21 3 21 9"/>
              <line x1="10" y1="14" x2="21" y2="3"/>
            </svg>
          </button>
        {/if}
      </div>
    {/if}

    <!-- Play overlay -->
    {#if hovered && imageData}
      <div class="play-overlay">
        <span class="play-text">Play Hear It</span>
      </div>
    {/if}
  </div>

  <!-- Product Info -->
  <div class="product-info">
    <h3 class="product-title" id="title-{product.sku}">{product.name}</h3>
    {#if product.brand}
      <p class="product-brand">{product.brand}{product.model ? ` ${product.model}` : ''}</p>
    {/if}

    {#if product.rating !== undefined}
      <StarRating rating={product.rating} reviewCount={product.review_count} size="sm" />
    {/if}

    {#if product.viewers_count && product.viewers_count > 0}
      <span class="viewers">{product.viewers_count} people viewing now</span>
    {/if}

    <div class="price-row">
      <PriceDisplay
        price={product.price}
        originalPrice={product.original_price}
        discountPct={product.discount_pct}
        currency={product.currency}
      />
    </div>

    <div class="status-badges">
      {#if product.in_stock !== false}
        <span class="badge badge-stock">In Stock</span>
      {:else}
        <span class="badge badge-oos">Out of Stock</span>
      {/if}
      {#if product.is_best_price}
        <span class="badge badge-best">Best Price</span>
      {/if}
    </div>

    <div class="product-actions">
      {#if product.url}
        <button class="action-btn store-link" onclick={handleOpenUrl} data-testid="store-link" aria-label="View deal in store">
          View Deals
        </button>
      {/if}
      {#if !inCollection}
        <button class="action-btn add-btn" onclick={handleAdd} disabled={adding} data-testid="add-to-collection">
          {#if added}
            Added ✓
          {:else if adding}
            Adding...
          {:else}
            + Collection
          {/if}
        </button>
      {/if}
    </div>
  </div>
</div>

<style>
  .product-card {
    border: 1px solid rgba(255, 122, 61, 0.06);
    border-radius: var(--radius-lg);
    overflow: hidden;
    background: var(--void-mid);
    box-shadow: var(--shadow-card);
    display: flex;
    flex-direction: column;
    transition: transform 350ms var(--ease-plug),
                box-shadow 350ms var(--ease-plug),
                border-color 350ms var(--ease-plug);
    position: relative;
    min-width: 300px;
    max-width: 400px;
  }

  .product-card:hover {
    transform: scale(1.03);
    box-shadow: var(--shadow-hover);
    border-color: rgba(255, 122, 61, 0.2);
  }

  .product-card:focus-visible {
    outline: 2px solid var(--glow-primary);
    outline-offset: 2px;
  }

  /* Image */
  .image-container {
    position: relative;
    width: 100%;
    aspect-ratio: 16 / 10;
    background: var(--void-deep);
    overflow: hidden;
  }

  .product-image {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
    transition: transform 350ms var(--ease-plug);
  }

  .product-card:hover .product-image {
    transform: scale(1.05);
  }

  .shimmer {
    width: 100%;
    height: 100%;
    background: linear-gradient(90deg, var(--void-raised) 25%, var(--glow-soft) 50%, var(--void-raised) 75%);
    background-size: 200% 100%;
    animation: shimmer 1.5s infinite;
  }

  @keyframes shimmer {
    0% { background-position: 200% 0; }
    100% { background-position: -200% 0; }
  }

  /* Store Logo */
  .store-logo {
    position: absolute;
    top: var(--space-2);
    left: var(--space-2);
    width: 28px;
    height: 28px;
    border-radius: var(--radius-sm);
    background: var(--void-mid);
    padding: 2px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px solid rgba(255, 255, 255, 0.06);
  }

  .store-logo img {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }

  /* Category Pill */
  .category-pill {
    position: absolute;
    top: var(--space-2);
    right: var(--space-2);
    padding: 2px var(--space-2);
    border-radius: var(--radius-pill);
    background: var(--void-mid);
    color: var(--text-warm);
    font-size: 0.65rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    border: 1px solid rgba(255, 255, 255, 0.06);
  }

  /* Deal Badge */
  .deal-badge {
    position: absolute;
    top: var(--space-10);
    right: var(--space-2);
    padding: 2px var(--space-2);
    border-radius: var(--radius-sm);
    background: var(--glow-hot);
    color: white;
    font-size: 0.7rem;
    font-weight: 700;
  }

  /* Artist Badge */
  .artist-badge {
    position: absolute;
    bottom: var(--space-2);
    left: var(--space-2);
    padding: 2px var(--space-2);
    border-radius: var(--radius-sm);
    background: var(--glow-medium);
    color: var(--text-bright);
    font-size: 0.65rem;
    font-weight: 600;
  }

  /* Quick Actions */
  .quick-actions {
    position: absolute;
    top: var(--space-2);
    right: var(--space-12);
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    animation: fadeIn 200ms var(--ease-plug);
  }

  .quick-action {
    width: 32px;
    height: 32px;
    border-radius: var(--radius-pill);
    background: var(--void-mid);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: var(--text-bright);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: background 150ms var(--ease-snap), transform 150ms var(--ease-strum);
  }

  .quick-action:hover {
    background: var(--void-hover);
    transform: scale(1.1);
  }

  .quick-action svg {
    width: 16px;
    height: 16px;
  }

  /* Play Overlay */
  .play-overlay {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(7, 7, 12, 0.6);
    animation: fadeIn 200ms var(--ease-plug);
  }

  .play-text {
    padding: var(--space-2) var(--space-4);
    border-radius: var(--radius-pill);
    background: var(--glow-primary);
    color: var(--void-deep);
    font-size: 0.8rem;
    font-weight: 700;
    letter-spacing: 0.02em;
  }

  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  /* Info */
  .product-info {
    padding: var(--space-4);
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    flex: 1;
  }

  .product-title {
    margin: 0;
    font-family: var(--font-display);
    font-size: 1rem;
    font-weight: 700;
    line-height: 1.3;
    color: var(--text-bright);
  }

  .product-brand {
    margin: 0;
    font-family: var(--font-body);
    font-size: 0.85rem;
    color: var(--text-warm);
  }

  .viewers {
    font-size: 0.75rem;
    color: var(--glow-cool);
    font-weight: 500;
  }

  .price-row {
    margin-top: var(--space-2);
  }

  .status-badges {
    display: flex;
    gap: var(--space-2);
    margin-top: var(--space-1);
  }

  .badge {
    font-size: 0.65rem;
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

  .badge-oos {
    background: rgba(255, 23, 68, 0.12);
    color: var(--danger);
  }

  .badge-best {
    background: rgba(255, 215, 0, 0.12);
    color: var(--glow-gold);
  }

  /* Actions */
  .product-actions {
    display: flex;
    gap: var(--space-2);
    margin-top: auto;
    padding-top: var(--space-2);
  }

  .action-btn {
    flex: 1;
    padding: var(--space-2) var(--space-3);
    border: none;
    border-radius: var(--radius-sm);
    font-size: 0.8rem;
    font-weight: 600;
    cursor: pointer;
    transition: background 150ms var(--ease-snap), transform 150ms var(--ease-snap);
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-1);
  }

  .action-btn:active {
    transform: scale(0.96);
  }

  .store-link {
    background: var(--glow-primary);
    color: var(--void-deep);
  }

  .store-link:hover {
    background: var(--glow-warm);
  }

  .add-btn {
    background: var(--void-raised);
    color: var(--text-bright);
    border: 1px solid var(--text-muted);
  }

  .add-btn:hover:not(:disabled) {
    background: var(--void-hover);
    border-color: var(--glow-primary);
  }

  .add-btn:disabled {
    opacity: 0.7;
    cursor: not-allowed;
  }
</style>
