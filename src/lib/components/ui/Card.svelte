<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    children: Snippet;
    media?: string;
    class?: string;
  }

  let { children, media, class: className = '' }: Props = $props();
</script>

<div class="card {className}">
  {#if media}
    <div class="card-media">
      <img src={media} alt="" loading="lazy" />
    </div>
  {/if}
  <div class="card-body">
    {@render children()}
  </div>
</div>

<style>
  .card {
    background: var(--void-raised);
    border: 1px solid var(--color-outline-variant);
    border-radius: var(--radius-lg);
    overflow: hidden;
    transition: box-shadow var(--transition-base),
                border-color var(--transition-base);
    position: relative;
  }

  .card::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 1px;
    background: linear-gradient(
      90deg,
      transparent,
      var(--glow-soft),
      transparent
    );
    pointer-events: none;
  }

  .card:hover {
    box-shadow: var(--shadow-hover);
    border-color: var(--text-muted);
  }

  .card-media {
    width: 100%;
    aspect-ratio: 16 / 9;
    overflow: hidden;
    background: var(--void-mid);
  }

  .card-media img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .card-body {
    padding: var(--spacing-md, 16px);
  }
</style>
