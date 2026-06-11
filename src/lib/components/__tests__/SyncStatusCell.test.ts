import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import SyncStatusCell from '../SyncStatusCell.svelte';

const mockDrops = [
  { sku: 'SKU-001', previous_price: 1200, new_price: 999, channel: 'ntfy', reason: 'Price dropped' },
  { sku: 'SKU-002', previous_price: 800, new_price: 650, channel: 'webhook', reason: 'Sale' },
];

describe('SyncStatusCell', () => {
  it('renders sync status title', () => {
    render(SyncStatusCell, { props: { drops: [], dropsSent: 0, syncState: 'idle' } });
    expect(screen.getByText('Sync Status')).toBeInTheDocument();
  });

  it('shows empty state when no drops', () => {
    render(SyncStatusCell, { props: { drops: [], dropsSent: 0, syncState: 'idle' } });
    expect(screen.getByText(/Sync catalog to see price drops/)).toBeInTheDocument();
  });

  it('shows drop count when drops exist', () => {
    render(SyncStatusCell, { props: { drops: mockDrops, dropsSent: 0, syncState: 'done' } });
    expect(screen.getByText(/2 price drop\(s\) detected/)).toBeInTheDocument();
  });

  it('shows sent count when drops were sent', () => {
    render(SyncStatusCell, { props: { drops: mockDrops, dropsSent: 1, syncState: 'done' } });
    expect(screen.getByText(/1 sent/)).toBeInTheDocument();
  });

  it('renders individual drop items', () => {
    render(SyncStatusCell, { props: { drops: mockDrops, dropsSent: 0, syncState: 'done' } });
    expect(screen.getByText('SKU-001')).toBeInTheDocument();
    expect(screen.getByText('$1200.00 → $999.00')).toBeInTheDocument();
    expect(screen.getByText('Price dropped')).toBeInTheDocument();
  });

  it('limits displayed drops to 3', () => {
    const manyDrops = [
      ...mockDrops,
      { sku: 'SKU-003', previous_price: 500, new_price: 400, channel: 'app', reason: 'Deal' },
      { sku: 'SKU-004', previous_price: 300, new_price: 250, channel: 'app', reason: 'Sale' },
    ];
    render(SyncStatusCell, { props: { drops: manyDrops, dropsSent: 0, syncState: 'done' } });
    expect(screen.getByText('SKU-001')).toBeInTheDocument();
    expect(screen.getByText('SKU-002')).toBeInTheDocument();
    expect(screen.getByText('SKU-003')).toBeInTheDocument();
    expect(screen.queryByText('SKU-004')).not.toBeInTheDocument();
  });
});
