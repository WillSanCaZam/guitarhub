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
    border-radius: var(--radius-md);
    font-family: var(--font-sans);
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: background var(--transition-base),
                opacity var(--transition-base);
    min-height: 40px;
  }

  .btn:focus-visible {
    outline: 2px solid var(--color-primary);
    outline-offset: 2px;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn--primary {
    background: var(--color-primary);
    color: var(--color-on-primary);
  }

  .btn--primary:hover:not(:disabled) {
    background: var(--color-primary-hover);
  }

  .btn--primary:active:not(:disabled) {
    background: var(--color-primary-active);
  }

  .btn--secondary {
    background: var(--color-secondary-container);
    color: var(--color-on-secondary-container);
  }

  .btn--secondary:hover:not(:disabled) {
    background: var(--color-secondary);
    color: var(--color-on-secondary);
  }

  .btn--ghost {
    background: transparent;
    color: var(--color-on-surface-variant);
  }

  .btn--ghost:hover:not(:disabled) {
    background: var(--color-surface-container-high);
    color: var(--color-on-surface);
  }
</style>
