import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';
import PriceChart from '../PriceChart.svelte';
import { invoke } from '@tauri-apps/api/core';

describe('PriceChart', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders chart with mock price history data', async () => {
    const mockPoints = [
      { source_id: 'src1', price: 100, recorded_at: 1 },
      { source_id: 'src1', price: 110, recorded_at: 2 },
    ];
    vi.mocked(invoke).mockResolvedValue(mockPoints);
    render(PriceChart, { props: { sku: 'test-sku', windowDays: 30 } });
    await waitFor(() => {
      expect(screen.getByRole('img')).toBeInTheDocument();
    });
  });

  it('shows empty state when no data', async () => {
    vi.mocked(invoke).mockResolvedValue([]);
    render(PriceChart, { props: { sku: 'test-sku', windowDays: 30 } });
    await waitFor(() => {
      expect(screen.getByRole('status')).toHaveTextContent('No price history available');
    });
  });

  it('shows loading state', async () => {
    vi.mocked(invoke).mockImplementation(() => new Promise(() => {}));
    render(PriceChart, { props: { sku: 'test-sku', windowDays: 30 } });
    expect(screen.getByText('Loading chart...')).toBeInTheDocument();
  });

  it('skips invoke when sku is falsy and shows empty state immediately', async () => {
    vi.mocked(invoke).mockResolvedValue([]);
    render(PriceChart, { props: { sku: '', windowDays: 30 } });
    await waitFor(() => {
      expect(screen.getByRole('status')).toHaveTextContent('No price history available');
    });
    expect(invoke).not.toHaveBeenCalled();
  });

  it('skips invoke when sku is undefined and shows empty state immediately', async () => {
    vi.mocked(invoke).mockResolvedValue([]);
    // @ts-expect-error — testing undefined SKU
    render(PriceChart, { props: { sku: undefined, windowDays: 30 } });
    await waitFor(() => {
      expect(screen.getByRole('status')).toHaveTextContent('No price history available');
    });
    expect(invoke).not.toHaveBeenCalled();
  });

  it('skips invoke when sku is whitespace-only and shows empty state', async () => {
    vi.mocked(invoke).mockResolvedValue([]);
    render(PriceChart, { props: { sku: '   ', windowDays: 30 } });
    await waitFor(() => {
      expect(screen.getByRole('status')).toHaveTextContent('No price history available');
    });
    expect(invoke).not.toHaveBeenCalled();
  });
});
