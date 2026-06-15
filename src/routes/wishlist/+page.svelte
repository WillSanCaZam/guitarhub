<script lang="ts">
  import { onMount } from 'svelte';
  import { wishlistState, loadWishlist, removeFromWishlist } from '$lib/stores/wishlist.svelte';

  onMount(() => {
    loadWishlist();
  });

  function handleRemove(id: number) {
    removeFromWishlist(id);
  }
</script>

<div class="page">
  <header class="wishlist-header">
    <a href="/" class="back-link">&larr; Back to Dashboard</a>
    <h1>Wishlist</h1>
  </header>

  {#if wishlistState.loading}
    <p class="status">Loading&hellip;</p>
  {:else if wishlistState.error}
    <p class="error">{wishlistState.error}</p>
  {:else if wishlistState.items.length === 0}
    <p class="empty-state">Your wishlist is empty</p>
  {:else}
    <ul class="wishlist-list">
      {#each wishlistState.items as item (item.id)}
        <li class="wishlist-item">
          <div class="item-info">
            <span class="item-name">{item.name ?? item.sku ?? 'Unknown'}</span>
            {#if item.brand}
              <span class="item-brand">{item.brand}</span>
            {/if}
            {#if item.price != null}
              <span class="item-price">{item.price}{item.currency ? ` ${item.currency}` : ''}</span>
            {/if}
          </div>
          <button class="remove-btn" onclick={() => handleRemove(item.id)}>Remove</button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .page {
    max-width: 1200px;
    margin: 0 auto;
    padding: 16px;
  }

  .wishlist-header {
    margin-bottom: 24px;
  }

  .back-link {
    display: inline-block;
    margin-bottom: 12px;
    color: var(--color-secondary);
    text-decoration: none;
    font-size: 0.9rem;
  }

  .back-link:hover {
    text-decoration: underline;
  }

  h1 {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--color-secondary);
  }

  .status,
  .empty-state {
    color: var(--color-on-surface-muted);
    font-size: 0.95rem;
  }

  .error {
    color: var(--color-error);
  }

  .wishlist-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .wishlist-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid var(--color-outline-variant);
  }

  .item-info {
    display: flex;
    gap: 12px;
    align-items: center;
    flex-wrap: wrap;
  }

  .item-name {
    font-weight: 600;
    color: var(--color-surface-container-high);
  }

  .item-brand {
    color: var(--color-on-surface-muted);
    font-size: 0.85rem;
  }

  .item-price {
    color: var(--color-on-surface-variant);
    font-size: 0.9rem;
  }

  .remove-btn {
    padding: 4px 12px;
    border: 1px solid var(--color-error);
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--color-error);
    cursor: pointer;
    font-size: 0.8rem;
    transition: background var(--transition-fast);
  }

  .remove-btn:hover {
    background: var(--color-error);
    color: var(--color-on-surface);
  }

  @media (prefers-color-scheme: dark) {
    h1 {
      color: var(--color-on-surface);
    }

    .back-link {
      color: var(--color-info);
    }

    .wishlist-item {
      border-bottom-color: var(--color-surface-container-high);
    }

    .item-name {
      color: var(--color-outline);
    }

    .item-brand {
      color: var(--color-on-surface-variant);
    }

    .item-price {
      color: var(--color-on-surface-variant);
    }

    .status,
    .empty-state {
      color: var(--color-on-surface-variant);
    }
  }
</style>