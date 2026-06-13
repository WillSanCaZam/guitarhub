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
    border-radius: var(--radius-sm, 4px);
    background: var(--color-surface-container-high, #242430);
    color: var(--color-on-surface-variant, #a0a0b0);
    font-family: var(--font-sans, system-ui, sans-serif);
    font-size: 0.75rem;
    font-weight: 500;
    line-height: 1.4;
    border: 1px solid var(--color-outline-variant, #2a2a38);
    cursor: default;
    white-space: nowrap;
    transition: background var(--transition-fast, 100ms ease),
                border-color var(--transition-fast, 100ms ease);
  }

  button.chip {
    cursor: pointer;
  }

  button.chip:hover {
    background: var(--color-surface-container-highest, #2c2c3a);
    border-color: var(--color-outline, #3a3a4a);
    color: var(--color-on-surface, #e8e8f0);
  }

  button.chip:focus-visible {
    outline: 2px solid var(--color-primary, #d4a017);
    outline-offset: 2px;
  }
</style>
