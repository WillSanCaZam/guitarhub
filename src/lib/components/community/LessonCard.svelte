<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!--
  LessonCard — Lesson preview card.
  Shows thumbnail, title, difficulty chip, author, duration, like count.
-->
<script lang="ts">
  import Chip from '$lib/components/ui/Chip.svelte';
  import type { Lesson } from '$lib/types/community';

  interface Props {
    lesson: Lesson;
    onLike?: (id: string) => void;
    onClick?: (id: string) => void;
  }

  let { lesson, onLike, onClick }: Props = $props();

  function timeAgo(timestamp: number): string {
    const seconds = Math.floor((Date.now() - timestamp) / 1000);
    if (seconds < 60) return 'just now';
    const minutes = Math.floor(seconds / 60);
    if (minutes < 60) return `${minutes}m ago`;
    const hours = Math.floor(minutes / 60);
    if (hours < 24) return `${hours}h ago`;
    const days = Math.floor(hours / 24);
    return `${days}d ago`;
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<article class="lesson-card" onclick={() => onClick?.(lesson.id)} onkeydown={(e) => e.key === 'Enter' && onClick?.(lesson.id)}>
  <div class="card-thumbnail">
    {#if lesson.contentUrl}
      <img
        src={`https://img.youtube.com/vi/${new URL(lesson.contentUrl).searchParams.get('v') || ''}/mqdefault.jpg`}
        alt={lesson.title}
        class="thumbnail-img"
        loading="lazy"
      />
    {:else}
      <div class="thumbnail-placeholder">♪</div>
    {/if}
  </div>

  <div class="card-content">
    <h3 class="lesson-title">{lesson.title}</h3>
    <p class="lesson-description">{lesson.description}</p>

    <div class="card-meta">
      <Chip label={lesson.difficulty} />
      <span class="like-count">♥ {lesson.likes}</span>
      <span class="timestamp">{timeAgo(lesson.createdAt)}</span>
    </div>

    {#if lesson.tags.length > 0}
      <div class="tags">
        {#each lesson.tags.slice(0, 3) as tag}
          <span class="tag">#{tag}</span>
        {/each}
      </div>
    {/if}
  </div>

  <button
    class="like-btn"
    onclick={(e) => { e.stopPropagation(); onLike?.(lesson.id); }}
    aria-label="Like lesson"
  >
    ♥
  </button>
</article>

<style>
  .lesson-card {
    display: flex;
    gap: var(--spacing-md);
    padding: var(--spacing-md);
    background: var(--color-surface-container-low);
    border-radius: var(--radius-md);
    border: 1px solid var(--color-outline-variant);
    cursor: pointer;
    transition: border-color var(--transition-fast), box-shadow var(--transition-fast);
    position: relative;
  }

  .lesson-card:hover {
    border-color: var(--color-outline);
    box-shadow: var(--elevation-1);
  }

  .card-thumbnail {
    flex-shrink: 0;
    width: 120px;
    height: 80px;
    border-radius: var(--radius-sm);
    overflow: hidden;
    background: var(--color-surface-container);
  }

  .thumbnail-img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .thumbnail-placeholder {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.5rem;
    color: var(--color-on-surface-muted);
  }

  .card-content {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .lesson-title {
    font-size: 1rem;
    font-weight: 600;
    color: var(--color-on-surface);
    margin: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .lesson-description {
    font-size: 0.85rem;
    color: var(--color-on-surface-variant);
    margin: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .card-meta {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    font-size: 0.8rem;
    color: var(--color-on-surface-muted);
  }

  .like-count {
    color: var(--color-error);
  }

  .tags {
    display: flex;
    gap: var(--spacing-xs);
    flex-wrap: wrap;
  }

  .tag {
    font-size: 0.75rem;
    color: var(--color-secondary);
    font-family: var(--font-mono);
  }

  .like-btn {
    position: absolute;
    top: var(--spacing-sm);
    right: var(--spacing-sm);
    background: var(--color-surface-container);
    border: 1px solid var(--color-outline-variant);
    color: var(--color-on-surface-variant);
    width: 32px;
    height: 32px;
    border-radius: var(--radius-full);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.85rem;
    transition: background var(--transition-fast), color var(--transition-fast);
  }

  .like-btn:hover {
    background: var(--color-error-container);
    color: var(--color-error);
  }
</style>
