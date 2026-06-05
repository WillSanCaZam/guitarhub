<script>
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import PriceBadge from './PriceBadge.svelte';

  let { product } = $props();

  let imageData = $state('');
  let imageError = $state(false);
  let priceInsight = $state(null);

  onMount(async () => {
    // Load image first
    try {
      imageData = await invoke('get_product_image', { imageUrl: product.image_url });
    } catch (e) {
      console.error('Failed to load product image:', e);
      imageError = true;
    }
    // Fetch price insight after product loads (avoid cascading)
    try {
      priceInsight = await invoke('get_price_insight', { sku: product.sku });
    } catch (e) {
      // silent fail — badge is optional
    }
  });
</script>

<div class="product-card">
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
  </div>
</div>

<style>
  .product-card {
    border: 1px solid #ddd;
    border-radius: 8px;
    overflow: hidden;
    background: #fff;
    transition: box-shadow 0.2s;
  }
  .product-card:hover {
    box-shadow: 0 2px 8px rgba(0,0,0,0.12);
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
    background: #f0f0f0;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 2rem;
    color: #999;
  }
  .product-info {
    padding: 12px;
  }
  .product-info h3 {
    margin: 0 0 4px;
    font-size: 1rem;
  }
  .brand {
    color: #666;
    font-size: 0.85rem;
    margin: 0 0 4px;
  }
  .price {
    font-weight: 600;
    margin: 0;
  }
</style>
