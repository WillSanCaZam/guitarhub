<script lang="ts">
  import { onMount } from 'svelte';
  import CollectionView from '$lib/components/CollectionView.svelte';
  import { loadCollection, loadCollectionStats } from '$lib/stores/collection.svelte';

  onMount(() => {
    loadCollection();
    loadCollectionStats();
  });
</script>

<div class="page">
  <header class="collection-header">
    <a href="/" class="back-link">← Back to Dashboard</a>
    <h1>My Collection</h1>
  </header>

  <CollectionView />
</div>

<style>
  .page {
    max-width: 1200px;
    margin: 0 auto;
    padding: 16px;
  }

  .collection-header {
    margin-bottom: 24px;
  }

  .back-link {
    display: inline-block;
    margin-bottom: 12px;
    color: #4a90d9;
    text-decoration: none;
    font-size: 0.9rem;
  }

  .back-link:hover {
    text-decoration: underline;
  }

  h1 {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 700;
    color: #1a1a2e;
  }

  @media (prefers-color-scheme: dark) {
    h1 {
      color: #e8e8f0;
    }

    .back-link {
      color: #7ab8e8;
    }
  }
</style>
