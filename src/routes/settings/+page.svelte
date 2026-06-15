<script>
  import Settings from '$lib/components/Settings.svelte';
</script>

<div class="page">
  <h1>Settings</h1>
  <Settings />
</div>

<style>
  .page {
    max-width: 480px;
    margin: 0 auto;
    padding: 16px;
  }

  h1 {
    margin: 0 0 16px;
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--color-secondary);
  }

  @media (prefers-color-scheme: dark) {
    h1 {
      color: var(--color-on-surface);
    }
  }
</style>
