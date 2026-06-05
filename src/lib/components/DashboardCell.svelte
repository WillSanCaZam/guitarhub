<script lang="ts">
  interface Props {
    title: string;
    icon?: string;
    loading?: boolean;
    empty?: boolean;
    children?: import('svelte').Snippet;
  }

  let { title, icon = '', loading = false, empty = false, children }: Props = $props();
</script>

<div class="dashboard-cell" role="region" aria-label={title} tabindex="0">
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
        <span class="empty-icon" aria-hidden="true">📭</span>
        <p class="empty-text">No data yet</p>
      </div>
    {:else}
      {@render children?.()}
    {/if}
  </div>
</div>

<style>
  .dashboard-cell {
    background: rgba(255, 255, 255, 0.7);
    backdrop-filter: blur(10px);
    -webkit-backdrop-filter: blur(10px);
    border-radius: 12px;
    padding: 16px;
    border: 1px solid rgba(255, 255, 255, 0.3);
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.05);
    display: flex;
    flex-direction: column;
    min-height: 120px;
    transition: box-shadow 0.2s ease, transform 0.15s ease;
    outline: none;
  }

  .dashboard-cell:focus-visible {
    box-shadow: 0 0 0 3px rgba(26, 26, 46, 0.25);
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
  }

  .empty-icon {
    font-size: 1.5rem;
    margin-bottom: 6px;
    opacity: 0.6;
  }

  .empty-text {
    margin: 0;
    font-size: 0.9rem;
  }

  @media (prefers-color-scheme: dark) {
    .dashboard-cell {
      background: rgba(30, 30, 40, 0.6);
      border-color: rgba(255, 255, 255, 0.08);
      box-shadow: 0 4px 6px rgba(0, 0, 0, 0.25);
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
</style>
