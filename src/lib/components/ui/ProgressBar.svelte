<script lang="ts">
  interface Props {
    value: number;
    max?: number;
  }

  let { value, max = 100 }: Props = $props();

  let percentage = $derived(Math.min(100, Math.max(0, (value / max) * 100)));
</script>

<div
  class="progress-bar"
  role="progressbar"
  aria-valuenow={value}
  aria-valuemin={0}
  aria-valuemax={max}
  aria-label="Progress"
>
  <div
    class="progress-fill"
    style="width: {percentage}%"
  ></div>
</div>

<style>
  .progress-bar {
    width: 100%;
    height: 4px;
    background: var(--color-surface-container-high);
    border-radius: var(--radius-full);
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: var(--color-primary);
    border-radius: var(--radius-full);
    transition: width var(--transition-base);
    animation: progress-pulse 2s ease-in-out infinite;
  }

  @keyframes progress-pulse {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.8;
    }
  }
</style>
