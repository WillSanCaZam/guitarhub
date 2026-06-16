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
    gap: var(--spacing-xs);
  }

  .input-label {
    font-family: var(--font-mono);
    font-size: 0.6875rem;
    font-weight: 500;
    letter-spacing: 0.03em;
    color: var(--color-on-surface-variant);
    text-transform: uppercase;
  }

  .input {
    padding: 10px 12px;
    background: var(--color-surface-container-lowest);
    border: 1px solid var(--color-outline-variant);
    border-bottom: 2px solid var(--color-outline-variant);
    border-radius: var(--radius-sm);
    color: var(--color-on-surface);
    font-family: var(--font-mono);
    font-size: 0.875rem;
    outline: none;
    transition: border-color var(--transition-fast);
  }

  .input::placeholder {
    color: var(--color-on-surface-muted);
  }

  .input:focus {
    border-bottom-color: var(--color-primary);
  }

  .input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
