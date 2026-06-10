import { describe, it, expect } from 'vitest';
import { filtersToParams, paramsToFilters } from '../filter';
import type { FilterState } from '../filter';

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
