<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- Feed route — infinite scroll feed with FeedCard list -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { communityState, loadFeed, likeContent } from '$lib/stores/community.svelte';
  import FeedCard from '$lib/components/community/FeedCard.svelte';
  import AuthGuard from '$lib/components/auth/AuthGuard.svelte';

  onMount(() => {
    loadFeed(true);
  });

  function handleLike(id: string) {
    likeContent(id);
  }

  function handleLoadMore() {
    if (!communityState.loading && communityState.hasMoreFeed) {
      loadFeed();
    }
  }
</script>

<svelte:head>
  <title>Feed — GuitarHub</title>
</svelte:head>

<AuthGuard>
  <div class="feed-page">
    <header class="page-header">
      <h1>Community Feed</h1>
    </header>

    {#if communityState.error}
      <div class="error-banner">{communityState.error}</div>
    {/if}

    <div class="feed-list">
      {#each communityState.feed as item (item.id)}
        <FeedCard {item} onLike={handleLike} />
      {/each}
    </div>

    {#if communityState.loading}
      <div class="loading">Loading...</div>
    {/if}

    {#if communityState.hasMoreFeed && !communityState.loading}
      <button class="load-more" onclick={handleLoadMore}>Load more</button>
    {/if}

    {#if !communityState.loading && communityState.feed.length === 0}
      <div class="empty-state">
        <p>No feed items yet. Follow some users to see their activity here.</p>
      </div>
    {/if}
  </div>
</AuthGuard>

<style>
  .feed-page {
    max-width: 600px;
    margin: 0 auto;
    padding: var(--spacing-md);
  }

  .page-header h1 {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--color-on-surface);
    margin: 0 0 var(--spacing-md) 0;
  }

  .error-banner {
    padding: var(--spacing-sm) var(--spacing-md);
    border-radius: var(--radius-md);
    background: var(--color-error-container);
    color: var(--color-on-error-container);
    margin-bottom: var(--spacing-md);
  }

  .feed-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .loading {
    text-align: center;
    padding: var(--spacing-lg);
    color: var(--color-on-surface-muted);
  }

  .load-more {
    display: block;
    width: 100%;
    padding: var(--spacing-sm);
    margin-top: var(--spacing-md);
    background: var(--color-surface-container);
    color: var(--color-on-surface);
    border: 1px solid var(--color-outline);
    border-radius: var(--radius-md);
    cursor: pointer;
    font-size: 0.9rem;
  }

  .load-more:hover {
    background: var(--color-surface-container-high);
  }

  .empty-state {
    text-align: center;
    padding: var(--spacing-xl);
    color: var(--color-on-surface-muted);
  }
</style>
