<script lang="ts">
  interface Props {
    title: string;
    icon?: string;
    loading?: boolean;
    empty?: boolean;
    emptyMessage?: string;
    emptyIcon?: string;
    children?: import('svelte').Snippet;
  }

  let {
    title,
    icon = '',
    loading = false,
    empty = false,
    emptyMessage = 'No data yet',
    emptyIcon = '📭',
    children
  }: Props = $props();
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div
  class="dashboard-cell"
  role="region"
  aria-label={title}
  tabindex="0"
>
  <!-- TODO: migrate emoji icons to inline SVG for consistency — tracked in design backlog -->
  <div class="cell-header">
    {#if icon}
      <span class="cell-icon" aria-hidden="true">{icon}</span>
    {/if}
    <h3 class="cell-title">{title}</h3>
  </div>
  <div class="cell-body">
    {#if loading}
      <div class="loading-wrap" aria-busy="true">
        <span class="spinner"></span>
        <span class="loading-text">Loading...</span>
      </div>
    {:else if empty}
      <div class="empty-wrap">
        <span class="empty-icon" aria-hidden="true">{emptyIcon}</span>
        <p class="empty-text">{emptyMessage}</p>
      </div>
    {:else}
      {@render children?.()}
    {/if}
  </div>
</div>

<style>
  .dashboard-cell {
    background: var(--color-surface-container);
    border-radius: var(--radius-lg);
    padding: var(--spacing-md);
    border: 1px solid var(--color-outline-variant);
    box-shadow: var(--shadow-1);
    display: flex;
    flex-direction: column;
    min-height: 120px;
    transition: box-shadow var(--transition-base);
    outline: none;
    cursor: default;
  }

  .dashboard-cell:hover {
    box-shadow: var(--shadow-2), var(--shadow-amber-glow);
  }

  .dashboard-cell:active {
    box-shadow: var(--shadow-1);
  }

  .dashboard-cell:focus-visible,
  .dashboard-cell:focus-within {
    outline: 2px solid var(--color-amber);
    outline-offset: 2px;
  }

  .cell-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    margin-bottom: var(--spacing-sm);
  }

  .cell-icon {
    font-size: 1.1rem;
    line-height: 1;
  }

  .cell-title {
    margin: 0;
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--color-on-surface-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    font-family: var(--font-mono);
  }

  .cell-body {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .loading-wrap {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-sm);
    flex: 1;
    color: var(--color-on-surface-muted);
    font-size: 0.9rem;
  }

  .empty-wrap {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    color: var(--color-on-surface-muted);
    text-align: center;
    gap: var(--spacing-xs);
  }

  .empty-icon {
    font-size: 1.5rem;
    opacity: 0.5;
  }

  .empty-text {
    margin: 0;
    font-size: 0.85rem;
  }

  @media (max-width: 768px) {
    .dashboard-cell {
      min-height: 88px;
    }
  }
</style>
