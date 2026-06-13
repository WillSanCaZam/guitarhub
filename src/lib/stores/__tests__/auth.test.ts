// SPDX-License-Identifier: GPL-3.0-or-later
//
// Phase 4: Auth Layer — tests for auth store, LoginForm, AuthGuard.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { authState, login, logout, checkServerHealth } from '$lib/stores/auth.svelte';
import LoginForm from '$lib/components/auth/LoginForm.svelte';
import AuthGuard from '$lib/components/auth/AuthGuard.svelte';
import SnippetTestWrapper from '$lib/components/__tests__/SnippetTestWrapper.svelte';

// ── Auth Store Tests ────────────────────────────────────────────────────

describe('auth store', () => {
  beforeEach(() => {
    // Reset store state
    authState.user = null;
    authState.token = null;
    authState.serverReachable = false;
    authState.loading = false;
    authState.error = null;
    vi.clearAllMocks();
  });

  it('initializes with default state', () => {
    expect(authState.user).toBeNull();
    expect(authState.token).toBeNull();
    expect(authState.serverReachable).toBe(false);
    expect(authState.loading).toBe(false);
    expect(authState.error).toBeNull();
  });

  it('login sets user and token on success', async () => {
    const mockInvoke = await import('@tauri-apps/api/core');
    vi.mocked(mockInvoke.invoke)
      .mockResolvedValueOnce('jwt-token-123')   // login returns token
      .mockResolvedValueOnce('user-123')         // get_current_user_id
      .mockResolvedValueOnce({                    // get_profile returns user
        id: 'user-123',
        username: 'tester',
        displayName: 'Tester',
        gearList: [],
        streakDays: 0,
        joinedAt: Date.now(),
      });

    await login('test@example.com', 'password123');

    expect(authState.token).toBe('jwt-token-123');
    expect(authState.user).not.toBeNull();
    expect(authState.user?.id).toBe('user-123');
    expect(authState.loading).toBe(false);
  });

  it('login sets error on failure', async () => {
    const mockInvoke = await import('@tauri-apps/api/core');
    vi.mocked(mockInvoke.invoke).mockRejectedValue(new Error('Invalid credentials'));

    await expect(login('bad@example.com', 'wrong')).rejects.toThrow('Invalid credentials');
    expect(authState.error).toContain('Invalid credentials');
    expect(authState.loading).toBe(false);
  });

  it('logout clears user and token', () => {
    authState.user = { id: '1', username: 'test' } as any;
    authState.token = 'some-token';

    logout();

    expect(authState.user).toBeNull();
    expect(authState.token).toBeNull();
  });

  it('checkServerHealth sets serverReachable', async () => {
    const mockInvoke = await import('@tauri-apps/api/core');
    vi.mocked(mockInvoke.invoke).mockResolvedValue(true);

    await checkServerHealth();

    expect(authState.serverReachable).toBe(true);
  });

  it('checkServerHealth handles failure gracefully', async () => {
    const mockInvoke = await import('@tauri-apps/api/core');
    vi.mocked(mockInvoke.invoke).mockRejectedValue(new Error('Network error'));

    await checkServerHealth();

    expect(authState.serverReachable).toBe(false);
  });
});

// ── LoginForm Tests ─────────────────────────────────────────────────────

describe('LoginForm', () => {
  beforeEach(() => {
    authState.user = null;
    authState.token = null;
    authState.error = null;
    authState.loading = false;
    vi.clearAllMocks();
  });

  it('renders email and password inputs', () => {
    render(LoginForm);
    expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/password/i)).toBeInTheDocument();
  });

  it('renders login button', () => {
    render(LoginForm);
    expect(screen.getByRole('button', { name: /sign in/i })).toBeInTheDocument();
  });

  it('renders OAuth buttons', () => {
    render(LoginForm);
    expect(screen.getByRole('button', { name: /github/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /google/i })).toBeInTheDocument();
  });

  it('displays error message when auth error exists', async () => {
    authState.error = 'Invalid credentials';
    render(LoginForm);
    expect(screen.getByText('Invalid credentials')).toBeInTheDocument();
  });

  it('disables button while loading', async () => {
    authState.loading = true;
    render(LoginForm);
    expect(screen.getByRole('button', { name: /signing in/i })).toBeDisabled();
  });
});

// ── AuthGuard Tests ─────────────────────────────────────────────────────

describe('AuthGuard', () => {
  beforeEach(() => {
    authState.user = null;
    authState.token = null;
    authState.serverReachable = false;
    vi.clearAllMocks();
  });

  it('shows login prompt when not authenticated', () => {
    authState.serverReachable = true;
    render(AuthGuard);
    expect(screen.getByText(/sign in/i)).toBeInTheDocument();
  });

  it('shows children when authenticated', () => {
    authState.user = { id: '1', username: 'test' } as any;
    authState.token = 'valid-token';
    authState.serverReachable = true;
    render(SnippetTestWrapper, { props: { component: 'authguard', text: 'Protected content' } });
    expect(screen.getByText('Protected content')).toBeInTheDocument();
  });

  it('shows offline notice when server unreachable', () => {
    authState.serverReachable = false;
    render(AuthGuard);
    expect(screen.getByText(/offline mode/i)).toBeInTheDocument();
  });
});
