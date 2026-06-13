<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- Profile route — user profile, practice history, gear list -->
<script lang="ts">
  import { profileState, loadProfile, loadStreak } from '$lib/stores/profile.svelte';
  import { authState } from '$lib/stores/auth.svelte';
  import ProfileHeader from '$lib/components/community/ProfileHeader.svelte';
  import StreakCounter from '$lib/components/community/StreakCounter.svelte';
  import AuthGuard from '$lib/components/auth/AuthGuard.svelte';
  import type { Streak } from '$lib/types/community';
  import { onMount } from 'svelte';

  onMount(() => {
    if (authState.user) {
      loadProfile(authState.user.id);
      loadStreak(authState.user.id);
    }
  });
</script>

<svelte:head>
  <title>Profile — GuitarHub</title>
</svelte:head>

<AuthGuard>
  <div class="profile-page">
    {#if profileState.profile}
      <ProfileHeader profile={profileState.profile} isOwnProfile={true} />

      {#if profileState.streak}
        <StreakCounter streak={profileState.streak} />
      {/if}

      <section class="gear-section">
        <h2>My Gear ({profileState.profile.gearList.length})</h2>
        {#if profileState.profile.gearList.length > 0}
          <div class="gear-list">
            {#each profileState.profile.gearList as sku}
              <div class="gear-item">{sku}</div>
            {/each}
          </div>
        {:else}
          <p class="empty-text">No gear added yet. Browse the catalog to add items.</p>
        {/if}
      </section>
    {:else if profileState.loading}
      <div class="loading">Loading profile...</div>
    {:else}
      <div class="loading">No profile found.</div>
    {/if}
  </div>
</AuthGuard>

<style>
  .profile-page {
    max-width: 600px;
    margin: 0 auto;
    padding: var(--spacing-md);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .gear-section {
    background: var(--color-surface-container-low);
    border-radius: var(--radius-md);
    border: 1px solid var(--color-outline-variant);
    padding: var(--spacing-md);
  }

  .gear-section h2 {
    font-size: 1.1rem;
    font-weight: 600;
    color: var(--color-on-surface);
    margin: 0 0 var(--spacing-sm) 0;
  }

  .gear-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .gear-item {
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--color-surface-container);
    border-radius: var(--radius-sm);
    font-family: var(--font-mono);
    font-size: 0.85rem;
    color: var(--color-on-surface-variant);
  }

  .empty-text {
    color: var(--color-on-surface-muted);
    font-size: 0.9rem;
    margin: 0;
  }

  .loading {
    text-align: center;
    padding: var(--spacing-xl);
    color: var(--color-on-surface-muted);
  }
</style>
