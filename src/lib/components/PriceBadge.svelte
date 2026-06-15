<script lang="ts">
  type BadgeLevel = 'green' | 'amber';

  interface Props {
    level?: BadgeLevel;
    pct?: number;
    confidence?: number;
    cnt_30d?: number;
    source_count_30d?: number;
    last_recorded_at?: number;
    min_30d?: number;
    avg_90d?: number;
    current?: number;
  }

  let {
    level = 'green' as BadgeLevel,
    pct = 0,
    confidence = 0,
    cnt_30d,
    source_count_30d,
    last_recorded_at,
    min_30d,
    avg_90d,
    current,
  }: Props = $props();

  /**
   * Map a 0-100 confidence score to one of three named tiers.
   * Tiers are derived in the UI from the numeric value; the server
   * returns only the number.
   */
  function getTier(c: number): string {
    if (c >= 80) return 'high';
    if (c >= 50) return 'medium';
    return 'low';
  }

  /**
   * 3-dot glyph representing the tier. Filled count = tier rank.
   * `aria-hidden` because the tier is also surfaced in the aria-label.
   */
  function dotsFor(t: string): string {
    if (t === 'high') return '•••';
    if (t === 'medium') return '••○';
    return '•○○';
  }

  let tier = $derived(getTier(confidence));
  let dots = $derived(dotsFor(tier));
  let tierTitle = $derived(tier.charAt(0).toUpperCase() + tier.slice(1));
  let confidenceLabel = $derived(`Confidence: ${confidence}% (${tierTitle})`);

  let titleLines = $derived(() => {
    const lines = [confidenceLabel];
    if (cnt_30d !== undefined && source_count_30d !== undefined && last_recorded_at !== undefined) {
      lines.push(`${cnt_30d} data points · ${source_count_30d} sources · last ${last_recorded_at} days ago`);
    }
    if (min_30d !== undefined && avg_90d !== undefined && current !== undefined) {
      lines.push(`Min 30d: $${min_30d.toFixed(2)}  |  Avg 90d: $${avg_90d.toFixed(2)}  |  Current: $${current.toFixed(2)}`);
    }
    return lines;
  });

  let title = $derived(titleLines().join('\n'));

  let ariaContext = $derived(() => {
    const parts = [];
    if (cnt_30d !== undefined && source_count_30d !== undefined && last_recorded_at !== undefined) {
      parts.push(`${cnt_30d} data points, ${source_count_30d} sources, last ${last_recorded_at} days ago`);
    }
    return parts;
  });
</script>

{#if level === 'green'}
  <span
    class="badge badge--green tier--{tier}"
    role="status"
    aria-label="Good price, {pct}% above 30-day low. Confidence {confidence}%, {tier}.{ariaContext().length ? ' ' + ariaContext().join('. ') : ''}"
    {title}
  >
    ✓ Good price <span class="dots" aria-hidden="true">{dots}</span>
  </span>
{:else if level === 'amber'}
  <span
    class="badge badge--amber tier--{tier}"
    role="status"
    aria-label="Above average price, {pct}% above 90-day average. Confidence {confidence}%, {tier}.{ariaContext().length ? ' ' + ariaContext().join('. ') : ''}"
    {title}
  >
    ↑ Above average <span class="dots" aria-hidden="true">{dots}</span>
  </span>
{/if}

<style>
  .badge {
    display: inline-block;
    font-size: 0.75rem;
    padding: 2px 8px;
    border-radius: 4px;
    font-weight: 600;
    margin-left: 8px;
    vertical-align: middle;
  }
  .badge--green {
    background: var(--color-success-container);
    color: var(--color-success);
  }
  .badge--amber {
    background: var(--color-on-surface)3cd;
    color: var(--color-warning);
  }
  .dots {
    margin-left: 4px;
    font-weight: 700;
    letter-spacing: 1px;
  }
  /* Low-tier dots are neutral grey regardless of badge colour. */
  .tier--low .dots {
    color: var(--color-on-surface-variant);
  }
  /* High/medium tier dots inherit the badge colour for cohesion. */
  .tier--high .dots,
  .tier--medium .dots {
    color: inherit;
  }
</style>
