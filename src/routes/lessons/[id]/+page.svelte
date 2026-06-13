<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- Lesson Detail route — lesson detail, video embed, description, comments -->
<script lang="ts">
  import { page } from '$app/stores';
  import { communityState, loadFeed, addComment } from '$lib/stores/community.svelte';
  import AuthGuard from '$lib/components/auth/AuthGuard.svelte';
  import type { Lesson } from '$lib/types/community';
  import { onMount } from 'svelte';

  let commentText = $state('');
  let submitting = $state(false);

  const lessonId = $derived($page.params.id);

  const lesson = $derived(() => {
    for (const item of communityState.feed) {
      if (item.itemType === 'lesson' && (item.content as Lesson).id === lessonId) {
        return item.content as Lesson;
      }
    }
    return null;
  });

  onMount(() => {
    if (communityState.feed.length === 0) {
      loadFeed(true);
    }
  });

  async function handleComment() {
    if (!commentText.trim() || submitting) return;
    submitting = true;
    try {
      await addComment('lesson', lessonId as string, commentText);
      commentText = '';
    } catch (e) {
      console.error('Failed to add comment:', e);
    } finally {
      submitting = false;
    }
  }
</script>

<svelte:head>
  <title>{lesson()?.title ?? 'Lesson'} — GuitarHub</title>
</svelte:head>

<AuthGuard>
  <div class="lesson-detail">
    {#if lesson()}
      {@const l = lesson()!}

      <div class="video-embed">
        {#if l.contentUrl}
          <iframe
            src={`https://www.youtube.com/embed/${new URL(l.contentUrl).searchParams.get('v') || ''}`}
            title={l.title}
            frameborder="0"
            allowfullscreen
          ></iframe>
        {:else}
          <div class="video-placeholder">No video available</div>
        {/if}
      </div>

      <div class="lesson-info">
        <h1 class="lesson-title">{l.title}</h1>
        <div class="lesson-meta">
          <span class="difficulty">{l.difficulty}</span>
          <span class="likes">♥ {l.likes}</span>
        </div>
        <p class="lesson-description">{l.description}</p>

        {#if l.tags.length > 0}
          <div class="tags">
            {#each l.tags as tag}
              <span class="tag">#{tag}</span>
            {/each}
          </div>
        {/if}
      </div>

      <div class="comments-section">
        <h2>Comments</h2>
        <form class="comment-form" onsubmit={(e) => { e.preventDefault(); handleComment(); }}>
          <textarea
            bind:value={commentText}
            placeholder="Add a comment..."
            rows="3"
          ></textarea>
          <button type="submit" disabled={!commentText.trim() || submitting}>
            {submitting ? 'Posting...' : 'Post Comment'}
          </button>
        </form>
      </div>
    {:else}
      <div class="loading">Loading lesson...</div>
    {/if}
  </div>
</AuthGuard>

<style>
  .lesson-detail {
    max-width: 800px;
    margin: 0 auto;
    padding: var(--spacing-md);
  }

  .video-embed {
    position: relative;
    padding-bottom: 56.25%;
    border-radius: var(--radius-md);
    overflow: hidden;
    background: var(--color-surface-container);
    margin-bottom: var(--spacing-md);
  }

  .video-embed iframe {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
  }

  .video-placeholder {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-on-surface-muted);
  }

  .lesson-title {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--color-on-surface);
    margin: 0 0 var(--spacing-sm) 0;
  }

  .lesson-meta {
    display: flex;
    gap: var(--spacing-md);
    font-size: 0.9rem;
    color: var(--color-on-surface-muted);
    margin-bottom: var(--spacing-md);
  }

  .difficulty {
    padding: var(--spacing-2xs) var(--spacing-sm);
    background: var(--color-primary-container);
    color: var(--color-on-primary-container);
    border-radius: var(--radius-sm);
    font-size: 0.8rem;
    text-transform: capitalize;
  }

  .lesson-description {
    font-size: 1rem;
    color: var(--color-on-surface-variant);
    line-height: 1.6;
    margin: 0 0 var(--spacing-md) 0;
  }

  .tags {
    display: flex;
    gap: var(--spacing-xs);
    flex-wrap: wrap;
    margin-bottom: var(--spacing-md);
  }

  .tag {
    font-size: 0.8rem;
    color: var(--color-secondary);
    font-family: var(--font-mono);
  }

  .comments-section {
    border-top: 1px solid var(--color-outline-variant);
    padding-top: var(--spacing-md);
  }

  .comments-section h2 {
    font-size: 1.1rem;
    font-weight: 600;
    color: var(--color-on-surface);
    margin: 0 0 var(--spacing-md) 0;
  }

  .comment-form {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .comment-form textarea {
    width: 100%;
    padding: var(--spacing-sm);
    background: var(--color-surface-container);
    border: 1px solid var(--color-outline);
    border-radius: var(--radius-md);
    color: var(--color-on-surface);
    font-family: var(--font-sans);
    font-size: 0.9rem;
    resize: vertical;
  }

  .comment-form textarea:focus {
    outline: none;
    border-color: var(--color-primary);
  }

  .comment-form button {
    align-self: flex-end;
    padding: var(--spacing-xs) var(--spacing-md);
    background: var(--color-primary);
    color: var(--color-on-primary);
    border: none;
    border-radius: var(--radius-md);
    cursor: pointer;
    font-size: 0.9rem;
  }

  .comment-form button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .loading {
    text-align: center;
    padding: var(--spacing-xl);
    color: var(--color-on-surface-muted);
  }
</style>
