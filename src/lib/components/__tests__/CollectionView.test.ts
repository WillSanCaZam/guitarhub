import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import CollectionView from '../CollectionView.svelte';
import { collectionStore } from '$lib/stores/collection';

const defaultStoreState = {
  items: [] as any[],
  loading: false,
  error: null,
  stats: null,
  collectedSkus: new Set<string>(),
};

describe('CollectionView', () => {
  it('shows empty state when no items', () => {
    collectionStore.set({ ...defaultStoreState, items: [] });
    render(CollectionView);
    expect(screen.getByText(/collection is empty/i)).toBeInTheDocument();
  });

  it('renders item list', () => {
    const items = [
      {
        id: 1,
        name: 'Fender Strat',
        brand: 'Fender',
        purchase_price: 1000,
        estimated_value: 1200,
        sku: null,
        purchase_currency: 'USD',
        purchase_date: null,
        condition: 'good',
        serial_number: null,
        notes: null,
        image_url: null,
        added_at: 0,
      },
    ];
    collectionStore.set({ ...defaultStoreState, items });
    render(CollectionView);
    expect(screen.getByText('Fender Strat')).toBeInTheDocument();
  });
});
