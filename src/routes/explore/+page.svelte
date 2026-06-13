<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- Explore route — search/filter grid, category tabs, trending content -->
<script lang="ts">
  import { communityState, loadFeed } from '$lib/stores/community.svelte';
  import LessonCard from '$lib/components/community/LessonCard.svelte';
  import AuthGuard from '$lib/components/auth/AuthGuard.svelte';
  import type { Lesson } from '$lib/types/community';
  import { onMount } from 'svelte';

  let searchQuery = $state('');
  let activeCategory = $state<'all' | 'lessons' | 'riffs' | 'trending'>('all');

  onMount(() => {
    loadFeed(true);
  });

  const filteredLessons = $derived(() => {
    return communityState.feed
      .filter(item => item.itemType === 'lesson')
      .filter(item => {
        if (!searchQuery) return true;
        const lesson = item.content as Lesson;
        return lesson.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
               lesson.description.toLowerCase().includes(searchQuery.toLowerCase());
      });
  });
</script>

<svelte:head>
  <title>Explore — GuitarHub</title>
</svelte:head>

<AuthGuard>
  <div class="explore-page">
    <header class="page-header">
      <h1>Explore</h1>
      <input
        type="search"
        class="search-input"
        placeholder="Search lessons, riffs..."
        bind:value={searchQuery}
        aria-label="Search community content"
      />
    </header>

    <div class="category-tabs">
      <button
        class="tab"
        class:active={activeCategory === 'all'}
        onclick={() => activeCategory = 'all'}
      >All</button>
      <button
        class="tab"
        class:active={activeCategory === 'lessons'}
        onclick={() => activeCategory = 'lessons'}
      >Lessons</button>
      <button
        class="tab"
        class:active={activeCategory === 'riffs'}
        onclick={() => activeCategory = 'riffs'}
      >Riffs</button>
      <button
        class="tab"
        class:active={activeCategory === 'trending'}
        onclick={() => activeCategory = 'trending'}
      >Trending</button>
    </div>

    <div class="explore-grid">
      {#each filteredLessons() as item (item.id)}
        <LessonCard lesson={item.content as Lesson} />
      {/each}
    </div>

    {#if filteredLessons().length === 0 && !communityState.loading}
      <div class="empty-state">
        <p>No results found. Try a different search term.</p>
      </div>
    {/if}
  </div>
</AuthGuard>

<style>
  .explore-page {
    max-width: 800px;
    margin: 0 auto;
    padding: var(--spacing-md);
  }

  .page-header {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
    margin-bottom: var(--spacing-md);
  }

  .page-header h1 {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--color-on-surface);
    margin: 0;
  }

  .search-input {
    width: 100%;
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--color-surface-container);
    border: 1px solid var(--color-outline);
    border-radius: var(--radius-md);
    color: var(--color-on-surface);
    font-size: 0.95rem;
  }

  .search-input:focus {
    outline: none;
    border-color: var(--color-primary);
  }

  .search-input::placeholder {
    color: var(--color-on-surface-muted);
  }

  .category-tabs {
    display: flex;
    gap: var(--spacing-xs);
    margin-bottom: var(--spacing-md);
    border-bottom: 1px solid var(--color-outline-variant);
    padding-bottom: var(--spacing-sm);
  }

  .tab {
    padding: var(--spacing-xs) var(--spacing-md);
    background: none;
    border: none;
    color: var(--color-on-surface-variant);
    font-size: 0.9rem;
    cursor: pointer;
    border-radius: var(--radius-sm);
    transition: background var(--transition-fast);
  }

  .tab:hover {
    background: var(--color-surface-container);
  }

  .tab.active {
    background: var(--color-primary-container);
    color: var(--color-on-primary-container);
  }

  .explore-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: var(--spacing-md);
  }

  .empty-state {
    text-align: center;
    padding: var(--spacing-xl);
    color: var(--color-on-surface-muted);
  }
</style>
