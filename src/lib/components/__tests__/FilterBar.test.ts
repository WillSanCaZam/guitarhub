import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import FilterBar from '../FilterBar.svelte';
import { filterState, DEFAULT_FILTERS } from '$lib/stores/filter.svelte';

describe('FilterBar', () => {
  beforeEach(() => {
    Object.assign(filterState, { ...DEFAULT_FILTERS });
  });

  it('renders category chips', () => {
    render(FilterBar);
    expect(screen.getByText('All')).toBeInTheDocument();
    expect(screen.getByText('Guitar')).toBeInTheDocument();
    expect(screen.getByText('Bass')).toBeInTheDocument();
    expect(screen.getByText('Amp')).toBeInTheDocument();
  });

  it('renders sort select', () => {
    render(FilterBar);
    expect(screen.getByTestId('filter-sort')).toBeInTheDocument();
  });

  it('renders clear all button when filters are active', () => {
    filterState.category = 'Guitar';
    render(FilterBar);
    expect(screen.getByTestId('filter-clear-all')).toBeInTheDocument();
  });

  it('does not render clear all button when no filters are active', () => {
    render(FilterBar);
    expect(screen.queryByTestId('filter-clear-all')).not.toBeInTheDocument();
  });

  it('updates state when category chip is clicked', async () => {
    render(FilterBar);
    const guitarChip = screen.getByText('Guitar');
    await fireEvent.click(guitarChip);
    expect(filterState.category).toBe('Guitar');
  });

  it('re-selecting same category keeps it active', async () => {
    filterState.category = 'Guitar';
    render(FilterBar);
    // Find the active chip specifically (has the .active class)
    const guitarChip = screen.getAllByText('Guitar')[0].closest('button');
    expect(guitarChip).toHaveClass('active');
    await fireEvent.click(guitarChip!);
    // Component always sets category to chip value (no toggle), so it stays 'Guitar'
    expect(filterState.category).toBe('Guitar');
  });

  it('updates state when sort is changed', async () => {
    render(FilterBar);
    const sortSelect = screen.getByTestId('filter-sort') as HTMLSelectElement;
    await fireEvent.change(sortSelect, { target: { value: 'price_asc' } });
    expect(filterState.sort).toBe('price_asc');
  });

  it('clears all filters when "Clear All" is clicked', async () => {
    filterState.category = 'Guitar';
    filterState.condition = 'new';
    filterState.sort = 'price_asc';

    render(FilterBar);
    await fireEvent.click(screen.getByTestId('filter-clear-all'));

    expect(filterState).toEqual(DEFAULT_FILTERS);
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

    expect(filterState.sort).toBe('relevance');
  });

  it('sort select has expected options', () => {
    render(FilterBar);
    const sortSelect = screen.getByTestId('filter-sort') as HTMLSelectElement;
    const sortOptions = Array.from(sortSelect.options).map((o) => o.value);
    expect(sortOptions).toEqual(['relevance', 'price_asc', 'price_desc', 'rating', 'newest']);
  });
});
