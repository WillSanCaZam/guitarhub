import { describe, it, expect, beforeEach } from 'vitest';
import {
  filterState,
  DEFAULT_FILTERS,
  filtersToParams,
  paramsToFilters,
  updateFilter,
  clearFilter,
  clearAllFilters,
} from '../filter.svelte';
import type { FilterState } from '../filter.svelte';

const FULL_FILTERS: FilterState = {
  category: 'Guitar',
  price_min: 100,
  price_max: 2000,
  source: 'Reverb',
  condition: 'excellent',
  listing_currency: 'USD',
  sort: 'price_asc',
};

const EMPTY_FILTERS: FilterState = {
  category: null,
  price_min: null,
  price_max: null,
  source: null,
  condition: null,
  listing_currency: null,
  sort: 'relevance',
};

describe('filter runes store', () => {
  beforeEach(() => {
    // Reset state to defaults
    Object.assign(filterState, { ...DEFAULT_FILTERS });
  });

  describe('initial state', () => {
    it('has correct default shape', () => {
      expect(filterState.category).toBeNull();
      expect(filterState.price_min).toBeNull();
      expect(filterState.price_max).toBeNull();
      expect(filterState.source).toBeNull();
      expect(filterState.condition).toBeNull();
      expect(filterState.listing_currency).toBeNull();
      expect(filterState.sort).toBe('relevance');
    });
  });

  describe('updateFilter', () => {
    it('updates a single field', () => {
      updateFilter('category', 'Guitar');
      expect(filterState.category).toBe('Guitar');
    });

    it('updates sort field', () => {
      updateFilter('sort', 'price_desc');
      expect(filterState.sort).toBe('price_desc');
    });

    it('updates price_min to a number', () => {
      updateFilter('price_min', 150);
      expect(filterState.price_min).toBe(150);
    });

    it('sets price_min to null (clearing)', () => {
      filterState.price_min = 100;
      updateFilter('price_min', null);
      expect(filterState.price_min).toBeNull();
    });
  });

  describe('clearFilter', () => {
    it('resets a string field to null', () => {
      filterState.category = 'Guitar';
      clearFilter('category');
      expect(filterState.category).toBeNull();
    });

    it('resets sort to relevance', () => {
      filterState.sort = 'price_asc';
      clearFilter('sort');
      expect(filterState.sort).toBe('relevance');
    });

    it('resets numeric field to null', () => {
      filterState.price_min = 100;
      filterState.price_max = 500;
      clearFilter('price_min');
      expect(filterState.price_min).toBeNull();
      expect(filterState.price_max).toBe(500);
    });
  });

  describe('clearAllFilters', () => {
    it('resets all fields to defaults', () => {
      Object.assign(filterState, FULL_FILTERS);
      clearAllFilters();
      expect(filterState).toEqual(EMPTY_FILTERS);
    });
  });

  describe('filtersToParams', () => {
    it('returns empty URLSearchParams when all filters are null/default', () => {
      const params = filtersToParams(EMPTY_FILTERS);
      expect(params.toString()).toBe('');
    });

    it('serialises a single string filter to a URL param', () => {
      const params = filtersToParams({ ...EMPTY_FILTERS, category: 'Guitar' });
      expect(params.get('category')).toBe('Guitar');
      expect([...params]).toHaveLength(1);
    });

    it('serialises numeric filters as strings', () => {
      const params = filtersToParams({ ...EMPTY_FILTERS, price_min: 150, price_max: 3000 });
      expect(params.get('price_min')).toBe('150');
      expect(params.get('price_max')).toBe('3000');
    });

    it('serialises sort as a URL param (non-null)', () => {
      const params = filtersToParams(FULL_FILTERS);
      expect(params.get('sort')).toBe('price_asc');
    });

    it('serialises all non-null fields from a full filter set', () => {
      const params = filtersToParams(FULL_FILTERS);
      expect(params.get('category')).toBe('Guitar');
      expect(params.get('price_min')).toBe('100');
      expect(params.get('price_max')).toBe('2000');
      expect(params.get('source')).toBe('Reverb');
      expect(params.get('condition')).toBe('excellent');
      expect(params.get('listing_currency')).toBe('USD');
      expect(params.get('sort')).toBe('price_asc');
      expect([...params]).toHaveLength(7);
    });

    it('omits null fields from the output', () => {
      const params = filtersToParams({ ...FULL_FILTERS, category: null, price_min: null });
      expect(params.get('category')).toBeNull();
      expect(params.get('price_min')).toBeNull();
      expect(params.get('sort')).toBe('price_asc');
    });
  });

  describe('paramsToFilters', () => {
    it('returns default filters from empty params', () => {
      const params = new URLSearchParams();
      const filters = paramsToFilters(params);
      expect(filters).toEqual(EMPTY_FILTERS);
    });

    it('restores a single string param', () => {
      const params = new URLSearchParams('category=Guitar');
      const filters = paramsToFilters(params);
      expect(filters.category).toBe('Guitar');
      expect(filters.price_min).toBeNull();
    });

    it('restores numeric params as numbers', () => {
      const params = new URLSearchParams('price_min=150&price_max=3000');
      const filters = paramsToFilters(params);
      expect(filters.price_min).toBe(150);
      expect(filters.price_max).toBe(3000);
    });

    it('restores full set of params', () => {
      const params = new URLSearchParams(
        'category=Guitar&price_min=100&price_max=2000&source=Reverb&condition=excellent&listing_currency=USD&sort=price_asc'
      );
      expect(paramsToFilters(params)).toEqual(FULL_FILTERS);
    });

    it('defaults sort to "relevance" when params are empty', () => {
      const filters = paramsToFilters(new URLSearchParams('category=Guitar'));
      expect(filters.sort).toBe('relevance');
    });

    it('parses sort from URL when present', () => {
      const filters = paramsToFilters(new URLSearchParams('sort=price_desc'));
      expect(filters.sort).toBe('price_desc');
    });

    it('ignores unknown params', () => {
      const filters = paramsToFilters(new URLSearchParams('unknown=foo&category=Pedals'));
      expect(filters.category).toBe('Pedals');
      expect(filters).not.toHaveProperty('unknown');
    });
  });

  describe('round-trip identity', () => {
    it('filterState → params → filterState is identity for full state', () => {
      const params = filtersToParams(FULL_FILTERS);
      const restored = paramsToFilters(params);
      expect(restored).toEqual(FULL_FILTERS);
    });

    it('filterState → params → filterState is identity for empty state', () => {
      const params = filtersToParams(EMPTY_FILTERS);
      const restored = paramsToFilters(params);
      expect(restored).toEqual(EMPTY_FILTERS);
    });

    it('filterState → params → filterState is identity for partial state', () => {
      const partial: FilterState = {
        category: 'Amps',
        price_min: null,
        price_max: null,
        source: 'eBay',
        condition: null,
        listing_currency: null,
        sort: 'price_asc',
      };
      const params = filtersToParams(partial);
      const restored = paramsToFilters(params);
      expect(restored).toEqual(partial);
    });
  });
});
