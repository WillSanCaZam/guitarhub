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
    background: var(--color-surface-container-high, #242430);
    border-radius: var(--radius-full, 9999px);
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: var(--color-primary, #d4a017);
    border-radius: var(--radius-full, 9999px);
    transition: width var(--transition-base, 200ms ease);
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
