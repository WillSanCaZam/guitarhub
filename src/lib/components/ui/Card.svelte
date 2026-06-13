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
    background: var(--color-surface-container, #1c1c26);
    border: 1px solid var(--color-outline-variant, #2a2a38);
    border-radius: var(--radius-lg, 16px);
    overflow: hidden;
    transition: box-shadow var(--transition-base, 200ms ease),
                border-color var(--transition-base, 200ms ease);
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
      rgba(255, 255, 255, 0.06),
      transparent
    );
    pointer-events: none;
  }

  .card:hover {
    box-shadow: var(--elevation-2, 0 2px 6px rgba(0, 0, 0, 0.5));
    border-color: var(--color-outline, #3a3a4a);
  }

  .card-media {
    width: 100%;
    aspect-ratio: 16 / 9;
    overflow: hidden;
    background: var(--color-surface-container-low, #161620);
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
