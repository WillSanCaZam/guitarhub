<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- My Gear route — user's gear list, add/remove, link to catalog items -->
<script lang="ts">
  import { profileState, loadProfile, addGear } from '$lib/stores/profile.svelte';
  import { authState } from '$lib/stores/auth.svelte';
  import AuthGuard from '$lib/components/auth/AuthGuard.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import { onMount } from 'svelte';

  let newSku = $state('');
  let adding = $state(false);

  onMount(() => {
    if (authState.user) {
      loadProfile(authState.user.id);
    }
  });

  async function handleAdd() {
    if (!newSku.trim() || !authState.user || adding) return;
    adding = true;
    try {
      await addGear(authState.user.id, newSku.trim());
      newSku = '';
    } catch (e) {
      console.error('Failed to add gear:', e);
    } finally {
      adding = false;
    }
  }
</script>

<svelte:head>
  <title>My Gear — GuitarHub</title>
</svelte:head>

<AuthGuard>
  <div class="my-gear-page">
    <header class="page-header">
      <h1>My Gear Collection</h1>
    </header>

    <form class="add-form" onsubmit={(e) => { e.preventDefault(); handleAdd(); }}>
      <input
        type="text"
        bind:value={newSku}
        placeholder="Enter gear SKU or model"
        disabled={adding}
      />
      <Button variant="primary" disabled={!newSku.trim() || adding}>
        {adding ? 'Adding...' : 'Add'}
      </Button>
    </form>

    {#if profileState.profile}
      <div class="gear-count">{profileState.profile.gearList.length} items</div>

      {#if profileState.profile.gearList.length > 0}
        <div class="gear-list">
          {#each profileState.profile.gearList as sku, i}
            <div class="gear-item">
              <span class="gear-index">{i + 1}</span>
              <span class="gear-sku">{sku}</span>
            </div>
          {/each}
        </div>
      {:else}
        <div class="empty-state">
          <p>Your gear collection is empty. Add items using the form above.</p>
        </div>
      {/if}
    {:else}
      <div class="loading">Loading...</div>
    {/if}
  </div>
</AuthGuard>

<style>
  .my-gear-page {
    max-width: 600px;
    margin: 0 auto;
    padding: var(--spacing-md);
  }

  .page-header h1 {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--color-on-surface);
    margin: 0 0 var(--spacing-md) 0;
  }

  .add-form {
    display: flex;
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-md);
  }

  .add-form input {
    flex: 1;
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--color-surface-container);
    border: 1px solid var(--color-outline);
    border-radius: var(--radius-md);
    color: var(--color-on-surface);
    font-size: 0.9rem;
  }

  .add-form input:focus {
    outline: none;
    border-color: var(--color-primary);
  }

  .gear-count {
    font-size: 0.85rem;
    color: var(--color-on-surface-muted);
    margin-bottom: var(--spacing-md);
  }

  .gear-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .gear-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--color-surface-container-low);
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-outline-variant);
  }

  .gear-index {
    font-size: 0.75rem;
    color: var(--color-on-surface-muted);
    font-family: var(--font-mono);
    width: 24px;
  }

  .gear-sku {
    font-family: var(--font-mono);
    font-size: 0.9rem;
    color: var(--color-on-surface);
  }

  .empty-state {
    text-align: center;
    padding: var(--spacing-xl);
    color: var(--color-on-surface-muted);
  }

  .loading {
    text-align: center;
    padding: var(--spacing-xl);
    color: var(--color-on-surface-muted);
  }
</style>
