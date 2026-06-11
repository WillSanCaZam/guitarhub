import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { writable } from 'svelte/store';
import SearchPanel from '../SearchPanel.svelte';
import { invoke } from '@tauri-apps/api/core';
import type { SortOrder } from '$lib/types/search';
import type { CollectionStore } from '$lib/stores/collection';
import type { FilterState } from '$lib/stores/filter';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

const defaultFilterStore = writable<FilterState>({
  category: null,
  price_min: null,
  price_max: null,
  source: null,
  condition: null,
  listing_currency: null,
  sort: 'relevance' as SortOrder,
});

const mockCollectionStore: CollectionStore = {
  stats: null,
  collectedSkus: new Set<string>(),
  items: [],
  loading: false,
  error: null,
};

describe('SearchPanel', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders search input and button', () => {
    render(SearchPanel, {
      props: { filterStore: defaultFilterStore, collectionStore: mockCollectionStore },
    });
    expect(screen.getByTestId('search-input')).toBeInTheDocument();
    expect(screen.getByTestId('search-button')).toBeInTheDocument();
  });

  it('disables search button when query is too short', () => {
    render(SearchPanel, {
      props: { filterStore: defaultFilterStore, collectionStore: mockCollectionStore },
    });
    const button = screen.getByTestId('search-button');
    expect(button).toBeDisabled();
  });

  it('shows initial empty state before searching', () => {
    render(SearchPanel, {
      props: { filterStore: defaultFilterStore, collectionStore: mockCollectionStore },
    });
    expect(screen.getByText(/Search to find guitar deals/)).toBeInTheDocument();
  });

  it('invokes search_products on Enter key', async () => {
    vi.mocked(invoke).mockResolvedValue({
      products: [],
      total: 0,
      offset: 0,
      limit: 20,
    });

    render(SearchPanel, {
      props: { filterStore: defaultFilterStore, collectionStore: mockCollectionStore },
    });

    const input = screen.getByTestId('search-input');
    await fireEvent.input(input, { target: { value: 'fender strat' } });
    await fireEvent.keyDown(input, { key: 'Enter' });

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('search_products', expect.objectContaining({
        query: 'fender strat',
      }));
    });
  });
});
