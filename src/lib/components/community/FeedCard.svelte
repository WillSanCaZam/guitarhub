<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!--
  FeedCard — Community feed item display.
  Shows author, content preview, like/comment counts, timestamp.
-->
<script lang="ts">
  import Avatar from '$lib/components/ui/Avatar.svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import type { FeedItem, Lesson, Riff } from '$lib/types/community';

  interface Props {
    item: FeedItem;
    onLike?: (id: string) => void;
    onComment?: (id: string) => void;
  }

  let { item, onLike, onComment }: Props = $props();

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

  function getContentTitle(): string {
    const content = item.content;
    if ('title' in content) return (content as Lesson | Riff).title;
    if ('content' in content) return (content as any).content.slice(0, 60);
    return 'Untitled';
  }

  function getLikeCount(): number {
    const content = item.content;
    if ('likes' in content) return (content as Lesson | Riff).likes;
    return 0;
  }
</script>

<div class="feed-card">
  <div class="card-header">
    <Avatar name={item.authorName} src={item.authorAvatar} size="sm" />
    <div class="author-info">
      <span class="author-name">{item.authorName}</span>
      <span class="timestamp">{timeAgo(item.createdAt)}</span>
    </div>
    <Badge variant={item.itemType === 'lesson' ? 'primary' : 'secondary'} label={item.itemType} />
  </div>

  <div class="card-body">
    <h3 class="content-title">{getContentTitle()}</h3>
  </div>

  <div class="card-footer">
    <button class="action-btn" onclick={() => onLike?.(item.id)} aria-label="Like">
      ♥ {getLikeCount()}
    </button>
    <button class="action-btn" onclick={() => onComment?.(item.id)} aria-label="Comment">
      💬 Comment
    </button>
  </div>
</div>

<style>
  .feed-card {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
    padding: var(--spacing-md);
    background: var(--color-surface-container-low);
    border-radius: var(--radius-md);
    border: 1px solid var(--color-outline-variant);
    transition: border-color var(--transition-fast);
  }

  .feed-card:hover {
    border-color: var(--color-outline);
  }

  .card-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .author-info {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
  }

  .author-name {
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--color-on-surface);
  }

  .timestamp {
    font-size: 0.75rem;
    color: var(--color-on-surface-muted);
  }

  .card-body {
    padding: var(--spacing-xs) 0;
  }

  .content-title {
    font-size: 1rem;
    font-weight: 600;
    color: var(--color-on-surface);
    margin: 0;
  }

  .card-footer {
    display: flex;
    gap: var(--spacing-md);
    padding-top: var(--spacing-xs);
    border-top: 1px solid var(--color-outline-variant);
  }

  .action-btn {
    background: none;
    border: none;
    color: var(--color-on-surface-variant);
    font-size: 0.8rem;
    cursor: pointer;
    padding: var(--spacing-xs) var(--spacing-sm);
    border-radius: var(--radius-sm);
    transition: background var(--transition-fast);
  }

  .action-btn:hover {
    background: var(--color-surface-container-high);
    color: var(--color-on-surface);
  }
</style>
