<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!--
  ProfileHeader — User profile header section.
  Shows avatar, display name, bio, streak counter, gear count, follow button.
-->
<script lang="ts">
  import Avatar from '$lib/components/ui/Avatar.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import type { UserProfile } from '$lib/types/community';

  interface Props {
    profile: UserProfile;
    isOwnProfile?: boolean;
    onFollow?: (userId: string) => void;
  }

  let { profile, isOwnProfile = false, onFollow }: Props = $props();
</script>

<div class="profile-header">
  <div class="profile-main">
    <Avatar name={profile.displayName} src={profile.avatarUrl} size="lg" />
    <div class="profile-info">
      <h2 class="display-name">{profile.displayName}</h2>
      <span class="username">@{profile.username}</span>
      {#if profile.bio}
        <p class="bio">{profile.bio}</p>
      {/if}
    </div>
  </div>

  <div class="profile-stats">
    <div class="stat">
      <span class="stat-value">{profile.streakDays}</span>
      <span class="stat-label">Day Streak</span>
    </div>
    <div class="stat">
      <span class="stat-value">{profile.gearList.length}</span>
      <span class="stat-label">Gear Items</span>
    </div>
  </div>

  {#if !isOwnProfile}
    <div class="profile-actions">
      <Button variant="primary" onclick={() => onFollow?.(profile.id)}>
        Follow
      </Button>
    </div>
  {/if}
</div>

<style>
  .profile-header {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
    padding: var(--spacing-lg);
    background: var(--color-surface-container-low);
    border-radius: var(--radius-lg);
    border: 1px solid var(--color-outline-variant);
  }

  .profile-main {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
  }

  .profile-info {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-2xs);
    flex: 1;
    min-width: 0;
  }

  .display-name {
    font-size: 1.25rem;
    font-weight: 700;
    color: var(--color-on-surface);
    margin: 0;
  }

  .username {
    font-size: 0.85rem;
    color: var(--color-on-surface-muted);
    font-family: var(--font-mono);
  }

  .bio {
    font-size: 0.9rem;
    color: var(--color-on-surface-variant);
    margin: var(--spacing-xs) 0 0 0;
    line-height: 1.4;
  }

  .profile-stats {
    display: flex;
    gap: var(--spacing-lg);
    padding: var(--spacing-sm) 0;
    border-top: 1px solid var(--color-outline-variant);
    border-bottom: 1px solid var(--color-outline-variant);
  }

  .stat {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-2xs);
  }

  .stat-value {
    font-size: 1.25rem;
    font-weight: 700;
    color: var(--color-primary);
  }

  .stat-label {
    font-size: 0.75rem;
    color: var(--color-on-surface-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .profile-actions {
    display: flex;
    justify-content: center;
  }
</style>
