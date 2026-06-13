<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- Saved Riffs route — saved riffs list, tablature preview, BPM/tuning metadata -->
<script lang="ts">
  import { communityState, loadFeed } from '$lib/stores/community.svelte';
  import AuthGuard from '$lib/components/auth/AuthGuard.svelte';
  import type { Riff } from '$lib/types/community';
  import { onMount } from 'svelte';

  onMount(() => {
    loadFeed(true);
  });

  const riffs = $derived(() => {
    return communityState.feed
      .filter(item => item.itemType === 'riff')
      .map(item => item.content as Riff);
  });
</script>

<svelte:head>
  <title>Saved Riffs — GuitarHub</title>
</svelte:head>

<AuthGuard>
  <div class="saved-riffs-page">
    <header class="page-header">
      <h1>Saved Riffs</h1>
    </header>

    {#if riffs().length > 0}
      <div class="riffs-list">
        {#each riffs() as riff (riff.id)}
          <div class="riff-card">
            <div class="riff-header">
              <h3 class="riff-title">{riff.title}</h3>
              <div class="riff-meta">
                <span class="bpm">{riff.bpm} BPM</span>
                <span class="tuning">{riff.tuning}</span>
              </div>
            </div>
            <pre class="tablature">{riff.tablature}</pre>
            <div class="riff-footer">
              <span class="likes">♥ {riff.likes}</span>
              <div class="tags">
                {#each riff.tags.slice(0, 3) as tag}
                  <span class="tag">#{tag}</span>
                {/each}
              </div>
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <div class="empty-state">
        <p>No saved riffs yet. Browse the Explore page to find riffs.</p>
      </div>
    {/if}
  </div>
</AuthGuard>

<style>
  .saved-riffs-page {
    max-width: 800px;
    margin: 0 auto;
    padding: var(--spacing-md);
  }

  .page-header h1 {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--color-on-surface);
    margin: 0 0 var(--spacing-md) 0;
  }

  .riffs-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .riff-card {
    padding: var(--spacing-md);
    background: var(--color-surface-container-low);
    border-radius: var(--radius-md);
    border: 1px solid var(--color-outline-variant);
  }

  .riff-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: var(--spacing-sm);
  }

  .riff-title {
    font-size: 1rem;
    font-weight: 600;
    color: var(--color-on-surface);
    margin: 0;
  }

  .riff-meta {
    display: flex;
    gap: var(--spacing-sm);
    font-size: 0.8rem;
    color: var(--color-on-surface-muted);
  }

  .bpm {
    font-family: var(--font-mono);
    color: var(--color-primary);
  }

  .tuning {
    font-family: var(--font-mono);
    color: var(--color-secondary);
  }

  .tablature {
    background: var(--color-surface-container);
    padding: var(--spacing-sm);
    border-radius: var(--radius-sm);
    font-family: var(--font-mono);
    font-size: 0.8rem;
    color: var(--color-on-surface-variant);
    overflow-x: auto;
    white-space: pre;
    margin: 0 0 var(--spacing-sm) 0;
    line-height: 1.4;
  }

  .riff-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.8rem;
    color: var(--color-on-surface-muted);
  }

  .likes {
    color: var(--color-error);
  }

  .tags {
    display: flex;
    gap: var(--spacing-xs);
  }

  .tag {
    font-size: 0.75rem;
    color: var(--color-secondary);
    font-family: var(--font-mono);
  }

  .empty-state {
    text-align: center;
    padding: var(--spacing-xl);
    color: var(--color-on-surface-muted);
  }
</style>
