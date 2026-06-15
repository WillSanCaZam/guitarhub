<script lang="ts">
  interface Props {
    trending: string[];
    onSearch?: (query: string) => void;
  }

  let { trending, onSearch }: Props = $props();
</script>

<div class="trending-pills" role="list" aria-label="Trending searches">
  <span class="trending-label" aria-hidden="true">
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
    </svg>
    Trending
  </span>
  {#each trending as query}
    <button
      class="trending-pill"
      onclick={() => onSearch?.(query)}
    >
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="8"/>
        <path d="m21 21-4.35-4.35"/>
      </svg>
      {query}
    </button>
  {/each}
</div>

<style>
  .trending-pills {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    overflow-x: auto;
    scroll-snap-type: x mandatory;
    -webkit-overflow-scrolling: touch;
    padding: var(--space-1) 0;
    scrollbar-width: none;
  }

  .trending-pills::-webkit-scrollbar {
    display: none;
  }

  .trending-label {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    color: var(--glow-primary);
    font-size: 0.8rem;
    font-weight: 600;
    white-space: nowrap;
    flex-shrink: 0;
  }

  .trending-pill {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-pill);
    background: var(--void-raised);
    color: var(--text-warm);
    font-size: 0.8rem;
    font-weight: 500;
    border: 1px solid rgba(255, 255, 255, 0.04);
    cursor: pointer;
    white-space: nowrap;
    scroll-snap-align: start;
    transition: background 150ms var(--ease-snap), color 150ms var(--ease-snap);
  }

  .trending-pill:hover {
    background: var(--void-hover);
    color: var(--text-bright);
    border-color: rgba(255, 122, 61, 0.15);
  }
</style>
