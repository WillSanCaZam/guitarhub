<script lang="ts">
  interface Props {
    rating: number;
    reviewCount?: number;
    size?: 'sm' | 'md' | 'lg';
  }

  let { rating, reviewCount, size = 'md' }: Props = $props();

  const stars = $derived(
    Array.from({ length: 5 }, (_, i) => {
      const filled = Math.min(1, Math.max(0, rating - i));
      return filled;
    })
  );

  const sizeMap = { sm: 14, md: 16, lg: 20 };
  const starSize = $derived(sizeMap[size]);
</script>

<span class="star-rating" aria-label="{rating} out of 5 stars">
  {#each stars as filled, i}
    <svg
      width={starSize}
      height={starSize}
      viewBox="0 0 24 24"
      class="star"
      class:empty={filled === 0}
      class:partial={filled > 0 && filled < 1}
    >
      <defs>
        {#if filled > 0 && filled < 1}
          <linearGradient id="partial-{i}">
            <stop offset="{filled * 100}%" stop-color="var(--glow-gold)" />
            <stop offset="{filled * 100}%" stop-color="var(--text-muted)" />
          </linearGradient>
        {/if}
      </defs>
      <path
        d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"
        fill={filled === 1
          ? 'var(--glow-gold)'
          : filled === 0
            ? 'var(--text-muted)'
            : `url(#partial-${i})`}
        stroke="none"
      />
    </svg>
  {/each}
  {#if reviewCount !== undefined}
    <span class="review-count">({rating.toFixed(1)}) · {reviewCount.toLocaleString()} reviews</span>
  {/if}
</span>

<style>
  .star-rating {
    display: inline-flex;
    align-items: center;
    gap: 2px;
  }

  .star {
    flex-shrink: 0;
  }

  .review-count {
    margin-left: var(--space-1);
    font-family: var(--font-body);
    font-size: 0.8rem;
    color: var(--text-dim);
  }
</style>
