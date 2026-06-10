import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import FilterBar from '../FilterBar.svelte';
import { filterStore, DEFAULT_FILTERS } from '$lib/stores/filter';
import { get } from 'svelte/store';
import type { FilterState } from '$lib/stores/filter';

describe('FilterBar', () => {
  beforeEach(() => {
    filterStore.set({ ...DEFAULT_FILTERS });
  });

  it('renders toggle button collapsed by default', () => {
    render(FilterBar);
    const toggle = screen.getByTestId('filter-toggle');
    expect(toggle).toBeInTheDocument();
    expect(toggle).toHaveTextContent(/Filters/);
    // Controls should NOT be visible when collapsed
    expect(screen.queryByTestId('filter-category')).not.toBeInTheDocument();
  });

  it('shows filter controls when expanded', async () => {
    render(FilterBar);
    const toggle = screen.getByTestId('filter-toggle');
    await fireEvent.click(toggle);

    // All controls should be visible
    expect(screen.getByTestId('filter-category')).toBeInTheDocument();
    expect(screen.getByTestId('filter-price-min')).toBeInTheDocument();
    expect(screen.getByTestId('filter-price-max')).toBeInTheDocument();
    expect(screen.getByTestId('filter-condition')).toBeInTheDocument();
    expect(screen.getByTestId('filter-currency')).toBeInTheDocument();
    expect(screen.getByTestId('filter-sort')).toBeInTheDocument();
    expect(screen.getByTestId('filter-clear-all')).toBeInTheDocument();
  });

  it('hides controls when toggled again', async () => {
    render(FilterBar);
    const toggle = screen.getByTestId('filter-toggle');

    await fireEvent.click(toggle);
    expect(screen.getByTestId('filter-category')).toBeInTheDocument();

    await fireEvent.click(toggle);
    expect(screen.queryByTestId('filter-category')).not.toBeInTheDocument();
  });

  it('updates store when category is changed', async () => {
    render(FilterBar);
    await fireEvent.click(screen.getByTestId('filter-toggle'));

    const categorySelect = screen.getByTestId('filter-category') as HTMLSelectElement;
    await fireEvent.change(categorySelect, { target: { value: 'Guitar' } });

    const state = get(filterStore);
    expect(state.category).toBe('Guitar');
  });

  it('updates store when price min is changed', async () => {
    render(FilterBar);
    await fireEvent.click(screen.getByTestId('filter-toggle'));

    const priceMin = screen.getByTestId('filter-price-min') as HTMLInputElement;
    await fireEvent.input(priceMin, { target: { value: '100' } });

    const state = get(filterStore);
    expect(state.price_min).toBe(100);
  });

  it('updates store when price max is changed', async () => {
    render(FilterBar);
    await fireEvent.click(screen.getByTestId('filter-toggle'));

    const priceMax = screen.getByTestId('filter-price-max') as HTMLInputElement;
    await fireEvent.input(priceMax, { target: { value: '2000' } });

    const state = get(filterStore);
    expect(state.price_max).toBe(2000);
  });

  it('updates store when condition is changed', async () => {
    render(FilterBar);
    await fireEvent.click(screen.getByTestId('filter-toggle'));

    const conditionSelect = screen.getByTestId('filter-condition') as HTMLSelectElement;
    await fireEvent.change(conditionSelect, { target: { value: 'new' } });

    const state = get(filterStore);
    expect(state.condition).toBe('new');
  });

  it('updates store when currency is changed', async () => {
    render(FilterBar);
    await fireEvent.click(screen.getByTestId('filter-toggle'));

    const currencySelect = screen.getByTestId('filter-currency') as HTMLSelectElement;
    await fireEvent.change(currencySelect, { target: { value: 'USD' } });

    const state = get(filterStore);
    expect(state.listing_currency).toBe('USD');
  });

  it('updates store when sort is changed', async () => {
    render(FilterBar);
    await fireEvent.click(screen.getByTestId('filter-toggle'));

    const sortSelect = screen.getByTestId('filter-sort') as HTMLSelectElement;
    await fireEvent.change(sortSelect, { target: { value: 'price_asc' } });

    const state = get(filterStore);
    expect(state.sort).toBe('price_asc');
  });

  it('clears all filters when "Clear All" is clicked', async () => {
    // Set some filters first
    filterStore.set({
      category: 'Guitar',
      price_min: 100,
      price_max: 2000,
      source: null,
      condition: 'new',
      listing_currency: 'USD',
      sort: 'price_asc',
    });

    render(FilterBar);
    await fireEvent.click(screen.getByTestId('filter-toggle'));
    await fireEvent.click(screen.getByTestId('filter-clear-all'));

    const state = get(filterStore);
    expect(state).toEqual(DEFAULT_FILTERS);
  });

  it('individual clear button resets category to null', async () => {
    filterStore.set({
      category: 'Guitar',
      price_min: null,
      price_max: null,
      source: null,
      condition: 'new',
      listing_currency: null,
      sort: 'relevance',
    });

    render(FilterBar);
    await fireEvent.click(screen.getByTestId('filter-toggle'));
    await fireEvent.click(screen.getByTestId('clear-category'));

    const state = get(filterStore);
    expect(state.category).toBeNull();
    expect(state.condition).toBe('new'); // other fields untouched
  });

  it('individual clear button resets price_min to null', async () => {
    filterStore.set({
      category: null,
      price_min: 100,
      price_max: 2000,
      source: null,
      condition: null,
      listing_currency: null,
      sort: 'relevance',
    });

    render(FilterBar);
    await fireEvent.click(screen.getByTestId('filter-toggle'));
    await fireEvent.click(screen.getByTestId('clear-price-min'));

    const state = get(filterStore);
    expect(state.price_min).toBeNull();
    expect(state.price_max).toBe(2000); // other fields untouched
  });

  it('individual clear button resets condition to null', async () => {
    filterStore.set({
      category: null,
      price_min: null,
      price_max: null,
      source: null,
      condition: 'used',
      listing_currency: null,
      sort: 'relevance',
    });

    render(FilterBar);
    await fireEvent.click(screen.getByTestId('filter-toggle'));
    await fireEvent.click(screen.getByTestId('clear-condition'));

    const state = get(filterStore);
    expect(state.condition).toBeNull();
  });

  it('individual clear button resets sort to relevance', async () => {
    filterStore.set({
      category: null,
      price_min: null,
      price_max: null,
      source: null,
      condition: null,
      listing_currency: null,
      sort: 'price_desc',
    });

    render(FilterBar);
    await fireEvent.click(screen.getByTestId('filter-toggle'));
    await fireEvent.click(screen.getByTestId('clear-sort'));

    const state = get(filterStore);
    // sort defaults back to 'relevance', not null
    expect(state.sort).toBe('relevance');
  });

  it('all individual clear buttons are rendered when expanded', async () => {
    render(FilterBar);
    await fireEvent.click(screen.getByTestId('filter-toggle'));

    expect(screen.getByTestId('clear-category')).toBeInTheDocument();
    expect(screen.getByTestId('clear-price-min')).toBeInTheDocument();
    expect(screen.getByTestId('clear-price-max')).toBeInTheDocument();
    expect(screen.getByTestId('clear-condition')).toBeInTheDocument();
    expect(screen.getByTestId('clear-currency')).toBeInTheDocument();
    expect(screen.getByTestId('clear-sort')).toBeInTheDocument();
  });

  it('category select starts with "All" option selected', async () => {
    render(FilterBar);
    await fireEvent.click(screen.getByTestId('filter-toggle'));

    const categorySelect = screen.getByTestId('filter-category') as HTMLSelectElement;
    expect(categorySelect.value).toBe('');
  });

  it('select controls have the expected options', async () => {
    render(FilterBar);
    await fireEvent.click(screen.getByTestId('filter-toggle'));

    // Condition options
    const conditionSelect = screen.getByTestId('filter-condition') as HTMLSelectElement;
    const conditionOptions = Array.from(conditionSelect.options).map((o) => o.value);
    expect(conditionOptions).toEqual(['', 'new', 'used', 'refurbished', 'unknown']);

    // Sort options
    const sortSelect = screen.getByTestId('filter-sort') as HTMLSelectElement;
    const sortOptions = Array.from(sortSelect.options).map((o) => o.value);
    expect(sortOptions).toEqual(['relevance', 'price_asc', 'price_desc', 'name_asc', 'name_desc']);

    // Currency should include USD at minimum
    const currencySelect = screen.getByTestId('filter-currency') as HTMLSelectElement;
    const currencyOptions = Array.from(currencySelect.options).map((o) => o.value);
    expect(currencyOptions).toContain('USD');
  });
});
