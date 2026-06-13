<script lang="ts">
  interface Props {
    label?: string;
    placeholder?: string;
    value: string;
    type?: string;
    disabled?: boolean;
    oninput?: (value: string) => void;
  }

  let {
    label,
    placeholder = '',
    value = $bindable(''),
    type = 'text',
    disabled = false,
    oninput,
  }: Props = $props();

  const inputId = `input-${Math.random().toString(36).slice(2, 9)}`;

  function handleInput(e: Event) {
    const target = e.target as HTMLInputElement;
    value = target.value;
    oninput?.(target.value);
  }
</script>

<div class="input-wrapper">
  {#if label}
    <label class="input-label" for={inputId}>{label}</label>
  {/if}
  <input
    id={inputId}
    class="input"
    {type}
    {placeholder}
    {disabled}
    {value}
    oninput={handleInput}
  />
</div>

<style>
  .input-wrapper {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs, 4px);
  }

  .input-label {
    font-family: var(--font-mono, monospace);
    font-size: 0.6875rem;
    font-weight: 500;
    letter-spacing: 0.03em;
    color: var(--color-on-surface-variant, #a0a0b0);
    text-transform: uppercase;
  }

  .input {
    padding: 10px 12px;
    background: var(--color-surface-container-lowest, #0e0e14);
    border: 1px solid var(--color-outline-variant, #2a2a38);
    border-bottom: 2px solid var(--color-outline-variant, #2a2a38);
    border-radius: var(--radius-sm, 4px);
    color: var(--color-on-surface, #e8e8f0);
    font-family: var(--font-mono, monospace);
    font-size: 0.875rem;
    outline: none;
    transition: border-color var(--transition-fast, 100ms ease);
  }

  .input::placeholder {
    color: var(--color-on-surface-muted, #666680);
  }

  .input:focus {
    border-bottom-color: var(--color-primary, #d4a017);
  }

  .input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
