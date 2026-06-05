<script>
  let { level = 'green', pct = 0, confidence = 0 } = $props();

  /**
   * Map a 0-100 confidence score to one of three named tiers.
   * Tiers are derived in the UI from the numeric value; the server
   * returns only the number.
   */
  function getTier(c) {
    if (c >= 80) return 'high';
    if (c >= 50) return 'medium';
    return 'low';
  }

  /**
   * 3-dot glyph representing the tier. Filled count = tier rank.
   * `aria-hidden` because the tier is also surfaced in the aria-label.
   */
  function dotsFor(t) {
    if (t === 'high') return '•••';
    if (t === 'medium') return '••○';
    return '•○○';
  }

  let tier = $derived(getTier(confidence));
  let dots = $derived(dotsFor(tier));
  let confidenceLabel = $derived(`Confidence: ${confidence}% (${tier})`);
</script>

{#if level === 'green'}
  <span
    class="badge badge--green tier--{tier}"
    role="status"
    aria-label="Good price, {pct}% above 30-day low. Confidence {confidence}%, {tier}."
    title={`${confidenceLabel}\nConfidence reflects data quality (volume, recency, source diversity, stability).`}
  >
    ✓ Good price <span class="dots" aria-hidden="true">{dots}</span>
  </span>
{:else if level === 'amber'}
  <span
    class="badge badge--amber tier--{tier}"
    role="status"
    aria-label="Above average price, {pct}% above 90-day average. Confidence {confidence}%, {tier}."
    title={`${confidenceLabel}\nConfidence reflects data quality (volume, recency, source diversity, stability).`}
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
    background: #d4edda;
    color: #155724;
  }
  .badge--amber {
    background: #fff3cd;
    color: #856404;
  }
  .dots {
    margin-left: 4px;
    font-weight: 700;
    letter-spacing: 1px;
  }
  /* Low-tier dots are neutral grey regardless of badge colour. */
  .tier--low .dots {
    color: #6c757d;
  }
  /* High/medium tier dots inherit the badge colour for cohesion. */
  .tier--high .dots,
  .tier--medium .dots {
    color: inherit;
  }
</style>
