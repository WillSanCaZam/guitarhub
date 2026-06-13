import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import CollectionView from '../CollectionView.svelte';
import { collectionState } from '$lib/stores/collection.svelte';

describe('CollectionView', () => {
  it('shows empty state when no items', () => {
    collectionState.items = [];
    collectionState.loading = false;
    collectionState.error = null;
    collectionState.stats = null;
    collectionState.collectedSkus = new Set();
    render(CollectionView);
    expect(screen.getByText(/collection is empty/i)).toBeInTheDocument();
  });

  it('renders item list', () => {
    collectionState.items = [
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
    collectionState.loading = false;
    collectionState.error = null;
    collectionState.stats = null;
    collectionState.collectedSkus = new Set();
    render(CollectionView);
    expect(screen.getByText('Fender Strat')).toBeInTheDocument();
  });
});
