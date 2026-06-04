<script>
  let { level = 'green', pct = 0 } = $props();
</script>

{#if level === 'green'}
  <span class="badge badge--green" role="status" aria-label="Good price, {pct}% above 30-day low" title="Current price is near the 30-day low">
    ✓ Good price
  </span>
{:else if level === 'amber'}
  <span class="badge badge--amber" role="status" aria-label="Above average price, {pct}% above 90-day average" title="Current price is above the 90-day average">
    ↑ Above average
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
</style>
