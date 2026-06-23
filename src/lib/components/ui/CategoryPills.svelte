<script lang="ts">
  interface CategoryPill {
    id: string;
    label: string;
    icon: string;
    count?: number;
  }

  interface Props {
    categories: CategoryPill[];
    selected: string[];
    onToggle: (id: string) => void;
    multiSelect?: boolean;
  }

  let { categories, selected, onToggle, multiSelect = false }: Props = $props();
</script>

<div class="category-pills" role="group" aria-label="Categories">
  {#each categories as cat}
    {@const isActive = selected.includes(cat.id)}
    <button
      class="category-pill"
      class:active={isActive}
      onclick={() => onToggle(cat.id)}
      aria-pressed={isActive}
      aria-label="{cat.label}{cat.count !== undefined ? `, ${cat.count} items` : ''}"
    >
      <span class="cat-icon">{cat.icon}</span>
      <span class="cat-label">{cat.label}</span>
      {#if cat.count !== undefined}
        <span class="cat-count">{cat.count}</span>
      {/if}
    </button>
  {/each}
</div>

<style>
  .category-pills {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    justify-content: center;
  }

  .category-pill {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-4);
    height: 44px;
    border-radius: var(--radius-pill);
    background: transparent;
    color: var(--text-muted);
    border: 1px solid var(--border-default);
    cursor: pointer;
    font-size: 0.85rem;
    font-weight: 500;
    transition: border-color 200ms var(--ease-plug),
                color 200ms var(--ease-plug),
                background 200ms var(--ease-plug),
                box-shadow 200ms var(--ease-plug),
                transform 300ms cubic-bezier(0.34, 1.56, 0.64, 1);
  }

  .category-pill:hover {
    border-color: var(--border-active);
    color: var(--text-secondary);
    background: var(--pill-hover-bg);
  }

  .category-pill:focus-visible {
    outline: 2px solid var(--glow-primary);
    outline-offset: 2px;
  }

  .category-pill:active:not(.active) {
    transform: scale(0.96);
  }

  .category-pill.active {
    background: var(--glow-soft);
    border-color: var(--orange-500);
    color: var(--orange-300);
    box-shadow: 0 0 12px var(--glow-medium);
  }

  @media (prefers-reduced-motion: reduce) {
    .category-pill {
      transition: border-color 100ms var(--ease-plug),
                  color 100ms var(--ease-plug),
                  background 100ms var(--ease-plug),
                  box-shadow 100ms var(--ease-plug);
    }
    .category-pill:active:not(.active) {
      transform: none;
    }
  }

  .cat-icon {
    font-size: 1rem;
  }

  .cat-count {
    font-size: 0.7rem;
    font-weight: 700;
    padding: 1px 6px;
    border-radius: var(--radius-pill);
    background: var(--void-raised);
    color: var(--text-dim);
    line-height: 1.4;
  }

  .category-pill.active .cat-count {
    background: var(--glow-medium);
    color: var(--orange-300);
  }
</style>
