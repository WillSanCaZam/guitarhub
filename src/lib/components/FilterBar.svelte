<script lang="ts">
  import { filterState, updateFilter, clearFilter, clearAllFilters } from '$lib/stores/filter.svelte';
  import type { FilterState } from '$lib/stores/filter.svelte';
  import PriceRangeSlider from './ui/PriceRangeSlider.svelte';

  const CONDITION_OPTIONS = ['new', 'used', 'refurbished', 'unknown'];
  const SORT_OPTIONS = ['relevance', 'price_asc', 'price_desc', 'rating', 'newest'] as const;

  const CATEGORY_CHIPS = [
    { label: 'All', value: '', icon: '🎵' },
    { label: 'Guitar', value: 'Guitar', icon: '🎸' },
    { label: 'Bass', value: 'Bass', icon: '🎸' },
    { label: 'Amp', value: 'Amplifier', icon: '🔊' },
    { label: 'Pedal', value: 'Pedal', icon: '🎛️' },
    { label: 'Keys', value: 'Keyboard', icon: '🎹' },
    { label: 'Drums', value: 'Drum', icon: '🥁' },
    { label: 'Studio', value: 'Microphone', icon: '🎤' },
    { label: 'Accessories', value: 'Accessory', icon: '🔧' },
  ];

  const activeFilters = $derived.by(() => {
    const filters: { key: keyof FilterState; label: string }[] = [];
    if (filterState.category) filters.push({ key: 'category', label: filterState.category });
    if (filterState.price_min) filters.push({ key: 'price_min', label: `$${filterState.price_min}+` });
    if (filterState.price_max) filters.push({ key: 'price_max', label: `$${filterState.price_max}-` });
    if (filterState.condition) filters.push({ key: 'condition', label: filterState.condition });
    if (filterState.sort !== 'relevance') filters.push({ key: 'sort', label: filterState.sort.replace('_', ' ') });
    return filters;
  });
</script>

<div class="filter-bar">
  <!-- Category Pills with Icons -->
  <div class="category-chips">
    {#each CATEGORY_CHIPS as chip}
      <button
        class="chip"
        class:active={filterState.category === chip.value}
        onclick={() => updateFilter('category', chip.value || null)}
      >
        <span class="chip-icon">{chip.icon}</span>
        <span class="chip-label">{chip.label}</span>
      </button>
    {/each}
  </div>

  <!-- Active Filter Pills -->
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

  <!-- Filter Controls -->
  <div class="filter-controls">
    <!-- Price Range Slider -->
    <div class="filter-group price-range-group">
      <label class="filter-label" for="price-range-slider">Price Range</label>
      <div id="price-range-slider">
        <PriceRangeSlider
        min={0}
        max={10000}
        step={50}
        value={[filterState.price_min ?? 0, filterState.price_max ?? 10000]}
        onchange={(val: [number, number]) => {
          updateFilter('price_min', val[0] > 0 ? val[0] : null);
          updateFilter('price_max', val[1] < 10000 ? val[1] : null);
        }}
      />
      </div>
    </div>

    <!-- Condition -->
    <div class="filter-group">
      <div class="filter-label-row">
        <label for="filter-condition" class="filter-label">Condition</label>
        <button
          class="clear-field-btn"
          data-testid="clear-condition"
          onclick={() => clearFilter('condition')}
          aria-label="Clear condition filter"
        >×</button>
      </div>
      <div class="checkbox-group">
        {#each CONDITION_OPTIONS as opt}
          <label class="checkbox-label">
            <input
              type="checkbox"
              checked={filterState.condition === opt}
              onchange={() => updateFilter('condition', filterState.condition === opt ? null : opt)}
            />
            <span class="checkbox-text">{opt}</span>
          </label>
        {/each}
      </div>
    </div>

    <!-- Sort -->
    <div class="filter-group">
      <div class="filter-label-row">
        <label for="filter-sort" class="filter-label">Sort By</label>
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
        class="filter-select"
        onchange={(e) => updateFilter('sort', (e.target as HTMLSelectElement).value as FilterState['sort'])}
      >
        {#each SORT_OPTIONS as opt}
          <option value={opt}>{opt.replace('_', ' ')}</option>
        {/each}
      </select>
    </div>

    <!-- Clear All -->
    {#if activeFilters.length > 0}
      <div class="filter-group filter-actions">
        <button
          class="clear-all-btn"
          data-testid="filter-clear-all"
          onclick={clearAllFilters}
        >
          Clear All
        </button>
      </div>
    {/if}
  </div>
</div>

<style>
  .filter-bar {
    margin-bottom: var(--space-4);
  }

  .category-chips {
    display: flex;
    gap: var(--space-2);
    overflow-x: auto;
    padding-bottom: var(--space-2);
    margin-bottom: var(--space-3);
    scrollbar-width: none;
  }

  .category-chips::-webkit-scrollbar {
    display: none;
  }

  .chip {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-4);
    height: 44px;
    border-radius: var(--radius-pill);
    background: var(--void-mid);
    color: var(--text-warm);
    border: 1px solid rgba(255, 255, 255, 0.06);
    font-size: 0.85rem;
    font-weight: 500;
    cursor: pointer;
    white-space: nowrap;
    flex-shrink: 0;
    transition: background 150ms var(--ease-snap), border-color 150ms var(--ease-snap), color 150ms var(--ease-snap);
  }

  .chip:hover {
    background: var(--void-hover);
    border-color: rgba(255, 122, 61, 0.15);
  }

  .chip.active {
    background: var(--glow-primary);
    color: var(--void-deep);
    border-color: var(--glow-primary);
  }

  .chip-icon {
    font-size: 1rem;
  }

  /* Active Filters */
  .active-filters {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    margin-bottom: var(--space-3);
  }

  .filter-pill {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    padding: var(--space-1) var(--space-3);
    border-radius: var(--radius-pill);
    background: var(--void-hover);
    color: var(--text-bright);
    font-size: 0.75rem;
    font-weight: 500;
  }

  .pill-remove {
    background: none;
    border: none;
    color: var(--text-dim);
    cursor: pointer;
    padding: 0;
    font-size: 1rem;
    line-height: 1;
    transition: color 150ms var(--ease-snap);
  }

  .pill-remove:hover {
    color: var(--danger);
  }

  /* Filter Controls */
  .filter-controls {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-3);
    padding: var(--space-4);
    background: var(--void-raised);
    border-radius: var(--radius-md);
    border: 1px solid rgba(255, 122, 61, 0.06);
  }

  .filter-group {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    min-width: 160px;
  }

  .price-range-group {
    min-width: 240px;
  }

  .filter-label-row {
    display: flex;
    align-items: center;
    gap: var(--space-1);
  }

  .filter-label {
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-dim);
  }

  .clear-field-btn {
    background: none;
    border: none;
    color: var(--text-muted);
    font-size: 1rem;
    cursor: pointer;
    padding: 0 2px;
    line-height: 1;
    transition: color 150ms var(--ease-snap);
  }

  .clear-field-btn:hover {
    color: var(--danger);
  }

  .checkbox-group {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    cursor: pointer;
    font-size: 0.85rem;
    color: var(--text-warm);
  }

  .checkbox-label input[type="checkbox"] {
    accent-color: var(--glow-primary);
    width: 16px;
    height: 16px;
  }

  .filter-select {
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--text-muted);
    border-radius: var(--radius-sm);
    background: var(--void-mid);
    color: var(--text-bright);
    font-size: 0.85rem;
    cursor: pointer;
    transition: border-color 150ms var(--ease-snap);
  }

  .filter-select:focus {
    outline: none;
    border-color: var(--glow-primary);
    box-shadow: 0 0 0 2px var(--glow-soft);
  }

  .filter-actions {
    justify-content: flex-end;
    min-width: auto;
  }

  .clear-all-btn {
    padding: var(--space-2) var(--space-4);
    background: transparent;
    color: var(--danger);
    border: 1px solid var(--danger);
    border-radius: var(--radius-sm);
    font-size: 0.8rem;
    font-weight: 600;
    cursor: pointer;
    transition: background 150ms var(--ease-snap), color 150ms var(--ease-snap);
  }

  .clear-all-btn:hover {
    background: var(--danger);
    color: var(--text-bright);
  }

  @media (max-width: 768px) {
    .filter-controls {
      flex-direction: column;
    }

    .filter-group {
      min-width: 100%;
    }
  }
</style>
