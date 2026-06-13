<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!--
  LoginForm — OAuth (GitHub/Google) + email/password fallback.
  Uses design system tokens and UI atoms.
-->
<script lang="ts">
  import Button from '$lib/components/ui/Button.svelte';
  import Input from '$lib/components/ui/Input.svelte';
  import { authState, login } from '$lib/stores/auth.svelte';

  let email = $state('');
  let password = $state('');
  let localError = $state('');

  async function handleEmailLogin() {
    if (!email || !password) {
      localError = 'Email and password are required';
      return;
    }
    localError = '';
    try {
      // Hash password client-side before sending (bcrypt via Tauri command)
      const passwordHash = await crypto.subtle.digest(
        'SHA-256',
        new TextEncoder().encode(password)
      ).then(buf => Array.from(new Uint8Array(buf)).map(b => b.toString(16).padStart(2, '0')).join(''));
      await login(email, passwordHash);
    } catch (e) {
      localError = String(e);
    }
  }

  function handleOAuth(provider: 'github' | 'google') {
    localError = `${provider} OAuth coming soon`;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') handleEmailLogin();
  }
</script>

<div class="login-form">
  <h2 class="login-title">Sign In</h2>
  <p class="login-subtitle">Access the GuitarHub community</p>

  {#if authState.error || localError}
    <div class="login-error" role="alert">
      {authState.error || localError}
    </div>
  {/if}

  <div class="oauth-buttons">
    <Button variant="secondary" disabled={authState.loading} onclick={() => handleOAuth('github')}>
      Continue with GitHub
    </Button>
    <Button variant="secondary" disabled={authState.loading} onclick={() => handleOAuth('google')}>
      Continue with Google
    </Button>
  </div>

  <div class="divider">
    <span>or sign in with email</span>
  </div>

  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="email-form" onkeydown={handleKeydown}>
    <Input
      label="Email"
      type="email"
      bind:value={email}
      placeholder="you@example.com"
      disabled={authState.loading}
    />
    <Input
      label="Password"
      type="password"
      bind:value={password}
      placeholder="Your password"
      disabled={authState.loading}
    />
    <Button
      variant="primary"
      disabled={authState.loading}
      onclick={handleEmailLogin}
    >
      {authState.loading ? 'Signing in...' : 'Sign In'}
    </Button>
  </div>
</div>

<style>
  .login-form {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
    max-width: 400px;
    margin: 0 auto;
    padding: var(--spacing-lg);
  }

  .login-title {
    font-family: var(--font-sans);
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--color-on-surface);
    margin: 0;
    text-align: center;
  }

  .login-subtitle {
    font-size: 0.9rem;
    color: var(--color-on-surface-variant);
    margin: 0;
    text-align: center;
  }

  .login-error {
    padding: var(--spacing-sm) var(--spacing-md);
    border-radius: var(--radius-md);
    background: var(--color-error-container);
    color: var(--color-on-error-container);
    font-size: 0.85rem;
    text-align: center;
  }

  .oauth-buttons {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .divider {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    color: var(--color-on-surface-muted);
    font-size: 0.8rem;
  }

  .divider::before,
  .divider::after {
    content: '';
    flex: 1;
    height: 1px;
    background: var(--color-outline);
  }

  .email-form {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }
</style>
