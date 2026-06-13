// SPDX-License-Identifier: GPL-3.0-or-later
//
// Profile state — Svelte 5 runes implementation.
// Manages current user profile, streak data, and gear list.

import { invoke } from '@tauri-apps/api/core';
import type { UserProfile, Streak } from '$lib/types/community';

export interface ProfileStore {
  profile: UserProfile | null;
  streak: Streak | null;
  loading: boolean;
  error: string | null;
}

/** Reactive profile state — access directly in components. */
export const profileState: ProfileStore = $state({
  profile: null,
  streak: null,
  loading: false,
  error: null,
});

/** Load a user profile by ID. */
export async function loadProfile(userId: string): Promise<void> {
  profileState.loading = true;
  profileState.error = null;
  try {
    const profile = await invoke<UserProfile | null>('get_profile', { userId });
    profileState.profile = profile;
    profileState.loading = false;
  } catch (e) {
    profileState.loading = false;
    profileState.error = String(e);
  }
}

/** Update the current user's profile. */
export async function updateProfile(
  userId: string,
  updates: { displayName?: string; bio?: string; avatarUrl?: string }
): Promise<void> {
  profileState.loading = true;
  profileState.error = null;
  try {
    await invoke('update_profile', {
      userId,
      displayName: updates.displayName ?? null,
      bio: updates.bio ?? null,
      avatarUrl: updates.avatarUrl ?? null,
    });
    // Refresh profile
    await loadProfile(userId);
  } catch (e) {
    profileState.loading = false;
    profileState.error = String(e);
  }
}

/** Load streak data for a user. */
export async function loadStreak(userId: string): Promise<void> {
  profileState.loading = true;
  profileState.error = null;
  try {
    const streak = await invoke<Streak | null>('get_streak', { userId });
    profileState.streak = streak;
    profileState.loading = false;
  } catch (e) {
    profileState.loading = false;
    profileState.error = String(e);
  }
}

/** Record a practice session (update streak). */
export async function updateStreak(userId: string): Promise<void> {
  try {
    await invoke('update_streak', { userId });
    await loadStreak(userId);
  } catch (e) {
    profileState.error = String(e);
  }
}

/** Add a gear item to user's gear list. */
export async function addGear(userId: string, gearSku: string): Promise<void> {
  try {
    await invoke('add_gear_to_list', { userId, gearSku });
    await loadProfile(userId);
  } catch (e) {
    profileState.error = String(e);
  }
}
