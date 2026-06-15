<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    title: string;
    seeAllHref?: string;
    children: Snippet;
  }

  let { title, seeAllHref, children }: Props = $props();
</script>

<section class="feed-section">
  <div class="section-header">
    <h2 class="section-title">{title}</h2>
    {#if seeAllHref}
      <a href={seeAllHref} class="see-all">See All →</a>
    {/if}
  </div>
  <div class="section-content">
    {@render children()}
  </div>
</section>

<style>
  .feed-section {
    margin-bottom: var(--space-8);
  }

  .section-header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    margin-bottom: var(--space-4);
  }

  .section-title {
    margin: 0;
    font-family: var(--font-display);
    font-size: 2.25rem;
    font-weight: 700;
    color: var(--text-bright);
    letter-spacing: -0.02em;
  }

  .see-all {
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--glow-primary);
    text-decoration: none;
    transition: color 150ms var(--ease-snap);
  }

  .see-all:hover {
    color: var(--glow-warm);
  }
</style>
