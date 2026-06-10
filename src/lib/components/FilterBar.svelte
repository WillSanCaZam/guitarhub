<script lang="ts">
  import { filterStore, syncFiltersToUrl, DEFAULT_FILTERS } from '$lib/stores/filter';
  import type { FilterState } from '$lib/stores/filter';

  let expanded = $state(false);

  const CONDITION_OPTIONS = ['new', 'used', 'refurbished', 'unknown'];
  const CURRENCY_OPTIONS = ['USD', 'EUR', 'GBP', 'JPY', 'CAD', 'AUD', 'CHF', 'CNY'];
  const SORT_OPTIONS = ['relevance', 'price_asc', 'price_desc', 'name_asc', 'name_desc'] as const;

  function updateField<K extends keyof FilterState>(field: K, value: FilterState[K]): void {
    filterStore.update((state) => {
      const next = { ...state, [field]: value };
      syncFiltersToUrl(next);
      return next;
    });
  }

  function clearField(field: keyof FilterState): void {
    filterStore.update((state) => {
      const next = { ...state, [field]: field === 'sort' ? 'relevance' as const : null };
      syncFiltersToUrl(next);
      return next;
    });
  }

  function handleClearAll(): void {
    const defaults = { ...DEFAULT_FILTERS };
    filterStore.set(defaults);
    syncFiltersToUrl(defaults);
  }
</script>

<div class="filter-bar">
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
            onclick={() => clearField('category')}
            aria-label="Clear category filter"
          >×</button>
        </div>
        <select
          id="filter-category"
          data-testid="filter-category"
          onchange={(e) => updateField('category', (e.target as HTMLSelectElement).value || null)}
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
            onclick={() => clearField('price_min')}
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
              updateField('price_min', val ? Number(val) : null);
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
            onclick={() => clearField('price_max')}
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
              updateField('price_max', val ? Number(val) : null);
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
            onclick={() => clearField('condition')}
            aria-label="Clear condition filter"
          >×</button>
        </div>
        <select
          id="filter-condition"
          data-testid="filter-condition"
          onchange={(e) => updateField('condition', (e.target as HTMLSelectElement).value || null)}
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
            onclick={() => clearField('listing_currency')}
            aria-label="Clear currency filter"
          >×</button>
        </div>
        <select
          id="filter-currency"
          data-testid="filter-currency"
          onchange={(e) => updateField('listing_currency', (e.target as HTMLSelectElement).value || null)}
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
            onclick={() => clearField('sort')}
            aria-label="Clear sort order"
          >×</button>
        </div>
        <select
          id="filter-sort"
          data-testid="filter-sort"
          onchange={(e) => updateField('sort', (e.target as HTMLSelectElement).value as FilterState['sort'])}
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
          onclick={handleClearAll}
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

  .filter-toggle {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 8px 16px;
    background: #1a1a2e;
    color: #fff;
    border: none;
    border-radius: 6px;
    font-size: 0.9rem;
    cursor: pointer;
  }

  .filter-toggle:hover {
    background: #2a2a4e;
  }

  .filter-controls {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    margin-top: 12px;
    padding: 16px;
    background: rgba(0, 0, 0, 0.03);
    border-radius: 8px;
    border: 1px solid #e0e0e0;
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
    color: #666;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .clear-field-btn {
    background: none;
    border: none;
    color: #c0392b;
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
    border: 1px solid #ccc;
    border-radius: 6px;
    font-size: 0.9rem;
    background: rgba(255, 255, 255, 0.8);
    box-sizing: border-box;
  }

  .filter-group select:focus,
  .filter-group input:focus {
    outline: none;
    border-color: #1a1a2e;
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
    color: #888;
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
    color: #c0392b;
    border: 1px solid #c0392b;
    border-radius: 6px;
    font-size: 0.85rem;
    cursor: pointer;
    align-self: flex-end;
  }

  .clear-all-btn:hover {
    background: #c0392b;
    color: #fff;
  }

  @media (prefers-color-scheme: dark) {
    .filter-controls {
      background: rgba(255, 255, 255, 0.05);
      border-color: #444;
    }

    .filter-group label {
      color: #aaa;
    }

    .filter-group select,
    .filter-group input {
      background: rgba(30, 30, 40, 0.6);
      border-color: #444;
      color: #e8e8f0;
    }

    .filter-group select:focus,
    .filter-group input:focus {
      border-color: #e8e8f0;
      box-shadow: 0 0 0 2px rgba(232, 232, 240, 0.15);
    }

    .currency-sign {
      color: #aaa;
    }

    .clear-all-btn {
      color: #e74c3c;
      border-color: #e74c3c;
    }

    .clear-all-btn:hover {
      background: #e74c3c;
      color: #fff;
    }

    .clear-field-btn {
      color: #e74c3c;
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
