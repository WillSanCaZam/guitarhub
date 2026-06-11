import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import CollectionStatsCell from '../CollectionStatsCell.svelte';
import type { CollectionStats, CollectionItem } from '$lib/types/collection';

vi.mock('$lib/utils/collectionValue', () => ({
  calculateCollectionGainLoss: vi.fn(() => 150),
  formatGainLoss: vi.fn(() => ({ text: '+$150', colorClass: 'gain' })),
}));

const mockStats: CollectionStats = {
  total_items: 5,
  total_value: 4500,
  top_item_name: 'Fender Stratocaster',
  top_item_value: 1200,
};

const mockItems: CollectionItem[] = [
  { id: 1, sku: 'SKU-001', name: 'Guitar', brand: 'Fender', purchase_price: 1000, condition: 'excellent' },
] as CollectionItem[];

describe('CollectionStatsCell', () => {
  it('renders collection stats when data is present', () => {
    render(CollectionStatsCell, { props: { stats: mockStats, items: mockItems, loading: false } });
    expect(screen.getByText('Collection')).toBeInTheDocument();
    expect(screen.getByText('5')).toBeInTheDocument();
    expect(screen.getByText('$4500')).toBeInTheDocument();
  });

  it('shows top item when available', () => {
    render(CollectionStatsCell, { props: { stats: mockStats, items: mockItems, loading: false } });
    expect(screen.getByText(/Top: Fender Stratocaster/)).toBeInTheDocument();
  });

  it('shows empty state when stats is null', () => {
    render(CollectionStatsCell, { props: { stats: null, items: [], loading: false } });
    expect(screen.getByText(/Start adding gear/)).toBeInTheDocument();
  });

  it('shows empty state when total_items is 0', () => {
    const emptyStats = { ...mockStats, total_items: 0 };
    render(CollectionStatsCell, { props: { stats: emptyStats, items: [], loading: false } });
    expect(screen.getByText(/Start adding gear/)).toBeInTheDocument();
  });

  it('shows loading state', () => {
    render(CollectionStatsCell, { props: { stats: null, items: [], loading: true } });
    expect(screen.getByText('Loading...')).toBeInTheDocument();
  });

  it('links to /collection page', () => {
    render(CollectionStatsCell, { props: { stats: mockStats, items: mockItems, loading: false } });
    const link = screen.getByRole('link');
    expect(link).toHaveAttribute('href', '/collection');
  });
});
