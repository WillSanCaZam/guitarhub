<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!--
  StreakCounter — Practice streak display.
  Shows current streak, longest streak, calendar heatmap.
-->
<script lang="ts">
  import type { Streak } from '$lib/types/community';

  interface Props {
    streak: Streak;
  }

  let { streak }: Props = $props();

  // Generate last 7 days for mini heatmap
  function getLast7Days(): { date: string; count: number }[] {
    const days = [];
    const today = new Date();
    for (let i = 6; i >= 0; i--) {
      const d = new Date(today);
      d.setDate(d.getDate() - i);
      const key = d.toISOString().split('T')[0];
      days.push({
        date: key,
        count: streak.calendarHeatmap[key] ?? 0,
      });
    }
    return days;
  }

  const last7Days = $derived(getLast7Days());
</script>

<div class="streak-counter">
  <div class="streak-numbers">
    <div class="streak-stat main">
      <span class="streak-value">{streak.currentStreak}</span>
      <span class="streak-label">Current Streak</span>
    </div>
    <div class="streak-divider"></div>
    <div class="streak-stat">
      <span class="streak-value secondary">{streak.longestStreak}</span>
      <span class="streak-label">Longest</span>
    </div>
  </div>

  <div class="heatmap">
    {#each last7Days as day}
      <div
        class="heatmap-cell"
        class:active={day.count > 0}
        class:hot={day.count >= 3}
        title={`${day.date}: ${day.count} sessions`}
      ></div>
    {/each}
  </div>

  <div class="heatmap-labels">
    {#each last7Days as day, i}
      <span class="heatmap-label">{i === 6 ? 'Now' : ''}</span>
    {/each}
  </div>
</div>

<style>
  .streak-counter {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
    padding: var(--spacing-md);
    background: var(--color-surface-container-low);
    border-radius: var(--radius-md);
    border: 1px solid var(--color-outline-variant);
  }

  .streak-numbers {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-lg);
  }

  .streak-stat {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-2xs);
  }

  .streak-value {
    font-size: 2rem;
    font-weight: 700;
    color: var(--color-primary);
    line-height: 1;
  }

  .streak-value.secondary {
    font-size: 1.25rem;
    color: var(--color-on-surface-variant);
  }

  .streak-label {
    font-size: 0.7rem;
    color: var(--color-on-surface-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .streak-divider {
    width: 1px;
    height: 40px;
    background: var(--color-outline-variant);
  }

  .heatmap {
    display: flex;
    gap: var(--spacing-xs);
    justify-content: center;
  }

  .heatmap-cell {
    width: 24px;
    height: 24px;
    border-radius: var(--radius-xs);
    background: var(--color-surface-container);
    border: 1px solid var(--color-outline-variant);
    transition: background var(--transition-fast);
  }

  .heatmap-cell.active {
    background: var(--color-primary-container);
    border-color: var(--color-primary);
  }

  .heatmap-cell.hot {
    background: var(--color-primary);
    border-color: var(--color-primary);
  }

  .heatmap-labels {
    display: flex;
    gap: var(--spacing-xs);
    justify-content: center;
  }

  .heatmap-label {
    width: 24px;
    text-align: center;
    font-size: 0.6rem;
    color: var(--color-on-surface-muted);
  }
</style>
