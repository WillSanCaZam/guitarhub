<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!--
  AuthGuard — wrapper component for authenticated routes.
  Shows login prompt if unauthenticated, offline notice if server unreachable.
-->
<script lang="ts">
  import { authState } from '$lib/stores/auth.svelte';
  import Button from '$lib/components/ui/Button.svelte';

  let { children }: { children?: import('svelte').Snippet } = $props();
</script>

{#if authState.user && authState.token}
  <!-- Authenticated: render protected content -->
  {#if children}
    {@render children()}
  {/if}
{:else if !authState.serverReachable}
  <!-- Offline: show community disabled notice -->
  <div class="auth-guard offline">
    <div class="guard-icon">📡</div>
    <h2 class="guard-title">Offline Mode</h2>
    <p class="guard-message">
      Community features require a server connection.
      Connect to the internet to access the GuitarHub community.
    </p>
    <p class="guard-hint">
      You can still browse your local catalog and gear collection.
    </p>
  </div>
{:else}
  <!-- Unauthenticated: show login prompt -->
  <div class="auth-guard login-prompt">
    <div class="guard-icon">🔐</div>
    <h2 class="guard-title">Sign In to Access</h2>
    <p class="guard-message">
      Join the GuitarHub community to share lessons, riffs, and track your practice streaks.
    </p>
    <Button variant="primary" onclick={() => window.location.href = '/settings'}>
      Go to Settings
    </Button>
  </div>
{/if}

<style>
  .auth-guard {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-md);
    min-height: 300px;
    padding: var(--spacing-xl);
    text-align: center;
  }

  .guard-icon {
    font-size: 2.5rem;
    margin-bottom: var(--spacing-sm);
  }

  .guard-title {
    font-family: var(--font-sans);
    font-size: 1.25rem;
    font-weight: 700;
    color: var(--color-on-surface);
    margin: 0;
  }

  .guard-message {
    font-size: 0.95rem;
    color: var(--color-on-surface-variant);
    max-width: 400px;
    margin: 0;
    line-height: 1.5;
  }

  .guard-hint {
    font-size: 0.85rem;
    color: var(--color-on-surface-muted);
    margin: 0;
  }

  .offline .guard-title {
    color: var(--color-warning);
  }

  .login-prompt .guard-title {
    color: var(--color-secondary);
  }
</style>
