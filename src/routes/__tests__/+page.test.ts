import { describe, it, expect, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';
import Page from '../+page.svelte';
import { invoke } from '@tauri-apps/api/core';
import { dashboardStats } from '$lib/stores/dashboard';
import { syncResult } from '$lib/stores/sync';
import { collectionStore } from '$lib/stores/collection';

describe('Dashboard Page', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    dashboardStats.set({ totalProducts: 0, wishlistCount: 0, recentSearches: [], loading: false, error: null });
    syncResult.set({ drops: [], drops_sent: 0, state: 'idle' });
    collectionStore.set({ items: [], stats: null, collectedSkus: new Set(), loading: false, error: null });
  });

  it('renders 9 dashboard cells', async () => {
    vi.mocked(invoke).mockImplementation((cmd) => {
      if (cmd === 'get_total_products') return Promise.resolve(10);
      if (cmd === 'get_wishlist_count') return Promise.resolve(5);
      if (cmd === 'get_recent_searches') return Promise.resolve(['guitar', 'bass']);
      if (cmd === 'get_collection_stats') return Promise.resolve({ total_items: 0, total_value: 0, top_item_name: null, top_item_value: 0 });
      if (cmd === 'get_collection') return Promise.resolve([]);
      return Promise.resolve();
    });
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
});
