<script lang="ts">
  interface Props {
    min: number;
    max: number;
    step?: number;
    value: [number, number];
    onchange?: (value: [number, number]) => void;
  }

  let { min, max, step = 1, value = $bindable([min, max]), onchange }: Props = $props();

  let trackEl = $state<HTMLElement | null>(null);
  let dragging = $state<'min' | 'max' | null>(null);

  const minPercent = $derived(((value[0] - min) / (max - min)) * 100);
  const maxPercent = $derived(((value[1] - min) / (max - min)) * 100);

  function handlePointerDown(e: PointerEvent, thumb: 'min' | 'max') {
    dragging = thumb;
    (e.target as HTMLElement).setPointerCapture(e.pointerId);
  }

  function handlePointerMove(e: PointerEvent) {
    if (!dragging || !trackEl) return;
    const rect = trackEl.getBoundingClientRect();
    const percent = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
    const raw = min + percent * (max - min);
    const stepped = Math.round(raw / step) * step;

    if (dragging === 'min') {
      const newMin = Math.min(stepped, value[1] - step);
      value = [Math.max(min, newMin), value[1]];
    } else {
      const newMax = Math.max(stepped, value[0] + step);
      value = [value[0], Math.min(max, newMax)];
    }
  }

  function handlePointerUp() {
    dragging = null;
    onchange?.(value);
  }
</script>

<div class="slider-container">
  <div class="slider-labels">
    <span class="slider-value">${value[0].toLocaleString()}</span>
    <span class="slider-separator">—</span>
    <span class="slider-value">${value[1].toLocaleString()}</span>
  </div>

  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
  <div
    class="slider-track"
    bind:this={trackEl}
    role="slider"
    tabindex="0"
    aria-valuemin={min}
    aria-valuemax={max}
    aria-valuenow={value[0]}
    aria-label="Price range"
  >
    <div
      class="slider-range"
      style="left: {minPercent}%; width: {maxPercent - minPercent}%;"
    ></div>

    <!-- Min Thumb -->
    <div
      class="slider-thumb"
      style="left: {minPercent}%;"
      onpointerdown={(e) => handlePointerDown(e, 'min')}
      onpointermove={handlePointerMove}
      onpointerup={handlePointerUp}
      role="slider"
      tabindex="0"
      aria-label="Minimum price"
      aria-valuemin={min}
      aria-valuemax={value[1]}
      aria-valuenow={value[0]}
    ></div>

    <!-- Max Thumb -->
    <div
      class="slider-thumb"
      style="left: {maxPercent}%;"
      onpointerdown={(e) => handlePointerDown(e, 'max')}
      onpointermove={handlePointerMove}
      onpointerup={handlePointerUp}
      role="slider"
      tabindex="0"
      aria-label="Maximum price"
      aria-valuemin={value[0]}
      aria-valuemax={max}
      aria-valuenow={value[1]}
    ></div>
  </div>

  <div class="slider-bounds">
    <span>${min.toLocaleString()}</span>
    <span>${max.toLocaleString()}</span>
  </div>
</div>

<style>
  .slider-container {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .slider-labels {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
  }

  .slider-value {
    font-family: var(--font-mono);
    font-weight: 600;
    font-size: 0.95rem;
    color: var(--glow-primary);
    min-width: 60px;
    text-align: center;
  }

  .slider-separator {
    color: var(--text-muted);
  }

  .slider-track {
    position: relative;
    height: 6px;
    background: var(--void-mid);
    border-radius: var(--radius-pill);
    cursor: pointer;
  }

  .slider-range {
    position: absolute;
    height: 100%;
    background: var(--glow-primary);
    border-radius: var(--radius-pill);
  }

  .slider-thumb {
    position: absolute;
    top: 50%;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: var(--glow-primary);
    border: 2px solid var(--void-deep);
    transform: translate(-50%, -50%);
    cursor: grab;
    transition: box-shadow 150ms var(--ease-snap);
    z-index: 1;
  }

  .slider-thumb:hover {
    box-shadow: 0 0 0 4px var(--glow-soft);
  }

  .slider-thumb:active {
    cursor: grabbing;
    box-shadow: 0 0 0 6px var(--glow-medium);
  }

  .slider-bounds {
    display: flex;
    justify-content: space-between;
    font-size: 0.75rem;
    color: var(--text-dim);
  }
</style>
