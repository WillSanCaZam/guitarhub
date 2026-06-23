<script lang="ts">
  import { onMount } from 'svelte';
  import { wishlistState, loadWishlist } from '$lib/stores/wishlist.svelte';
  import GearCard from '$lib/components/GearCard.svelte';
  import SkeletonLoader from '$lib/components/ui/SkeletonLoader.svelte';
  import EmptyState from '$lib/components/ui/EmptyState.svelte';

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
    <SkeletonLoader variant="card-grid" count={6} />
  {:else if wishlistState.error}
    <p class="error">{wishlistState.error}</p>
  {:else if wishlistState.items.length === 0}
    <EmptyState
      variant="wishlist"
      title="No favorites yet"
      description="Search for gear and tap the heart to save it here."
      actionLabel="Browse Gear"
      onAction={() => window.location.href = '/'}
    />
  {:else}
    <div class="wishlist-grid">
      {#each wishlistState.items as item (item.id)}
        <GearCard
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

  .error {
    color: var(--danger);
    font-size: 0.95rem;
  }
</style>