<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import PriceBadge from './PriceBadge.svelte';
  import { addToCollection } from '$lib/stores/collection.svelte';
  import type { PriceInsight } from '$lib/types/price';

  interface ProductCardProduct {
    sku: string;
    name: string;
    brand: string;
    model?: string;
    price: number;
    currency?: string;
    image_url?: string;
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

  onMount(async () => {
    // Load image first
    try {
      imageData = await invoke<string>('get_product_image', { imageUrl: product.image_url });
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
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div class="product-card" tabindex="0">
  {#if imageData}
    <img src={imageData} alt={product.name} class="product-image" />
  {:else}
    <div class="placeholder" aria-label={product.name}>
      {#if imageError}
        <span class="error-icon">!</span>
      {:else}
        <span class="loading-icon">...</span>
      {/if}
    </div>
  {/if}
  <div class="product-info">
    <h3>{product.name}</h3>
    {#if product.brand}
      <p class="brand">{product.brand}</p>
    {/if}
    {#if product.price}
      <p class="price">
        {product.price} {product.currency ?? ''}
        {#if priceInsight && priceInsight.level !== 'hidden'}
          <PriceBadge level={priceInsight.level} pct={priceInsight.pct} confidence={priceInsight.confidence} />
        {/if}
      </p>
    {/if}
    {#if !inCollection}
      <button
        class="add-btn"
        onclick={handleAdd}
        disabled={adding}
        aria-label={added ? 'Added to collection' : 'Add to collection'}
        data-testid="add-to-collection"
      >
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

<style>
  .product-card {
    border: 1px solid var(--color-outline);
    border-radius: 8px;
    overflow: hidden;
    background: var(--color-on-surface);
    transition: box-shadow 0.2s;
  }
  .product-card:hover {
    box-shadow: 0 2px 8px rgba(0,0,0,0.12);
  }
  .product-card:focus-visible {
    outline: 2px solid var(--color-secondary);
    outline-offset: 2px;
  }
  .product-image {
    width: 100%;
    height: 200px;
    object-fit: cover;
    display: block;
  }
  .placeholder {
    width: 100%;
    height: 200px;
    background: var(--color-surface);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 2rem;
    color: var(--color-on-surface-variant);
  }
  .product-info {
    padding: 12px;
  }
  .product-info h3 {
    margin: 0 0 4px;
    font-size: 1rem;
  }
  .brand {
    color: var(--color-on-surface-muted);
    font-size: 0.85rem;
    margin: 0 0 4px;
  }
  .price {
    font-weight: 600;
    margin: 0;
  }

  .add-btn {
    margin-top: 8px;
    padding: 8px 12px;
    background: var(--color-secondary);
    color: var(--color-on-surface);
    border: none;
    border-radius: 6px;
    font-size: 0.85rem;
    cursor: pointer;
    width: 100%;
    transition: background 0.15s;
  }

  .add-btn:hover:not(:disabled) {
    background: var(--color-secondary);
  }

  .add-btn:disabled {
    opacity: 0.7;
    cursor: not-allowed;
  }

  @media (prefers-color-scheme: dark) {
    .add-btn {
      background: var(--color-secondary);
      color: var(--color-on-surface);
    }

    .add-btn:hover:not(:disabled) {
      background: var(--color-secondary-hover);
    }
  }
</style>
