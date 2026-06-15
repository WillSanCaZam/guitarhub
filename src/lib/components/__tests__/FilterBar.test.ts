import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import FilterBar from '../FilterBar.svelte';
import { filterState, DEFAULT_FILTERS } from '$lib/stores/filter.svelte';

describe('FilterBar', () => {
  beforeEach(() => {
    Object.assign(filterState, { ...DEFAULT_FILTERS });
  });

  it('renders filter controls always visible (no toggle)', () => {
    render(FilterBar);
    // Controls should be visible immediately — no expand needed
    expect(screen.getByTestId('filter-category')).toBeInTheDocument();
    expect(screen.getByTestId('filter-price-min')).toBeInTheDocument();
    expect(screen.getByTestId('filter-price-max')).toBeInTheDocument();
    expect(screen.getByTestId('filter-condition')).toBeInTheDocument();
    expect(screen.getByTestId('filter-currency')).toBeInTheDocument();
    expect(screen.getByTestId('filter-sort')).toBeInTheDocument();
    expect(screen.getByTestId('filter-clear-all')).toBeInTheDocument();
    // Toggle button should not exist
    expect(screen.queryByTestId('filter-toggle')).not.toBeInTheDocument();
  });

  it('updates state when category is changed', async () => {
    render(FilterBar);

    const categorySelect = screen.getByTestId('filter-category') as HTMLSelectElement;
    await fireEvent.change(categorySelect, { target: { value: 'Guitar' } });

    expect(filterState.category).toBe('Guitar');
  });

  it('updates state when price min is changed', async () => {
    render(FilterBar);

    const priceMin = screen.getByTestId('filter-price-min') as HTMLInputElement;
    await fireEvent.input(priceMin, { target: { value: '100' } });

    expect(filterState.price_min).toBe(100);
  });

  it('updates state when price max is changed', async () => {
    render(FilterBar);

    const priceMax = screen.getByTestId('filter-price-max') as HTMLInputElement;
    await fireEvent.input(priceMax, { target: { value: '2000' } });

    expect(filterState.price_max).toBe(2000);
  });

  it('updates state when condition is changed', async () => {
    render(FilterBar);

    const conditionSelect = screen.getByTestId('filter-condition') as HTMLSelectElement;
    await fireEvent.change(conditionSelect, { target: { value: 'new' } });

    expect(filterState.condition).toBe('new');
  });

  it('updates state when currency is changed', async () => {
    render(FilterBar);

    const currencySelect = screen.getByTestId('filter-currency') as HTMLSelectElement;
    await fireEvent.change(currencySelect, { target: { value: 'USD' } });

    expect(filterState.listing_currency).toBe('USD');
  });

  it('updates state when sort is changed', async () => {
    render(FilterBar);

    const sortSelect = screen.getByTestId('filter-sort') as HTMLSelectElement;
    await fireEvent.change(sortSelect, { target: { value: 'price_asc' } });

    expect(filterState.sort).toBe('price_asc');
  });

  it('clears all filters when "Clear All" is clicked', async () => {
    // Set some filters first
    filterState.category = 'Guitar';
    filterState.price_min = 100;
    filterState.price_max = 2000;
    filterState.source = null;
    filterState.condition = 'new';
    filterState.listing_currency = 'USD';
    filterState.sort = 'price_asc';

    render(FilterBar);
    await fireEvent.click(screen.getByTestId('filter-clear-all'));

    expect(filterState).toEqual(DEFAULT_FILTERS);
  });

  it('individual clear button resets category to null', async () => {
    filterState.category = 'Guitar';
    filterState.condition = 'new';

    render(FilterBar);
    await fireEvent.click(screen.getByTestId('clear-category'));

    expect(filterState.category).toBeNull();
    expect(filterState.condition).toBe('new'); // other fields untouched
  });

  it('individual clear button resets price_min to null', async () => {
    filterState.price_min = 100;
    filterState.price_max = 2000;

    render(FilterBar);
    await fireEvent.click(screen.getByTestId('clear-price-min'));

    expect(filterState.price_min).toBeNull();
    expect(filterState.price_max).toBe(2000); // other fields untouched
  });

  it('individual clear button resets condition to null', async () => {
    filterState.condition = 'used';

    render(FilterBar);
    await fireEvent.click(screen.getByTestId('clear-condition'));

    expect(filterState.condition).toBeNull();
  });

  it('individual clear button resets sort to relevance', async () => {
    filterState.sort = 'price_desc';

    render(FilterBar);
    await fireEvent.click(screen.getByTestId('clear-sort'));

    // sort defaults back to 'relevance', not null
    expect(filterState.sort).toBe('relevance');
  });

  it('all individual clear buttons are rendered', () => {
    render(FilterBar);

    expect(screen.getByTestId('clear-category')).toBeInTheDocument();
    expect(screen.getByTestId('clear-price-min')).toBeInTheDocument();
    expect(screen.getByTestId('clear-price-max')).toBeInTheDocument();
    expect(screen.getByTestId('clear-condition')).toBeInTheDocument();
    expect(screen.getByTestId('clear-currency')).toBeInTheDocument();
    expect(screen.getByTestId('clear-sort')).toBeInTheDocument();
  });

  it('category select starts with "All" option selected', () => {
    render(FilterBar);

    const categorySelect = screen.getByTestId('filter-category') as HTMLSelectElement;
    expect(categorySelect.value).toBe('');
  });

  it('select controls have the expected options', () => {
    render(FilterBar);

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
