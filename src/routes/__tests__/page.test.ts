import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';
import Page from '../+page.svelte';
import { invoke } from '@tauri-apps/api/core';
import { collectionState } from '$lib/stores/collection.svelte';
import { syncState } from '$lib/stores/sync.svelte';
import { filterState } from '$lib/stores/filter.svelte';

describe('Dashboard Page', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    collectionState.items = [];
    collectionState.stats = null;
    collectionState.collectedSkus = new Set();
    collectionState.loading = false;
    collectionState.error = null;
    filterState.category = null;
    filterState.price_min = null;
    filterState.price_max = null;
    filterState.source = null;
    filterState.condition = null;
    filterState.listing_currency = null;
    filterState.sort = 'relevance';

    // Default mock for all invoke calls
    vi.mocked(invoke).mockImplementation((cmd) => {
      if (cmd === 'list_connections') return Promise.resolve([]);
      if (cmd === 'get_featured_products') return Promise.resolve([]);
      if (cmd === 'get_price_drops') return Promise.resolve([]);
      if (cmd === 'get_new_arrivals') return Promise.resolve([]);
      if (cmd === 'get_collection') return Promise.resolve([]);
      if (cmd === 'get_collection_stats') return Promise.resolve({ total_items: 0, total_value: 0, top_item_name: null, top_item_value: 0 });
      return Promise.resolve();
    });
  });

  it('renders hero section with search bar', async () => {
    render(Page);
    await waitFor(() => {
      expect(screen.getByRole('banner')).toBeInTheDocument();
      expect(screen.getByRole('search', { name: /search gear/i })).toBeInTheDocument();
    });
  });

  it('renders featured rig section', async () => {
    render(Page);
    await waitFor(() => {
      expect(screen.getByText('Tim Henson')).toBeInTheDocument();
      expect(screen.getByText('Featured Rig of the Week')).toBeInTheDocument();
    });
  });

  it('renders trending section', async () => {
    render(Page);
    await waitFor(() => {
      expect(screen.getByText('Trending Now')).toBeInTheDocument();
      expect(screen.getAllByText('John Mayer Strat').length).toBeGreaterThanOrEqual(1);
    });
  });

  it('shows skeleton loading state while data loads', async () => {
    vi.mocked(invoke).mockImplementation(() => new Promise(() => {}));
    render(Page);
    await waitFor(() => {
      const skeletons = document.querySelectorAll('.skeleton-card');
      expect(skeletons.length).toBeGreaterThanOrEqual(1);
    });
  });

  it('renders product cards when data is available', async () => {
    vi.mocked(invoke).mockImplementation((cmd) => {
      if (cmd === 'list_connections') return Promise.resolve([]);
      if (cmd === 'get_featured_products') return Promise.resolve([
        { sku: 'sku1', source_id: 'reverb', name: 'Test Guitar', brand: 'Fender', model: 'Strat', category: 'Guitar', subcategory: 'Electric', price: 999, currency: 'USD', condition: 'new', availability: 'In Stock', url: 'https://example.com', image_url: '', specs_json: '{}', seller: 'Test', location: 'US' },
      ]);
      if (cmd === 'get_price_drops') return Promise.resolve([
        { sku: 'sku2', source_id: 'guitarcenter', name: 'Price Drop Pedal', brand: 'Boss', model: 'DS-1', category: 'Pedal', subcategory: 'Distortion', price: 49, currency: 'USD', condition: 'new', availability: 'In Stock', url: 'https://example.com', image_url: '', specs_json: '{}', seller: 'Test', location: 'US' },
      ]);
      if (cmd === 'get_new_arrivals') return Promise.resolve([]);
      if (cmd === 'get_collection') return Promise.resolve([]);
      if (cmd === 'get_collection_stats') return Promise.resolve({ total_items: 0, total_value: 0, top_item_name: null, top_item_value: 0 });
      return Promise.resolve();
    });

    render(Page);
    await waitFor(() => {
      expect(screen.getByText('Test Guitar')).toBeInTheDocument();
      expect(screen.getByText('Price Drop Pedal')).toBeInTheDocument();
    });
  });
});
