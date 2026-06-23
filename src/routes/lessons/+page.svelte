<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- Lessons route — lesson list with difficulty filter -->
<script lang="ts">
  import { communityState, loadFeed } from '$lib/stores/community.svelte';
  import LessonCard from '$lib/components/community/LessonCard.svelte';
  import AuthGuard from '$lib/components/auth/AuthGuard.svelte';
  import EmptyState from '$lib/components/ui/EmptyState.svelte';
  import type { Lesson } from '$lib/types/community';
  import { onMount } from 'svelte';

  let difficultyFilter = $state<string>('all');

  onMount(() => {
    loadFeed(true);
  });

  const filteredLessons = $derived(() => {
    return communityState.feed
      .filter(item => item.itemType === 'lesson')
      .filter(item => {
        if (difficultyFilter === 'all') return true;
        return (item.content as Lesson).difficulty === difficultyFilter;
      })
      .map(item => item.content as Lesson);
  });
</script>

<svelte:head>
  <title>Lessons — GuitarHub</title>
</svelte:head>

<AuthGuard>
  <div class="lessons-page">
    <header class="page-header">
      <h1>Lessons</h1>
      <div class="filters">
        <button class="filter-btn" class:active={difficultyFilter === 'all'} onclick={() => difficultyFilter = 'all'}>All</button>
        <button class="filter-btn" class:active={difficultyFilter === 'beginner'} onclick={() => difficultyFilter = 'beginner'}>Beginner</button>
        <button class="filter-btn" class:active={difficultyFilter === 'intermediate'} onclick={() => difficultyFilter = 'intermediate'}>Intermediate</button>
        <button class="filter-btn" class:active={difficultyFilter === 'advanced'} onclick={() => difficultyFilter = 'advanced'}>Advanced</button>
      </div>
    </header>

    <div class="lessons-grid">
      {#each filteredLessons() as lesson (lesson.id)}
        <LessonCard {lesson} />
      {/each}
    </div>

    {#if filteredLessons().length === 0 && !communityState.loading}
      <EmptyState
        variant="lessons"
        title="No lessons found"
        description="Try a different filter."
      />
    {/if}
  </div>
</AuthGuard>

<style>
  .lessons-page {
    max-width: 800px;
    margin: 0 auto;
    padding: var(--spacing-md);
  }

  .page-header {
    margin-bottom: var(--spacing-md);
  }

  .page-header h1 {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--color-on-surface);
    margin: 0 0 var(--spacing-md) 0;
  }

  .filters {
    display: flex;
    gap: var(--spacing-xs);
  }

  .filter-btn {
    padding: var(--spacing-xs) var(--spacing-md);
    background: var(--color-surface-container);
    border: 1px solid var(--color-outline-variant);
    border-radius: var(--radius-full);
    color: var(--color-on-surface-variant);
    font-size: 0.85rem;
    cursor: pointer;
    transition: background var(--transition-fast);
  }

  .filter-btn:hover {
    background: var(--color-surface-container-high);
  }

  .filter-btn.active {
    background: var(--color-primary-container);
    color: var(--color-on-primary-container);
    border-color: var(--color-primary);
  }

  .lessons-grid {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }
</style>
