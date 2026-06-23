<script lang="ts">
  interface Props {
    variant: 'card-grid' | 'card-list' | 'text' | 'hero' | 'detail'
    count?: number
  }

  let { variant, count = 1 }: Props = $props()
</script>

{#if variant === 'card-grid'}
  <div class="skeleton-card-grid" aria-busy="true" aria-label="Loading content">
    {#each Array(count) as _, i}
      <div class="skeleton-card" style="animation-delay: {i * 60}ms">
        <div class="skeleton-image"></div>
        <div class="skeleton-text"></div>
        <div class="skeleton-text short"></div>
      </div>
    {/each}
  </div>
{:else if variant === 'card-list'}
  <div class="skeleton-card-list" aria-busy="true" aria-label="Loading content">
    {#each Array(count) as _, i}
      <div class="skeleton-card-row" style="animation-delay: {i * 60}ms">
        <div class="skeleton-thumb"></div>
        <div class="skeleton-card-info">
          <div class="skeleton-text"></div>
          <div class="skeleton-text short"></div>
        </div>
      </div>
    {/each}
  </div>
{:else if variant === 'text'}
  <div class="skeleton-text-block" aria-busy="true" aria-label="Loading text">
    {#each Array(count) as _, i}
      <div class="skeleton-text-line" style="animation-delay: {i * 40}ms; width: {70 + (i % 3) * 10}%"></div>
    {/each}
  </div>
{:else if variant === 'hero'}
  <div class="skeleton-hero" aria-busy="true" aria-label="Loading hero">
    <div class="skeleton-hero-title"></div>
    <div class="skeleton-hero-subtitle"></div>
    <div class="skeleton-hero-search"></div>
  </div>
{:else if variant === 'detail'}
  <div class="skeleton-detail" aria-busy="true" aria-label="Loading product details">
    <div class="skeleton-gallery"></div>
    <div class="skeleton-info">
      <div class="skeleton-text wide"></div>
      <div class="skeleton-text"></div>
      <div class="skeleton-text short"></div>
      <div class="skeleton-text medium"></div>
    </div>
  </div>
{/if}

<style>
  /* Shimmer animation — uses the canonical @keyframes shimmer from animations.css */
  .skeleton-image,
  .skeleton-thumb,
  .skeleton-text,
  .skeleton-text-line,
  .skeleton-hero-title,
  .skeleton-hero-subtitle,
  .skeleton-hero-search,
  .skeleton-gallery {
    background: linear-gradient(
      90deg,
      var(--shimmer-base) 25%,
      var(--shimmer-highlight) 50%,
      var(--shimmer-base) 75%
    );
    background-size: 200% 100%;
    animation: shimmer 1.5s infinite;
  }

  /* Card Grid */
  .skeleton-card-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: var(--space-5);
  }

  .skeleton-card {
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
    overflow: hidden;
    background: var(--void-mid);
    animation: fadeIn 300ms var(--ease-plug) both;
  }

  .skeleton-image {
    width: 100%;
    aspect-ratio: 16 / 10;
  }

  .skeleton-text {
    height: 14px;
    margin: 12px 16px 0;
    border-radius: 4px;
  }

  .skeleton-text.short {
    width: 60%;
    margin-bottom: 16px;
  }

  .skeleton-text.wide {
    width: 80%;
  }

  .skeleton-text.medium {
    width: 45%;
  }

  /* Card List */
  .skeleton-card-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .skeleton-card-row {
    display: flex;
    gap: var(--space-3);
    padding: var(--space-3);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    background: var(--void-mid);
    animation: fadeIn 300ms var(--ease-plug) both;
  }

  .skeleton-thumb {
    width: 80px;
    height: 60px;
    border-radius: var(--radius-sm);
    flex-shrink: 0;
  }

  .skeleton-card-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  /* Text Block */
  .skeleton-text-block {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .skeleton-text-line {
    height: 16px;
    border-radius: 4px;
    animation: fadeIn 300ms var(--ease-plug) both;
  }

  /* Hero */
  .skeleton-hero {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-16) var(--space-8);
  }

  .skeleton-hero-title {
    width: 300px;
    height: 40px;
    border-radius: var(--radius-sm);
  }

  .skeleton-hero-subtitle {
    width: 200px;
    height: 20px;
    border-radius: var(--radius-sm);
  }

  .skeleton-hero-search {
    width: 400px;
    height: 56px;
    border-radius: var(--radius-lg);
  }

  /* Detail */
  .skeleton-detail {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-8);
  }

  .skeleton-gallery {
    aspect-ratio: 4 / 3;
    border-radius: var(--radius-lg);
  }

  .skeleton-info {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  /* Reduced motion */
  @media (prefers-reduced-motion: reduce) {
    .skeleton-image,
    .skeleton-thumb,
    .skeleton-text,
    .skeleton-text-line,
    .skeleton-hero-title,
    .skeleton-hero-subtitle,
    .skeleton-hero-search,
    .skeleton-gallery {
      animation: none;
      opacity: 0.6;
    }

    .skeleton-card,
    .skeleton-card-row,
    .skeleton-text-line {
      animation: none;
    }
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(8px); }
    to { opacity: 1; transform: translateY(0); }
  }

  @media (max-width: 768px) {
    .skeleton-detail {
      grid-template-columns: 1fr;
    }
  }
</style>
