<script lang="ts">
  interface Props {
    trending: string[];
    onSearch?: (query: string) => void;
  }

  let { trending, onSearch }: Props = $props();
</script>

<div class="trending-section" role="list" aria-label="Trending searches">
  <span class="trending-label" aria-hidden="true">
    <span class="fire-icon" aria-hidden="true">🔥</span>
    Trending
  </span>
  <div class="trending-scroll">
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
</div>

<style>
  .trending-section {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    margin-bottom: var(--space-4);
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

  .fire-icon {
    font-size: 0.9rem;
  }

  .trending-scroll {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    overflow-x: auto;
    scroll-snap-type: x mandatory;
    -webkit-overflow-scrolling: touch;
    scrollbar-width: none;
    padding: var(--space-1) 0;
    scroll-behavior: smooth;
  }

  .trending-scroll::-webkit-scrollbar {
    display: none;
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
    border: 1px solid var(--border-subtle);
    cursor: pointer;
    white-space: nowrap;
    scroll-snap-align: start;
    transition: background 150ms var(--ease-snap), color 150ms var(--ease-snap), border-color 150ms var(--ease-snap), box-shadow 150ms var(--ease-snap);
  }

  .trending-pill:hover {
    background: var(--void-hover);
    color: var(--text-bright);
    border-color: var(--border-glow);
    box-shadow: 0 0 12px var(--glow-soft);
  }

  @media (prefers-reduced-motion: reduce) {
    .trending-pill {
      transition: none;
    }
    .trending-scroll {
      scroll-behavior: auto;
    }
  }
</style>
