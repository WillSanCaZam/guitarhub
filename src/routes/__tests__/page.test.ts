import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/svelte';
import Page from '../+page.svelte';
import { invoke } from '@tauri-apps/api/core';
import { dashboardStats } from '$lib/stores/dashboard';
import { syncResult } from '$lib/stores/sync';
import { collectionStore } from '$lib/stores/collection';
import { filterStore } from '$lib/stores/filter';

// Mock @tanstack/svelte-virtual to prevent infinite $effect loop in jsdom.
// In jsdom, DOM measurements return 0 causing Virtualizer.setOptions to
// trigger store writes that re-enter the $effect indefinitely.
vi.mock('@tanstack/svelte-virtual', () => ({
  createVirtualizer: () => ({
    subscribe: (fn: (value: unknown) => void) => {
      fn({
        setOptions: () => {}, // no-op breaks the infinite loop
        getTotalSize: () => 0,
        getVirtualItems: () => [],
      });
      return () => {}; // unsubscribe
    },
  }),
}));

describe('Dashboard Page', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    dashboardStats.set({ totalProducts: 0, wishlistCount: 0, recentSearches: [], loading: false, error: null });
    syncResult.set({ drops: [], drops_sent: 0, state: 'idle' });
    collectionStore.set({ items: [], stats: null, collectedSkus: new Set(), loading: false, error: null });
    filterStore.set({ category: null, price_min: null, price_max: null, source: null, condition: null, listing_currency: null, sort: 'relevance' });

    // Default mock for dashboard load calls
    vi.mocked(invoke).mockImplementation((cmd) => {
      if (cmd === 'get_total_products') return Promise.resolve(10);
      if (cmd === 'get_wishlist_count') return Promise.resolve(5);
      if (cmd === 'get_recent_searches') return Promise.resolve(['guitar', 'bass']);
      if (cmd === 'get_collection_stats') return Promise.resolve({ total_items: 0, total_value: 0, top_item_name: null, top_item_value: 0 });
      if (cmd === 'get_collection') return Promise.resolve([]);
      return Promise.resolve();
    });
  });

  it('renders 9 dashboard cells', async () => {
    render(Page);
    await waitFor(() => {
      const cells = document.querySelectorAll('.dashboard-cell');
      expect(cells).toHaveLength(9);
    });
  });

  it('shows gain/loss text when collection has items', async () => {
    vi.mocked(invoke).mockImplementation((cmd) => {
      if (cmd === 'get_total_products') return Promise.resolve(10);
      if (cmd === 'get_wishlist_count') return Promise.resolve(5);
      if (cmd === 'get_recent_searches') return Promise.resolve([]);
      if (cmd === 'get_collection_stats') return Promise.resolve({ total_items: 2, total_value: 2500, top_item_name: 'Fender', top_item_value: 1200 });
      if (cmd === 'get_collection') return Promise.resolve([
        { id: 1, sku: 'sku1', name: 'Guitar 1', brand: 'Fender', purchase_price: 1000, purchase_currency: 'USD', purchase_date: null, condition: 'good', serial_number: null, notes: null, image_url: null, added_at: 0, estimated_value: 1200 },
        { id: 2, sku: 'sku2', name: 'Guitar 2', brand: 'Gibson', purchase_price: 800, purchase_currency: 'USD', purchase_date: null, condition: 'good', serial_number: null, notes: null, image_url: null, added_at: 0, estimated_value: 700 },
      ]);
      return Promise.resolve();
    });
    render(Page);
    await waitFor(() => {
      expect(screen.getByText('gain/loss')).toBeInTheDocument();
    });
    const gainLossLabel = screen.getByText('gain/loss');
    const statContainer = gainLossLabel.closest('.collection-stats');
    expect(statContainer).toBeInTheDocument();
  });

  it('shows loading states', async () => {
    vi.mocked(invoke).mockImplementation(() => new Promise(() => {}));
    render(Page);
    await waitFor(() => {
      const loaders = screen.getAllByText('Loading...');
      expect(loaders.length).toBeGreaterThanOrEqual(1);
    });
  });

  it('shows empty states', async () => {
    vi.mocked(invoke).mockImplementation((cmd) => {
      if (cmd === 'get_total_products') return Promise.resolve(0);
      if (cmd === 'get_wishlist_count') return Promise.resolve(0);
      if (cmd === 'get_recent_searches') return Promise.resolve([]);
      if (cmd === 'get_collection_stats') return Promise.resolve({ total_items: 0, total_value: 0, top_item_name: null, top_item_value: 0 });
      if (cmd === 'get_collection') return Promise.resolve([]);
      return Promise.resolve();
    });
    render(Page);
    await waitFor(() => {
      expect(screen.getByText('No products in catalog yet')).toBeInTheDocument();
    });
  });

  it('renders FilterBar component', async () => {
    render(Page);
    await waitFor(() => {
      expect(screen.getByTestId('filter-toggle')).toBeInTheDocument();
    });
  });

  it('search sends filters from filterStore', async () => {
    // Set a filter before searching
    filterStore.update((s) => ({ ...s, category: 'Guitar', condition: 'new' }));

    vi.mocked(invoke).mockImplementation((cmd, args) => {
      if (cmd === 'search_products') {
        return Promise.resolve({
          products: [{ sku: 'ABC123', name: 'Test Guitar', brand: 'Fender', model: 'Strat', category: 'Guitar', subcategory: 'Electric', price: 999, currency: 'USD', condition: 'new', availability: 'In Stock', url: 'https://example.com', image_url: '', specs_json: '{}', seller: 'Test', location: 'US' }],
          total: 1,
          offset: 0,
          limit: 20,
        });
      }
      if (cmd === 'get_total_products') return Promise.resolve(10);
      if (cmd === 'get_wishlist_count') return Promise.resolve(5);
      if (cmd === 'get_recent_searches') return Promise.resolve(['guitar', 'bass']);
      if (cmd === 'get_collection_stats') return Promise.resolve({ total_items: 0, total_value: 0, top_item_name: null, top_item_value: 0 });
      if (cmd === 'get_collection') return Promise.resolve([]);
      return Promise.resolve();
    });

    render(Page);

    // Type a search query
    const input = screen.getByTestId('search-input') as HTMLInputElement;
    await fireEvent.input(input, { target: { value: 'test guitar' } });

    // Click search
    const button = screen.getByTestId('search-button');
    await fireEvent.click(button);

    // Wait for invoke to be called with search_products
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('search_products', expect.objectContaining({
        query: 'test guitar',
        filters: expect.objectContaining({
          category: 'Guitar',
          condition: 'new',
        }),
        sort: 'relevance',
      }));
    });
  });

  it('search sends all-null filters when no filters are set', async () => {
    // Reset filters to defaults
    filterStore.set({ category: null, price_min: null, price_max: null, source: null, condition: null, listing_currency: null, sort: 'relevance' });

    vi.mocked(invoke).mockImplementation((cmd, args) => {
      if (cmd === 'search_products') {
        return Promise.resolve({
          products: [],
          total: 0,
          offset: 0,
          limit: 20,
        });
      }
      if (cmd === 'get_total_products') return Promise.resolve(10);
      if (cmd === 'get_wishlist_count') return Promise.resolve(5);
      if (cmd === 'get_recent_searches') return Promise.resolve(['guitar', 'bass']);
      if (cmd === 'get_collection_stats') return Promise.resolve({ total_items: 0, total_value: 0, top_item_name: null, top_item_value: 0 });
      if (cmd === 'get_collection') return Promise.resolve([]);
      return Promise.resolve();
    });

    render(Page);

    const input = screen.getByTestId('search-input') as HTMLInputElement;
    await fireEvent.input(input, { target: { value: 'test' } });

    const button = screen.getByTestId('search-button');
    await fireEvent.click(button);

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('search_products', expect.objectContaining({
        filters: {
          category: null,
          price_min: null,
          price_max: null,
          source: null,
          condition: null,
          listing_currency: null,
        },
      }));
    });
  });
});
