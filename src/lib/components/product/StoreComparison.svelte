<script lang="ts">
  interface Store {
    name: string;
    price: number;
    url: string;
    logo?: string;
  }

  interface Props {
    stores: Store[];
  }

  let { stores }: Props = $props();

  const sortedStores = $derived(
    [...stores].sort((a, b) => a.price - b.price)
  );

  const lowestPrice = $derived(sortedStores[0]?.price ?? 0);

  function openStore(url: string) {
    if (url && url !== '#') {
      window.open(url, '_blank', 'noopener');
    }
  }
</script>

<div class="store-comparison">
  <h3 class="comp-title">Where to Buy</h3>
  <div class="store-list">
    {#each sortedStores as store, i}
      <div class="store-row" class:cheapest={store.price === lowestPrice}>
        <div class="store-info">
          {#if store.logo}
            <img src={store.logo} alt="" class="store-logo" />
          {:else}
            <span class="store-icon">🏪</span>
          {/if}
          <span class="store-name">{store.name}</span>
        </div>
        <div class="store-price">
          <span class="price" class:best={store.price === lowestPrice}>
            ${store.price.toLocaleString()}
          </span>
          {#if store.price === lowestPrice}
            <span class="best-badge">Best Price</span>
          {/if}
        </div>
        <button class="go-btn" onclick={() => openStore(store.url)}>
          Go to Store →
        </button>
      </div>
    {/each}
  </div>
</div>

<style>
  .store-comparison {
    background: var(--void-raised);
    border-radius: var(--radius-md);
    padding: var(--space-4);
  }

  .comp-title {
    margin: 0 0 var(--space-3);
    font-family: var(--font-display);
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-bright);
  }

  .store-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .store-row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-3);
    border-radius: var(--radius-sm);
    background: var(--void-mid);
    transition: background 150ms var(--ease-snap);
  }

  .store-row:hover {
    background: var(--void-hover);
  }

  .store-row.cheapest {
    border: 1px solid rgba(0, 230, 118, 0.2);
  }

  .store-info {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex: 1;
  }

  .store-logo {
    width: 24px;
    height: 24px;
    object-fit: contain;
  }

  .store-icon {
    font-size: 1.2rem;
  }

  .store-name {
    font-weight: 500;
    color: var(--text-bright);
  }

  .store-price {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-width: 120px;
    justify-content: flex-end;
  }

  .price {
    font-family: var(--font-mono);
    font-weight: 600;
    color: var(--text-warm);
  }

  .price.best {
    color: var(--success);
    font-size: 1.1rem;
  }

  .best-badge {
    font-size: 0.6rem;
    font-weight: 700;
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    background: rgba(0, 230, 118, 0.12);
    color: var(--success);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .go-btn {
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-sm);
    background: var(--void-raised);
    color: var(--glow-primary);
    border: 1px solid rgba(255, 122, 61, 0.15);
    font-size: 0.8rem;
    font-weight: 600;
    cursor: pointer;
    transition: background 150ms var(--ease-snap), border-color 150ms var(--ease-snap);
  }

  .go-btn:hover {
    background: var(--glow-primary);
    color: var(--void-deep);
    border-color: var(--glow-primary);
  }
</style>
