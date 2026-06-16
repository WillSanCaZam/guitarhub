<script lang="ts">
  interface Props {
    label: string;
    onclick?: () => void;
  }

  let { label, onclick }: Props = $props();
</script>

{#if onclick}
  <button class="chip" onclick={onclick}>
    {label}
  </button>
{:else}
  <span class="chip">
    {label}
  </span>
{/if}

<style>
  .chip {
    display: inline-flex;
    align-items: center;
    padding: 4px 12px;
    border-radius: var(--radius-sm);
    background: var(--color-surface-container-high);
    color: var(--color-on-surface-variant);
    font-family: var(--font-sans);
    font-size: 0.75rem;
    font-weight: 500;
    line-height: 1.4;
    border: 1px solid var(--color-outline-variant);
    cursor: default;
    white-space: nowrap;
    transition: background var(--transition-fast),
                border-color var(--transition-fast);
  }

  button.chip {
    cursor: pointer;
  }

  button.chip:hover {
    background: var(--color-surface-container-highest);
    border-color: var(--color-outline);
    color: var(--color-on-surface);
  }

  button.chip:focus-visible {
    outline: 2px solid var(--color-primary);
    outline-offset: 2px;
  }
</style>
