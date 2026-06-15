<script lang="ts">
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import ProductDetail from '$lib/components/product/ProductDetail.svelte';
  import type { RawProduct } from '$lib/types/search';

  let product = $state<RawProduct | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  let sku = $state('');

  onMount(async () => {
    const unsub = page.subscribe(p => {
      sku = (p.params as Record<string, string>).sku ?? '';
    });
    if (!sku) { unsub(); return; }
    try {
      product = await invoke<RawProduct>('get_product_detail', { sku });
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
    unsub();
  });
</script>

<svelte:head>
  <title>{product ? `${product.name} — GuitarHub` : 'Loading... — GuitarHub'}</title>
</svelte:head>

<div class="product-page">
  <!-- Breadcrumb -->
  <nav class="breadcrumb" aria-label="Breadcrumb">
    <a href="/">Home</a>
    <span class="separator">›</span>
    {#if product}
      {#if product.category}
        <a href={`/explore?category=${encodeURIComponent(product.category)}`}>{product.category}</a>
        <span class="separator">›</span>
      {/if}
      <span class="current">{product.name}</span>
    {/if}
  </nav>

  {#if loading}
    <div class="loading-state" aria-busy="true">
      <div class="skeleton-detail">
        <div class="skeleton-gallery"></div>
        <div class="skeleton-info">
          <div class="skeleton-text wide"></div>
          <div class="skeleton-text"></div>
          <div class="skeleton-text short"></div>
        </div>
      </div>
    </div>
  {:else if error}
    <div class="error-state" role="alert">
      <p>Failed to load product: {error}</p>
      <a href="/">← Back to Home</a>
    </div>
  {:else if product}
    <ProductDetail {product} />
  {:else}
    <div class="error-state">
      <p>Product not found</p>
      <a href="/">← Back to Home</a>
    </div>
  {/if}
</div>

<style>
  .product-page {
    max-width: 1440px;
    margin: 0 auto;
    padding: var(--space-6);
  }

  .breadcrumb {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-6);
    font-size: 0.85rem;
    flex-wrap: wrap;
  }

  .breadcrumb a {
    color: var(--text-dim);
    text-decoration: none;
    transition: color 150ms var(--ease-snap);
  }

  .breadcrumb a:hover {
    color: var(--glow-primary);
  }

  .separator {
    color: var(--text-muted);
  }

  .current {
    color: var(--text-bright);
    font-weight: 500;
  }

  .loading-state, .error-state {
    padding: var(--space-16) var(--space-8);
    text-align: center;
  }

  .error-state p {
    color: var(--text-dim);
    font-size: 1.1rem;
    margin-bottom: var(--space-4);
  }

  .error-state a {
    color: var(--glow-primary);
    text-decoration: none;
  }

  .skeleton-detail {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-8);
  }

  .skeleton-gallery {
    aspect-ratio: 4 / 3;
    border-radius: var(--radius-lg);
    background: linear-gradient(90deg, var(--void-raised) 25%, var(--void-hover) 50%, var(--void-raised) 75%);
    background-size: 200% 100%;
    animation: shimmer 1.5s infinite;
  }

  .skeleton-info {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .skeleton-text {
    height: 18px;
    border-radius: 4px;
    background: linear-gradient(90deg, var(--void-raised) 25%, var(--void-hover) 50%, var(--void-raised) 75%);
    background-size: 200% 100%;
    animation: shimmer 1.5s infinite;
  }

  .skeleton-text.wide { width: 80%; }
  .skeleton-text.short { width: 40%; }

  @keyframes shimmer {
    0% { background-position: 200% 0; }
    100% { background-position: -200% 0; }
  }
</style>
