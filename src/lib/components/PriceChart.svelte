<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  interface PricePoint {
    source_id: string;
    price: number;
    recorded_at: number;
  }

  interface SvgSource {
    source_id: string;
    points: string;
    color: string;
    count: number;
  }

  interface SvgData {
    width: number;
    height: number;
    sources: SvgSource[];
    insufficientSources: string[];
  }

  interface Props {
    sku: string;
    windowDays?: number;
  }

  let { sku, windowDays = 365 }: Props = $props();
  let points = $state<PricePoint[]>([]);
  let loading = $state<boolean>(true);
  let error = $state<unknown>(null);

  onMount(async () => {
    if (!sku?.trim()) {
      loading = false;
      return;
    }
    try {
      points = await invoke<PricePoint[]>('get_price_history', { sku, windowDays });
    } catch (e) {
      error = e;
    } finally {
      loading = false;
    }
  });

  // Color palette (5 colors, cycle if more sources)
  const COLORS = ['#4e79a7', '#f28e2b', '#e15759', '#76b7b2', '#59a14f'];
  const CHART_HEIGHT = 200;
  const CHART_PADDING = 10;

  let svgData = $derived(computeSvgData(points));

  function computeSvgData(points: PricePoint[]): SvgData {
    if (!points || points.length === 0) {
      return { width: 400, height: CHART_HEIGHT, sources: [], insufficientSources: [] };
    }

    // Group by source_id
    const groups: Record<string, PricePoint[]> = {};
    for (const p of points) {
      if (!groups[p.source_id]) groups[p.source_id] = [];
      groups[p.source_id].push(p);
    }

    // Compute global extents
    let minTime = Infinity, maxTime = -Infinity;
    let minPrice = Infinity, maxPrice = -Infinity;
    for (const p of points) {
      if (p.recorded_at < minTime) minTime = p.recorded_at;
      if (p.recorded_at > maxTime) maxTime = p.recorded_at;
      if (p.price < minPrice) minPrice = p.price;
      if (p.price > maxPrice) maxPrice = p.price;
    }

    const timeRange = Math.max(maxTime - minTime, 1);
    const priceRange = Math.max(maxPrice - minPrice, 1);
    const width = Math.max(timeRange, 400);
    const height = CHART_HEIGHT;

    const sourceIds = Object.keys(groups);
    const insufficientSources: string[] = [];

    const sources = sourceIds.map((sid, idx) => {
      const pts = groups[sid];
      const color = COLORS[idx % COLORS.length];

      // Downsample if >500 points
      let display = pts;
      if (pts.length > 500) {
        display = [];
        const step = pts.length / 500;
        for (let i = 0; i < 500; i++) {
          display.push(pts[Math.floor(i * step)]);
        }
      }

      // Insufficient data flag
      if (pts.length < 30) {
        insufficientSources.push(sid);
      }

      // Build polyline points string
      const polyline = display
        .map(
          (p) =>
            `${((p.recorded_at - minTime) / timeRange) * (width - 2 * CHART_PADDING) + CHART_PADDING},` +
            `${height - CHART_PADDING - ((p.price - minPrice) / priceRange) * (height - 2 * CHART_PADDING)}`
        )
        .join(' ');

      return { source_id: sid, points: polyline, color, count: pts.length };
    });

    return { width, height, sources, insufficientSources };
  }
</script>

{#if loading}
  <div aria-busy="true" class="chart-placeholder">Loading chart...</div>
{:else if error}
  <div class="empty-state" role="status">No price history available</div>
{:else if svgData.sources.length === 0}
  <div class="empty-state" role="status">No price history available</div>
{:else}
  <div class="chart-container" role="img" aria-label="Price history chart for {sku}">
    <svg
      viewBox="0 0 {svgData.width} {svgData.height}"
      preserveAspectRatio="xMidYMid meet"
      style="width: 100%; height: auto; max-height: {CHART_HEIGHT}px;"
    >
      <title>Price history for {sku}</title>
      {#each svgData.sources as source}
        <polyline
          points={source.points}
          fill="none"
          stroke={source.color}
          stroke-width="2"
          vector-effect="non-scaling-stroke"
        />
      {/each}
    </svg>
    {#if svgData.insufficientSources.length > 0}
      <p class="note">
        Insufficient data for {svgData.insufficientSources.join(', ')}
      </p>
    {/if}
  </div>
{/if}

<style>
  .chart-container {
    width: 100%;
    margin: 8px 0;
  }
  .chart-placeholder {
    height: 200px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-on-surface-variant);
    background: var(--color-surface);
    border-radius: 8px;
  }
  .empty-state {
    height: 200px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-on-surface-variant);
    background: var(--color-surface);
    border-radius: 8px;
    font-size: 0.9rem;
  }
  .note {
    font-size: 0.8rem;
    color: var(--color-warning);
    margin: 4px 0 0;
    text-align: center;
  }
</style>
