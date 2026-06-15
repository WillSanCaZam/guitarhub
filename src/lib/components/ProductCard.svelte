<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import PriceBadge from './PriceBadge.svelte';
  import { addToCollection } from '$lib/stores/collection.svelte';
  import { wishlistState, addToWishlist, removeFromWishlist } from '$lib/stores/wishlist.svelte';
  import type { PriceInsight } from '$lib/types/price';

  interface ProductCardProduct {
    sku: string;
    name: string;
    brand: string;
    model?: string;
    price: number;
    currency?: string;
    image_url?: string;
    url?: string;
    condition?: string;
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

  const isInWishlist = $derived(
    wishlistState.items.some(item => item.sku === product.sku)
  );

  onMount(async () => {
    // Load image first
    try {
      imageData = await invoke<string>('get_product_image', { imageUrl: product.image_url });
      imageLoaded = true;
    } catch (e) {
      console.error('Failed to load product image:', e);
      imageError = true;
    }
    // Fetch price insight after product loads (avoid cascading)
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
<div class="product-card" tabindex="0">
  <div class="image-container">
    {#if imageData}
      <img src={imageData} alt={product.name} class="product-image" />
    {:else}
      <div class="shimmer skeleton" aria-label={product.name}></div>
    {/if}
    {#if product.condition}
      <span class="condition-badge">{product.condition}</span>
    {/if}
  </div>
  <div class="product-info">
    <h3 class="product-title">{product.name}</h3>
    {#if product.brand}
      <p class="product-brand">{product.brand}</p>
    {/if}
    {#if product.price}
      <p class="product-price">
        {product.price} {product.currency ?? ''}
        {#if priceInsight && priceInsight.level !== 'hidden'}
          <PriceBadge level={priceInsight.level} pct={priceInsight.pct} confidence={priceInsight.confidence} />
        {/if}
      </p>
    {/if}
    <div class="product-actions">
      {#if product.url}
        <button class="action-btn store-link" onclick={handleOpenUrl} data-testid="store-link" aria-label="Ver oferta en tienda">
          Ver oferta →
        </button>
      {/if}
      <button class="action-btn wishlist-toggle" onclick={handleWishlistToggle} data-testid="wishlist-toggle" aria-label={isInWishlist ? 'Remove from wishlist' : 'Add to wishlist'}>
        <svg viewBox="0 0 24 24" fill={isInWishlist ? 'currentColor' : 'none'} stroke="currentColor" stroke-width="2">
          <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/>
        </svg>
      </button>
      {#if !inCollection}
        <button class="action-btn add-btn" onclick={handleAdd} disabled={adding} data-testid="add-to-collection">
          {#if added}
            Added ✓
          {:else if adding}
            Adding...
          {:else}
            Add to collection
          {/if}
        </button>
      {/if}
    </div>
  </div>
</div>

<style>
  .product-card {
    border: 1px solid var(--color-outline);
    border-radius: var(--radius-md);
    overflow: hidden;
    background: var(--color-surface);
    transition: box-shadow var(--transition-base);
    display: flex;
    flex-direction: column;
  }
  .product-card:hover {
    box-shadow: var(--shadow-card-hover);
  }
  .product-card:focus-visible {
    outline: 2px solid var(--color-primary);
    outline-offset: 2px;
  }
  .image-container {
    position: relative;
    width: 100%;
    aspect-ratio: 16 / 10;
    background: var(--color-surface-container);
  }
  .product-image {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }
  .shimmer {
    width: 100%;
    height: 100%;
    background: linear-gradient(90deg, var(--color-surface-container) 25%, var(--color-surface-container-high) 50%, var(--color-surface-container) 75%);
    background-size: 200% 100%;
    animation: shimmer 1.5s infinite;
  }
  @keyframes shimmer {
    0% { background-position: 200% 0; }
    100% { background-position: -200% 0; }
  }
  .condition-badge {
    position: absolute;
    top: var(--spacing-sm);
    right: var(--spacing-sm);
    padding: var(--spacing-2xs) var(--spacing-sm);
    border-radius: var(--radius-sm);
    background: var(--color-surface-container-high);
    color: var(--color-on-surface);
    font-size: 0.75rem;
    font-weight: 500;
    text-transform: capitalize;
  }
  .product-info {
    padding: var(--spacing-md);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
    flex: 1;
  }
  .product-title {
    margin: 0;
    font-family: var(--font-display);
    font-size: 1rem;
    font-weight: 700;
    line-height: 1.3;
  }
  .product-brand {
    margin: 0;
    font-family: var(--font-body);
    font-size: 0.85rem;
    color: var(--color-on-surface-muted);
  }
  .product-price {
    font-family: var(--font-mono);
    font-weight: 600;
    margin: 0;
    margin-top: auto;
  }
  .product-actions {
    display: flex;
    gap: var(--spacing-sm);
    margin-top: var(--spacing-sm);
  }
  .action-btn {
    flex: 1;
    padding: var(--spacing-sm) var(--spacing-md);
    border: none;
    border-radius: var(--radius-md);
    font-size: 0.85rem;
    cursor: pointer;
    transition: background var(--transition-fast);
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-xs);
  }
  .store-link {
    background: var(--color-primary);
    color: var(--color-on-primary);
  }
  .store-link:hover {
    background: var(--color-primary-hover);
  }
  .wishlist-toggle {
    background: var(--color-surface-container-high);
    color: var(--color-on-surface);
    flex: 0;
    padding: var(--spacing-sm);
  }
  .wishlist-toggle:hover {
    background: var(--color-surface-container-highest);
  }
  .wishlist-toggle svg {
    width: 20px;
    height: 20px;
  }
  .add-btn {
    background: var(--color-secondary);
    color: var(--color-on-secondary);
  }
  .add-btn:hover:not(:disabled) {
    background: var(--color-secondary-hover);
  }
  .add-btn:disabled {
    opacity: 0.7;
    cursor: not-allowed;
  }
</style>
