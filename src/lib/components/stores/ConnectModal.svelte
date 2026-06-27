<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'
  import StoreIcon from './StoreIcon.svelte'
  import type { StoreDef, Connection } from '$lib/types/stores'

  let {
    store,
    onClose,
    onConnected,
  }: {
    store: StoreDef
    onClose: () => void
    onConnected: (connection: Connection) => void
  } = $props()

  let token = $state('')
  let step = $state<'guide' | 'validating' | 'success' | 'error'>('guide')
  let error = $state('')
  let username = $state('')

  function handleOpenSettings() {
    window.open(store.token_url, '_blank')
    step = 'guide'
  }

  async function handleValidate() {
    if (!token.trim()) return

    step = 'validating'
    error = ''

    try {
      // First validate the token
      const result = await invoke<string>('validate_store_token', {
        storeId: store.id,
        token: token.trim(),
      })
      username = result

      // Then create the connection
      const conn = await invoke<Connection>('connect_store', {
        storeId: store.id,
        token: token.trim(),
      })

      step = 'success'
      setTimeout(() => onConnected(conn), 1500)
    } catch (e) {
      error = String(e)
      step = 'error'
    }
  }

  function handlePaste() {
    navigator.clipboard.readText().then((text) => {
      token = text
    }).catch(() => {
      // Fallback — user can type manually
    })
  }

  function handleRetry() {
    step = 'guide'
    error = ''
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      onClose()
    }
  }

  function handleOverlayClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose()
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="modal-overlay" onclick={handleOverlayClick} role="dialog" aria-modal="true" tabindex="-1" aria-label="Connect {store.name}">
  <div class="modal-content">
    <button class="close-btn" onclick={onClose} aria-label="Close modal">
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="18" y1="6" x2="6" y2="18" />
        <line x1="6" y1="6" x2="18" y2="18" />
      </svg>
    </button>

    <div class="modal-header">
      <StoreIcon storeId={store.id} size={40} />
      <h2 class="modal-title">Connect {store.name}</h2>
    </div>

    {#if step === 'guide' || step === 'validating'}
      <div class="modal-body">
        <p class="step-label">Step 1: Generate a Personal Access Token</p>
        <ol class="guide-list">
          <li>
            <button class="link-btn" onclick={handleOpenSettings}>
              Open {store.name} Settings
            </button>
            — this opens {store.token_url} in your browser
          </li>
          <li>Log in to your {store.name} account if needed</li>
          <li>Navigate to the <strong>API / Personal Access Tokens</strong> section</li>
          <li>Click <strong>Generate new token</strong></li>
          <li>Copy the generated token (it starts with <code>pat_</code>)</li>
        </ol>

        <p class="step-label">Step 2: Paste your token below</p>

        <div class="input-row">
          <input
            type="password"
            bind:value={token}
            placeholder="Paste your PAT here..."
            class="token-input"
            disabled={step === 'validating'}
            autocomplete="off"
          />
          <button
            class="paste-btn"
            onclick={handlePaste}
            disabled={step === 'validating'}
            aria-label="Paste from clipboard"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="8" y="2" width="8" height="4" rx="1" />
              <path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2" />
            </svg>
          </button>
        </div>

        <button
          class="btn btn-validate"
          onclick={handleValidate}
          disabled={step === 'validating' || !token.trim()}
        >
          {step === 'validating' ? 'Validating...' : 'Connect'}
        </button>
      </div>
    {:else if step === 'success'}
      <div class="modal-body success-state">
        <div class="success-icon">✓</div>
        <p class="success-msg">Connected as @{username}</p>
        <p class="success-sub">Your listings will appear in the catalog shortly.</p>
      </div>
    {:else if step === 'error'}
      <div class="modal-body error-state">
        <div class="error-icon">✕</div>
        <p class="error-msg">{error}</p>
        <button class="btn btn-retry" onclick={handleRetry}>
          Try Again
        </button>
      </div>
    {/if}
  </div>
</div>

<style>
  .modal-overlay {
    position: fixed;
    inset: 0;
    background: var(--surface-overlay);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: var(--z-modal);
    padding: var(--space-5);
  }

  .modal-content {
    background: var(--void-mid);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
    max-width: 520px;
    width: 100%;
    position: relative;
    box-shadow: var(--shadow-modal);
    max-height: 90vh;
    overflow-y: auto;
  }

  .close-btn {
    position: absolute;
    top: var(--space-4);
    right: var(--space-4);
    background: none;
    border: none;
    color: var(--text-dim);
    cursor: pointer;
    padding: var(--space-1);
    border-radius: var(--radius-xs);
    transition: color var(--transition-fast), background var(--transition-fast);
  }

  .close-btn:hover {
    color: var(--text-bright);
    background: var(--void-hover);
  }

  .modal-header {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-6) var(--space-6) 0;
  }

  .modal-title {
    margin: 0;
    font-family: var(--font-body);
    font-size: 1.25rem;
    font-weight: 700;
    color: var(--text-bright);
  }

  .modal-body {
    padding: var(--space-6);
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .step-label {
    margin: 0;
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--text-warm);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .guide-list {
    margin: 0;
    padding-left: var(--space-5);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    font-size: 0.9rem;
    color: var(--text-warm);
    line-height: 1.5;
  }

  .guide-list li {
    margin: 0;
  }

  .guide-list code {
    font-family: var(--font-mono);
    font-size: 0.8rem;
    background: var(--void-deep);
    padding: 1px var(--space-1);
    border-radius: var(--radius-xs);
    color: var(--glow-primary);
  }

  .link-btn {
    display: inline;
    background: none;
    border: none;
    color: var(--glow-primary);
    font-size: 0.9rem;
    font-weight: 600;
    cursor: pointer;
    padding: 0;
    text-decoration: underline;
    text-underline-offset: 2px;
  }

  .link-btn:hover {
    color: var(--glow-warm);
  }

  .input-row {
    display: flex;
    gap: var(--space-2);
  }

  .token-input {
    flex: 1;
    padding: var(--space-3);
    border: 1px solid var(--text-muted);
    border-radius: var(--radius-sm);
    background: var(--void-deep);
    color: var(--text-bright);
    font-family: var(--font-mono);
    font-size: 0.85rem;
  }

  .token-input:focus {
    outline: none;
    border-color: var(--glow-primary);
    box-shadow: 0 0 0 2px var(--glow-soft);
  }

  .token-input::placeholder {
    color: var(--text-dim);
  }

  .paste-btn {
    padding: var(--space-2) var(--space-3);
    background: var(--void-raised);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    color: var(--text-warm);
    cursor: pointer;
    display: flex;
    align-items: center;
    transition: background var(--transition-fast), color var(--transition-fast);
  }

  .paste-btn:hover:not(:disabled) {
    background: var(--void-hover);
    color: var(--text-bright);
  }

  .btn {
    padding: var(--space-3) var(--space-6);
    border: none;
    border-radius: var(--radius-sm);
    font-size: 0.9rem;
    font-weight: 600;
    cursor: pointer;
    transition: background var(--transition-fast), opacity var(--transition-fast);
    align-self: flex-start;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-validate {
    background: var(--glow-primary);
    color: var(--void-deep);
  }

  .btn-validate:hover:not(:disabled) {
    background: var(--glow-warm);
  }

  .btn-retry {
    background: var(--void-raised);
    color: var(--text-bright);
    border: 1px solid var(--border-active);
  }

  .btn-retry:hover {
    background: var(--void-hover);
  }

  .success-state,
  .error-state {
    align-items: center;
    text-align: center;
  }

  .success-icon {
    width: 48px;
    height: 48px;
    border-radius: var(--radius-pill);
    background: var(--glow-success);
    color: var(--success);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.5rem;
    font-weight: 700;
  }

  .error-icon {
    width: 48px;
    height: 48px;
    border-radius: var(--radius-pill);
    background: var(--glow-danger);
    color: var(--danger);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.5rem;
    font-weight: 700;
  }

  .success-msg {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 600;
    color: var(--success);
  }

  .success-sub {
    margin: 0;
    font-size: 0.85rem;
    color: var(--text-dim);
  }

  .error-msg {
    margin: 0;
    font-size: 0.9rem;
    color: var(--danger);
  }
</style>
