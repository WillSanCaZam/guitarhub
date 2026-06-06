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
  class="dashboard-cell glassmorphism"
  role="region"
  aria-label={title}
  tabindex="0"
>
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
    background: rgba(255, 255, 255, 0.08);
    border-radius: 12px;
    padding: 16px;
    border: 1px solid rgba(255, 255, 255, 0.25);
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.05);
    display: flex;
    flex-direction: column;
    min-height: 120px;
    transition: transform 0.2s ease, box-shadow 0.2s ease;
    outline: none;
    cursor: default;
  }

  @supports (backdrop-filter: blur(12px)) {
    .dashboard-cell {
      background: rgba(255, 255, 255, 0.55);
      backdrop-filter: blur(12px);
      -webkit-backdrop-filter: blur(12px);
    }
  }

  .dashboard-cell:hover {
    transform: translateY(-2px);
    box-shadow: 0 8px 16px rgba(0, 0, 0, 0.1);
  }

  .dashboard-cell:active {
    transform: translateY(0);
  }

  .dashboard-cell:focus-visible,
  .dashboard-cell:focus-within {
    outline: 2px solid #4a90d9;
    outline-offset: 2px;
  }

  .cell-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 12px;
  }

  .cell-icon {
    font-size: 1.1rem;
    line-height: 1;
  }

  .cell-title {
    margin: 0;
    font-size: 0.9rem;
    font-weight: 600;
    color: #1a1a2e;
    text-transform: uppercase;
    letter-spacing: 0.05em;
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
    gap: 10px;
    flex: 1;
    color: #666;
    font-size: 0.95rem;
  }

  .spinner {
    width: 18px;
    height: 18px;
    border: 2px solid #ddd;
    border-top-color: #1a1a2e;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .empty-wrap {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    color: #999;
    text-align: center;
    gap: 6px;
  }

  .empty-icon {
    font-size: 1.5rem;
    opacity: 0.6;
  }

  .empty-text {
    margin: 0;
    font-size: 0.9rem;
  }

  @media (prefers-color-scheme: dark) {
    .dashboard-cell {
      background: rgba(20, 20, 30, 0.55);
      border-color: rgba(255, 255, 255, 0.1);
      box-shadow: 0 4px 6px rgba(0, 0, 0, 0.25);
    }

    .dashboard-cell:hover {
      box-shadow: 0 8px 16px rgba(0, 0, 0, 0.35);
    }

    .cell-title {
      color: #e8e8f0;
    }

    .loading-wrap {
      color: #aaa;
    }

    .spinner {
      border-color: #444;
      border-top-color: #e8e8f0;
    }

    .empty-wrap {
      color: #888;
    }
  }

  @media (max-width: 768px) {
    .dashboard-cell {
      min-height: 88px;
    }
  }
</style>
