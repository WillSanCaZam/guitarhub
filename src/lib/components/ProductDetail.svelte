<script lang="ts">
  import PriceChart from './PriceChart.svelte';

  interface ProductDetailProduct {
    sku: string;
    name: string;
    brand?: string;
    price?: number;
    currency?: string;
  }

  interface Props {
    product: ProductDetailProduct;
  }

  let { product }: Props = $props();
</script>

<div class="product-detail">
  <div class="detail-header">
    <h2>{product.name}</h2>
    {#if product.brand}
      <p class="brand">{product.brand}</p>
    {/if}
    {#if product.price}
      <p class="price">{product.price} {product.currency ?? ''}</p>
    {/if}
  </div>

  <div class="chart-section">
    <h3>Price History</h3>
    <PriceChart sku={product.sku} />
  </div>
</div>

<style>
  .product-detail {
    padding: 16px;
    max-width: 800px;
  }
  .detail-header {
    margin-bottom: 24px;
  }
  .detail-header h2 {
    margin: 0 0 4px;
    font-size: 1.5rem;
  }
  .brand {
    color: var(--color-on-surface-muted);
    font-size: 0.9rem;
    margin: 0 0 4px;
  }
  .price {
    font-weight: 600;
    font-size: 1.25rem;
    margin: 8px 0 0;
  }
  .chart-section h3 {
    margin: 0 0 8px;
    font-size: 1rem;
    color: var(--color-outline-variant);
  }
</style>
