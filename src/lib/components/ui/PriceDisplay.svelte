<script lang="ts">
  interface Props {
    price: number;
    originalPrice?: number;
    discountPct?: number;
    currency?: string;
    size?: 'sm' | 'md' | 'lg';
  }

  let { price, originalPrice, discountPct, currency = 'USD', size = 'md' }: Props = $props();

  const formattedPrice = $derived(
    new Intl.NumberFormat('en-US', { style: 'currency', currency, maximumFractionDigits: 0 }).format(price)
  );

  const formattedOriginal = $derived(
    originalPrice
      ? new Intl.NumberFormat('en-US', { style: 'currency', currency, maximumFractionDigits: 0 }).format(originalPrice)
      : null
  );

  const calculatedDiscount = $derived(
    discountPct ?? (originalPrice ? Math.round(((originalPrice - price) / originalPrice) * 100) : null)
  );
</script>

<span class="price-display" class:has-discount={!!calculatedDiscount}>
  <span class="current-price">{formattedPrice}</span>
  {#if formattedOriginal}
    <span class="original-price">{formattedOriginal}</span>
  {/if}
  {#if calculatedDiscount}
    <span class="discount-badge">-{calculatedDiscount}%</span>
  {/if}
</span>

<style>
  .price-display {
    display: inline-flex;
    align-items: baseline;
    gap: var(--space-2);
    flex-wrap: wrap;
  }

  .current-price {
    font-family: var(--font-mono);
    font-weight: 700;
    font-size: 1.25rem;
    color: var(--glow-primary);
    letter-spacing: -0.01em;
  }

  .original-price {
    font-family: var(--font-mono);
    font-size: 0.9rem;
    color: var(--text-dim);
    text-decoration: line-through;
  }

  .discount-badge {
    font-family: var(--font-body);
    font-size: 0.75rem;
    font-weight: 700;
    color: var(--void-deep);
    background: var(--glow-hot);
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    letter-spacing: 0.02em;
  }
</style>
