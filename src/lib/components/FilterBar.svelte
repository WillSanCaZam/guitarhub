<script lang="ts">
  import { filterState, updateFilter, clearFilter, clearAllFilters } from '$lib/stores/filter.svelte';
  import type { FilterState } from '$lib/stores/filter.svelte';

  let expanded = $state(false);

  const CONDITION_OPTIONS = ['new', 'used', 'refurbished', 'unknown'];
  const CURRENCY_OPTIONS = ['USD', 'EUR', 'GBP', 'JPY', 'CAD', 'AUD', 'CHF', 'CNY'];
  const SORT_OPTIONS = ['relevance', 'price_asc', 'price_desc', 'name_asc', 'name_desc'] as const;

  const CATEGORY_CHIPS = [
    { label: 'All', value: '' },
    { label: 'Guitar', value: 'Guitar' },
    { label: 'Bass', value: 'Bass' },
    { label: 'Amp', value: 'Amplifier' },
    { label: 'Pedal', value: 'Pedal' },
    { label: 'Keys', value: 'Keyboard' },
  ];

  const activeFilters = $derived.by(() => {
    const filters: { key: keyof FilterState; label: string }[] = [];
    if (filterState.category) filters.push({ key: 'category', label: filterState.category });
    if (filterState.price_min) filters.push({ key: 'price_min', label: `$${filterState.price_min}+` });
    if (filterState.price_max) filters.push({ key: 'price_max', label: `$${filterState.price_max}-` });
    if (filterState.condition) filters.push({ key: 'condition', label: filterState.condition });
    if (filterState.listing_currency) filters.push({ key: 'listing_currency', label: filterState.listing_currency });
    if (filterState.sort !== 'relevance') filters.push({ key: 'sort', label: filterState.sort.replace('_', ' ') });
    return filters;
  });
</script>

<div class="filter-bar">
  <div class="category-chips">
    {#each CATEGORY_CHIPS as chip}
      <button
        class="chip"
        class:active={filterState.category === chip.value}
        onclick={() => updateFilter('category', chip.value || null)}
      >
        {chip.label}
      </button>
    {/each}
  </div>
  {#if activeFilters.length > 0}
    <div class="active-filters">
      {#each activeFilters as filter}
        <span class="filter-pill">
          {filter.label}
          <button class="pill-remove" onclick={() => clearFilter(filter.key)} aria-label={`Remove ${filter.label} filter`}>×</button>
        </span>
      {/each}
    </div>
  {/if}
  <button
    class="filter-toggle"
    onclick={() => (expanded = !expanded)}
    data-testid="filter-toggle"
    aria-expanded={expanded}
  >
    Filters {expanded ? '▲' : '▼'}
  </button>

  {#if expanded}
    <div class="filter-controls">
      <!-- Category -->
      <div class="filter-group">
        <div class="filter-label-row">
          <label for="filter-category">Category</label>
          <button
            class="clear-field-btn"
            data-testid="clear-category"
            onclick={() => clearFilter('category')}
            aria-label="Clear category filter"
          >×</button>
        </div>
        <select
          id="filter-category"
          data-testid="filter-category"
          onchange={(e) => updateFilter('category', (e.target as HTMLSelectElement).value || null)}
        >
          <option value="">All</option>
          <option value="Guitar">Guitar</option>
          <option value="Bass">Bass</option>
          <option value="Amplifier">Amplifier</option>
          <option value="Pedal">Pedal</option>
          <option value="Keyboard">Keyboard</option>
          <option value="Drum">Drum</option>
          <option value="Microphone">Microphone</option>
          <option value="Accessory">Accessory</option>
          <option value="Speaker">Speaker</option>
          <option value="Instrument">Instrument</option>
          <option value="Other">Other</option>
        </select>
      </div>

      <!-- Price Min -->
      <div class="filter-group">
        <div class="filter-label-row">
          <label for="filter-price-min">Min Price</label>
          <button
            class="clear-field-btn"
            data-testid="clear-price-min"
            onclick={() => clearFilter('price_min')}
            aria-label="Clear minimum price filter"
          >×</button>
        </div>
        <div class="price-input-wrap">
          <span class="currency-sign">$</span>
          <input
            id="filter-price-min"
            type="number"
            min="0"
            step="any"
            placeholder="Min"
            data-testid="filter-price-min"
            oninput={(e) => {
              const val = (e.target as HTMLInputElement).value;
              updateFilter('price_min', val ? Number(val) : null);
            }}
          />
        </div>
      </div>

      <!-- Price Max -->
      <div class="filter-group">
        <div class="filter-label-row">
          <label for="filter-price-max">Max Price</label>
          <button
            class="clear-field-btn"
            data-testid="clear-price-max"
            onclick={() => clearFilter('price_max')}
            aria-label="Clear maximum price filter"
          >×</button>
        </div>
        <div class="price-input-wrap">
          <span class="currency-sign">$</span>
          <input
            id="filter-price-max"
            type="number"
            min="0"
            step="any"
            placeholder="Max"
            data-testid="filter-price-max"
            oninput={(e) => {
              const val = (e.target as HTMLInputElement).value;
              updateFilter('price_max', val ? Number(val) : null);
            }}
          />
        </div>
      </div>

      <!-- Condition -->
      <div class="filter-group">
        <div class="filter-label-row">
          <label for="filter-condition">Condition</label>
          <button
            class="clear-field-btn"
            data-testid="clear-condition"
            onclick={() => clearFilter('condition')}
            aria-label="Clear condition filter"
          >×</button>
        </div>
        <select
          id="filter-condition"
          data-testid="filter-condition"
          onchange={(e) => updateFilter('condition', (e.target as HTMLSelectElement).value || null)}
        >
          <option value="">Any</option>
          {#each CONDITION_OPTIONS as opt}
            <option value={opt}>{opt}</option>
          {/each}
        </select>
      </div>

      <!-- Currency -->
      <div class="filter-group">
        <div class="filter-label-row">
          <label for="filter-currency">Currency</label>
          <button
            class="clear-field-btn"
            data-testid="clear-currency"
            onclick={() => clearFilter('listing_currency')}
            aria-label="Clear currency filter"
          >×</button>
        </div>
        <select
          id="filter-currency"
          data-testid="filter-currency"
          onchange={(e) => updateFilter('listing_currency', (e.target as HTMLSelectElement).value || null)}
        >
          <option value="">Any</option>
          {#each CURRENCY_OPTIONS as opt}
            <option value={opt}>{opt}</option>
          {/each}
        </select>
      </div>

      <!-- Sort -->
      <div class="filter-group">
        <div class="filter-label-row">
          <label for="filter-sort">Sort By</label>
          <button
            class="clear-field-btn"
            data-testid="clear-sort"
            onclick={() => clearFilter('sort')}
            aria-label="Clear sort order"
          >×</button>
        </div>
        <select
          id="filter-sort"
          data-testid="filter-sort"
          onchange={(e) => updateFilter('sort', (e.target as HTMLSelectElement).value as FilterState['sort'])}
        >
          {#each SORT_OPTIONS as opt}
            <option value={opt}>{opt.replace('_', ' ')}</option>
          {/each}
        </select>
      </div>

      <!-- Clear All -->
      <div class="filter-group filter-actions">
        <button
          class="clear-all-btn"
          data-testid="filter-clear-all"
          onclick={clearAllFilters}
        >
          Clear All Filters
        </button>
      </div>
    </div>
  {/if}
</div>

<style>
  .filter-bar {
    margin-bottom: 16px;
  }

  .category-chips {
    display: flex;
    gap: var(--spacing-sm);
    overflow-x: auto;
    padding-bottom: var(--spacing-sm);
    margin-bottom: var(--spacing-sm);
  }

  .chip {
    padding: var(--spacing-xs) var(--spacing-md);
    border-radius: var(--radius-pill);
    border: 1px solid var(--color-outline);
    background: transparent;
    color: var(--color-on-surface-variant);
    font-size: 0.85rem;
    cursor: pointer;
    white-space: nowrap;
    transition: background var(--transition-fast), color var(--transition-fast);
  }

  .chip:hover {
    background: var(--color-surface-container);
  }

  .chip.active {
    background: var(--color-primary);
    color: var(--color-on-primary);
    border-color: var(--color-primary);
  }

  .active-filters {
    display: flex;
    flex-wrap: wrap;
    gap: var(--spacing-xs);
    margin-bottom: var(--spacing-sm);
  }

  .filter-pill {
    display: inline-flex;
    align-items: center;
    gap: var(--spacing-xs);
    padding: var(--spacing-2xs) var(--spacing-sm);
    border-radius: var(--radius-pill);
    background: var(--color-surface-container-high);
    color: var(--color-on-surface);
    font-size: 0.75rem;
  }

  .pill-remove {
    background: none;
    border: none;
    color: var(--color-on-surface-muted);
    cursor: pointer;
    padding: 0;
    font-size: 1rem;
    line-height: 1;
  }

  .pill-remove:hover {
    color: var(--color-error);
  }

  .filter-toggle {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 8px 16px;
    background: var(--color-secondary);
    color: var(--color-on-surface);
    border: none;
    border-radius: 6px;
    font-size: 0.9rem;
    cursor: pointer;
  }

  .filter-toggle:hover {
    background: var(--color-secondary);
  }

  .filter-controls {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    margin-top: 12px;
    padding: 16px;
    background: rgba(0, 0, 0, 0.03);
    border-radius: 8px;
    border: 1px solid var(--color-outline);
  }

  .filter-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 140px;
  }

  .filter-label-row {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .filter-group label {
    font-size: 0.8rem;
    color: var(--color-on-surface-muted);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .clear-field-btn {
    background: none;
    border: none;
    color: var(--color-error);
    font-size: 1rem;
    cursor: pointer;
    padding: 0 2px;
    line-height: 1;
    opacity: 0.6;
  }

  .clear-field-btn:hover {
    opacity: 1;
  }

  .filter-group select,
  .filter-group input {
    padding: 8px 10px;
    border: 1px solid var(--color-on-surface-variant);
    border-radius: 6px;
    font-size: 0.9rem;
    background: rgba(255, 255, 255, 0.8);
    box-sizing: border-box;
  }

  .filter-group select:focus,
  .filter-group input:focus {
    outline: none;
    border-color: var(--color-secondary);
    box-shadow: 0 0 0 2px rgba(26, 26, 46, 0.15);
  }

  .price-input-wrap {
    position: relative;
    display: flex;
    align-items: center;
  }

  .currency-sign {
    position: absolute;
    left: 10px;
    color: var(--color-on-surface-muted);
    font-size: 0.85rem;
    pointer-events: none;
  }

  .price-input-wrap input {
    padding-left: 22px;
    width: 100%;
  }

  .filter-actions {
    justify-content: flex-end;
    min-width: auto;
  }

  .clear-all-btn {
    padding: 8px 16px;
    background: transparent;
    color: var(--color-error);
    border: 1px solid var(--color-error);
    border-radius: 6px;
    font-size: 0.85rem;
    cursor: pointer;
    align-self: flex-end;
  }

  .clear-all-btn:hover {
    background: var(--color-error);
    color: var(--color-on-surface);
  }

  @media (prefers-color-scheme: dark) {
    .filter-controls {
      background: rgba(255, 255, 255, 0.05);
      border-color: var(--color-outline-variant);
    }

    .filter-group label {
      color: var(--color-on-surface-variant);
    }

    .filter-group select,
    .filter-group input {
      background: rgba(30, 30, 40, 0.6);
      border-color: var(--color-outline-variant);
      color: var(--color-on-surface);
    }

    .filter-group select:focus,
    .filter-group input:focus {
      border-color: var(--color-on-surface);
      box-shadow: 0 0 0 2px rgba(232, 232, 240, 0.15);
    }

    .currency-sign {
      color: var(--color-on-surface-variant);
    }

    .clear-all-btn {
      color: var(--color-error);
      border-color: var(--color-error);
    }

    .clear-all-btn:hover {
      background: var(--color-error);
      color: var(--color-on-surface);
    }

    .clear-field-btn {
      color: var(--color-error);
    }
  }

  @media (max-width: 768px) {
    .filter-controls {
      flex-direction: column;
    }

    .filter-group {
      min-width: 100%;
    }

    .filter-toggle {
      min-height: 44px;
    }

    .clear-all-btn {
      min-height: 44px;
      width: 100%;
    }
  }
</style>
