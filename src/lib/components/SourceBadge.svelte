<script lang="ts">
  let {
    sourceId,
    userId,
  }: {
    /** Store source identifier (e.g. "reverb", "guitarcenter") */
    sourceId: string
    /** Non-null when the product belongs to a user-connected store */
    userId?: string | null
  } = $props()

  const sourceLabel = $derived.by(() => {
    switch (sourceId) {
      case 'reverb':
        return 'Reverb'
      case 'guitarcenter':
        return 'Guitar Center'
      default:
        return sourceId.charAt(0).toUpperCase() + sourceId.slice(1)
    }
  })
</script>

<span class="source-badge" class:user-product={!!userId}>
  {#if userId}
    <svg width="10" height="10" viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/></svg>
    Your Listing — via {sourceLabel}
  {:else}
    via {sourceLabel}
  {/if}
</span>

<style>
  .source-badge {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    padding: 2px var(--space-2);
    border-radius: var(--radius-pill);
    font-size: 0.65rem;
    font-weight: 600;
    line-height: 1.4;
    white-space: nowrap;
    background: var(--void-mid);
    color: var(--text-dim);
    border: 1px solid var(--border-subtle);
  }

  .source-badge.user-product {
    background: var(--glow-primary);
    color: var(--void-deep);
    border-color: var(--glow-primary);
  }
</style>
