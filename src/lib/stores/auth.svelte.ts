// SPDX-License-Identifier: GPL-3.0-or-later
//
// Auth state — Svelte 5 runes implementation.
// Manages user authentication, token, and server reachability.

import { invoke } from '@tauri-apps/api/core';
import type { UserProfile } from '$lib/types/community';

export interface AuthStore {
  user: UserProfile | null;
  token: string | null;
  serverReachable: boolean;
  loading: boolean;
  error: string | null;
}

/** Reactive auth state — access directly in components. */
export const authState: AuthStore = $state({
  user: null,
  token: null,
  serverReachable: false,
  loading: false,
  error: null,
});

/**
 * Register a new user account.
 * Returns the user ID on success.
 */
export async function register(
  username: string,
  email: string,
  passwordHash: string
): Promise<string> {
  authState.loading = true;
  authState.error = null;
  try {
    const userId = await invoke<string>('register', {
      username,
      email,
      passwordHash,
    });
    authState.loading = false;
    return userId;
  } catch (e) {
    authState.loading = false;
    authState.error = String(e);
    throw e;
  }
}

/**
 * Login with email and password hash.
 * Sets user and token on success.
 */
export async function login(email: string, passwordHash: string): Promise<void> {
  authState.loading = true;
  authState.error = null;
  try {
    const token = await invoke<string | null>('login', {
      email,
      passwordHash,
    });
    authState.token = token;

    // Fetch user profile if token obtained
    if (token) {
      const userId = await invoke<string | null>('get_current_user_id', { token });
      if (userId) {
        const user = await invoke<UserProfile | null>('get_profile', { userId });
        authState.user = user;
      }
    }

    authState.loading = false;
  } catch (e) {
    authState.loading = false;
    authState.error = String(e);
    throw e;
  }
}

/** Logout — clear all auth state. */
export function logout(): void {
  authState.user = null;
  authState.token = null;
  authState.error = null;
  invoke('logout').catch(() => {
    // Best-effort — token cleared locally regardless
  });
}

/**
 * Check if the community server is reachable.
 * Sets serverReachable flag accordingly.
 */
export async function checkServerHealth(): Promise<boolean> {
  try {
    const healthy = await invoke<boolean>('health_check');
    authState.serverReachable = healthy;
    return healthy;
  } catch {
    authState.serverReachable = false;
    return false;
  }
}
