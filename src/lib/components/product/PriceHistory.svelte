<script lang="ts">
  interface PricePoint {
    date: string;
    price: number;
  }

  interface Props {
    history: PricePoint[];
  }

  let { history }: Props = $props();

  const lowestPrice = $derived(Math.min(...history.map(p => p.price)));
  const avgPrice = $derived(
    Math.round(history.reduce((sum, p) => sum + p.price, 0) / history.length)
  );

  const svgPath = $derived.by(() => {
    if (history.length < 2) return '';
    const prices = history.map(p => p.price);
    const min = Math.min(...prices);
    const max = Math.max(...prices);
    const range = max - min || 1;
    const width = 400;
    const height = 80;
    const padding = 10;

    const points = prices.map((price, i) => {
      const x = padding + (i / (prices.length - 1)) * (width - 2 * padding);
      const y = padding + (1 - (price - min) / range) * (height - 2 * padding);
      return { x, y };
    });

    return points.map((p, i) => `${i === 0 ? 'M' : 'L'} ${p.x} ${p.y}`).join(' ');
  });

  const svgArea = $derived.by(() => {
    if (history.length < 2) return '';
    const prices = history.map(p => p.price);
    const min = Math.min(...prices);
    const max = Math.max(...prices);
    const range = max - min || 1;
    const width = 400;
    const height = 80;
    const padding = 10;

    const points = prices.map((price, i) => {
      const x = padding + (i / (prices.length - 1)) * (width - 2 * padding);
      const y = padding + (1 - (price - min) / range) * (height - 2 * padding);
      return { x, y };
    });

    const line = points.map((p, i) => `${i === 0 ? 'M' : 'L'} ${p.x} ${p.y}`).join(' ');
    return `${line} L ${points[points.length - 1].x} ${height - padding} L ${points[0].x} ${height - padding} Z`;
  });
</script>

<div class="price-history">
  <div class="chart-container">
    <svg viewBox="0 0 400 80" class="sparkline" preserveAspectRatio="none">
      <defs>
        <linearGradient id="areaGradient" x1="0%" y1="0%" x2="0%" y2="100%">
          <stop offset="0%" stop-color="var(--glow-primary)" stop-opacity="0.3" />
          <stop offset="100%" stop-color="var(--glow-primary)" stop-opacity="0" />
        </linearGradient>
      </defs>
      {#if svgArea}
        <path d={svgArea} fill="url(#areaGradient)" />
      {/if}
      {#if svgPath}
        <path d={svgPath} fill="none" stroke="var(--glow-primary)" stroke-width="2" />
      {/if}
    </svg>
  </div>

  <div class="price-stats">
    <div class="stat">
      <span class="stat-label">Lowest</span>
      <span class="stat-value best">${lowestPrice.toLocaleString()}</span>
    </div>
    <div class="stat">
      <span class="stat-label">Average</span>
      <span class="stat-value">${avgPrice.toLocaleString()}</span>
    </div>
  </div>

  <button class="alert-btn">
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="16" height="16">
      <path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/>
      <path d="M13.73 21a2 2 0 0 1-3.46 0"/>
    </svg>
    Set Price Alert
  </button>
</div>

<style>
  .price-history {
    background: var(--void-raised);
    border-radius: var(--radius-md);
    padding: var(--space-4);
  }

  .chart-container {
    margin-bottom: var(--space-3);
  }

  .sparkline {
    width: 100%;
    height: 80px;
  }

  .price-stats {
    display: flex;
    gap: var(--space-6);
    margin-bottom: var(--space-3);
  }

  .stat {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .stat-label {
    font-size: 0.75rem;
    color: var(--text-dim);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .stat-value {
    font-family: var(--font-mono);
    font-weight: 700;
    font-size: 1.1rem;
    color: var(--text-bright);
  }

  .stat-value.best {
    color: var(--success);
  }

  .alert-btn {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-4);
    background: var(--glow-primary);
    color: var(--void-deep);
    border: none;
    border-radius: var(--radius-sm);
    font-size: 0.8rem;
    font-weight: 600;
    cursor: pointer;
    transition: background 150ms var(--ease-snap);
  }

  .alert-btn:hover {
    background: var(--glow-warm);
  }
</style>
