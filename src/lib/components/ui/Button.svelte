<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    variant?: 'primary' | 'secondary' | 'ghost';
    disabled?: boolean;
    children: Snippet;
    onclick?: () => void;
  }

  let { variant = 'primary', disabled = false, children, onclick }: Props = $props();
</script>

<button
  class="btn btn--{variant}"
  {disabled}
  {onclick}
>
  {@render children()}
</button>

<style>
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 10px 20px;
    border: none;
    border-radius: var(--radius-md, 8px);
    font-family: var(--font-sans, system-ui, sans-serif);
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: background var(--transition-base, 200ms ease),
                opacity var(--transition-base, 200ms ease);
    min-height: 40px;
  }

  .btn:focus-visible {
    outline: 2px solid var(--color-primary, #d4a017);
    outline-offset: 2px;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn--primary {
    background: var(--color-primary, #d4a017);
    color: var(--color-on-primary, #1a1a2e);
  }

  .btn--primary:hover:not(:disabled) {
    background: var(--color-primary-hover, #c49316);
  }

  .btn--primary:active:not(:disabled) {
    background: var(--color-primary-active, #b38614);
  }

  .btn--secondary {
    background: var(--color-secondary-container, #1e3a5f);
    color: var(--color-on-secondary-container, #c8ddf5);
  }

  .btn--secondary:hover:not(:disabled) {
    background: var(--color-secondary, #4a90d9);
    color: var(--color-on-secondary, #ffffff);
  }

  .btn--ghost {
    background: transparent;
    color: var(--color-on-surface-variant, #a0a0b0);
  }

  .btn--ghost:hover:not(:disabled) {
    background: var(--color-surface-container-high, #242430);
    color: var(--color-on-surface, #e8e8f0);
  }
</style>
