<script lang="ts">
  import { onMount } from 'svelte';
  import { wishlistState, loadWishlist } from '$lib/stores/wishlist.svelte';
  import ProductCard from '$lib/components/ProductCard.svelte';

  onMount(() => {
    loadWishlist();
  });

  const totalValue = $derived(
    wishlistState.items.reduce((sum, item) => sum + (item.price ?? 0), 0)
  );
</script>

<div class="page">
  <header class="wishlist-header">
    <a href="/" class="back-link">← Dashboard</a>
    <div class="header-content">
      <h1>My Wishlist</h1>
      <span class="count">{wishlistState.items.length} items · {totalValue.toLocaleString()} USD</span>
    </div>
  </header>

  {#if wishlistState.loading}
    <div class="skeleton-grid">
      {#each Array(6) as _, i}
        <div class="skeleton-card" style="animation-delay: {i * 50}ms">
          <div class="skeleton-image"></div>
          <div class="skeleton-text"></div>
          <div class="skeleton-text short"></div>
        </div>
      {/each}
    </div>
  {:else if wishlistState.error}
    <p class="error">{wishlistState.error}</p>
  {:else if wishlistState.items.length === 0}
    <div class="empty-state">
      <svg width="80" height="80" viewBox="0 0 80 80" fill="none">
        <path d="M40 70L10 42C4 36 4 26 10 20C16 14 26 14 32 20L40 28L48 20C54 14 64 14 70 20C76 26 76 36 70 42L40 70Z" 
              stroke="var(--color-outline)" stroke-width="2" fill="none"/>
      </svg>
      <h2>No favorites yet</h2>
      <p>Search for gear and tap the heart to save it here.</p>
      <a href="/" class="empty-cta">Browse Gear</a>
    </div>
  {:else}
    <div class="wishlist-grid">
      {#each wishlistState.items as item (item.id)}
        <ProductCard
          product={{
            sku: item.sku ?? `wishlist-${item.id}`,
            name: item.name ?? item.sku ?? 'Unknown',
            brand: item.brand ?? '',
            price: item.price ?? 0,
            currency: item.currency ?? 'USD',
            url: item.product_url ?? undefined,
            image_url: item.image_url ?? undefined,
          }}
        />
      {/each}
    </div>
  {/if}
</div>

<style>
  .page {
    max-width: 1200px;
    margin: 0 auto;
    padding: var(--space-lg);
  }

  .wishlist-header {
    margin-bottom: var(--space-xl);
  }

  .back-link {
    display: inline-block;
    margin-bottom: var(--space-sm);
    color: var(--text-dim);
    text-decoration: none;
    font-size: 0.85rem;
    transition: color var(--transition-fast);
  }

  .back-link:hover {
    color: var(--glow-primary);
  }

  .header-content {
    display: flex;
    align-items: baseline;
    gap: var(--space-md);
  }

  h1 {
    margin: 0;
    font-family: var(--font-display);
    font-size: 1.75rem;
    font-weight: 700;
    color: var(--text-bright);
  }

  .count {
    color: var(--text-dim);
    font-size: 0.9rem;
  }

  .error {
    color: var(--danger);
    font-size: 0.95rem;
  }

  .wishlist-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: var(--space-5);
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-3xl) var(--space-lg);
    text-align: center;
  }

  .empty-state h2 {
    margin: 0;
    font-family: var(--font-display);
    font-size: 1.25rem;
    color: var(--text-bright);
  }

  .empty-state p {
    margin: 0;
    color: var(--text-dim);
    font-size: 0.9rem;
  }

  .empty-cta {
    display: inline-flex;
    align-items: center;
    padding: var(--space-sm) var(--space-lg);
    background: var(--glow-primary);
    color: var(--void-deep);
    border-radius: var(--radius-pill);
    text-decoration: none;
    font-weight: 600;
    font-size: 0.9rem;
    transition: transform var(--transition-fast), box-shadow var(--transition-fast);
  }

  .empty-cta:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
  }

  .skeleton-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: var(--space-5);
  }

  .skeleton-card {
    border: 1px solid var(--color-outline);
    border-radius: var(--radius-md);
    overflow: hidden;
    background: var(--color-surface);
    animation: fadeIn 0.3s ease both;
  }

  .skeleton-image {
    width: 100%;
    aspect-ratio: 16 / 10;
    background: linear-gradient(90deg, var(--color-surface-container) 25%, var(--color-surface-container-high) 50%, var(--color-surface-container) 75%);
    background-size: 200% 100%;
    animation: shimmer 1.5s infinite;
  }

  .skeleton-text {
    height: 14px;
    margin: 12px 16px 0;
    background: var(--color-surface-container);
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