import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import ProductCard from '../ProductCard.svelte';
import { invoke } from '@tauri-apps/api/core';
import { addToCollection } from '$lib/stores/collection';

vi.mock('$lib/stores/collection', async (importOriginal) => {
  const actual = await importOriginal<typeof import('$lib/stores/collection')>();
  return {
    ...actual,
    addToCollection: vi.fn(),
  };
});

const mockProduct = {
  sku: 'test-sku',
  name: 'Test Guitar',
  brand: 'Test Brand',
  price: 999,
  currency: 'USD',
  image_url: 'http://example.com/image.jpg',
};

describe('ProductCard', () => {
  it('renders product info', async () => {
    vi.mocked(invoke).mockImplementation((cmd) => {
      if (cmd === 'get_product_image') return Promise.resolve('data:image/png;base64,test');
      if (cmd === 'get_price_insight') return Promise.resolve(null);
      return Promise.resolve();
    });
    render(ProductCard, { props: { product: mockProduct } });
    await waitFor(() => {
      expect(screen.getByText('Test Guitar')).toBeInTheDocument();
    });
    expect(screen.getByText('Test Brand')).toBeInTheDocument();
    expect(screen.getByText('999 USD')).toBeInTheDocument();
  });

  it('renders price badge when priceInsight is present', async () => {
    vi.mocked(invoke).mockImplementation((cmd) => {
      if (cmd === 'get_product_image') return Promise.resolve('data:image/png;base64,test');
      if (cmd === 'get_price_insight') return Promise.resolve({ level: 'green', pct: 15, confidence: 85 });
      return Promise.resolve();
    });
    render(ProductCard, { props: { product: mockProduct } });
    await waitFor(() => {
      expect(screen.getByRole('status')).toBeInTheDocument();
    });
  });

  it('calls addToCollection on button click', async () => {
    vi.mocked(invoke).mockImplementation((cmd) => {
      if (cmd === 'get_product_image') return Promise.resolve('data:image/png;base64,test');
      if (cmd === 'get_price_insight') return Promise.resolve(null);
      return Promise.resolve();
    });
    vi.mocked(addToCollection).mockResolvedValue(undefined);
    render(ProductCard, { props: { product: mockProduct } });
    await waitFor(() => {
      expect(screen.getByText('Test Guitar')).toBeInTheDocument();
    });
    const button = screen.getByRole('button', { name: /add to collection/i });
    await fireEvent.click(button);
    expect(addToCollection).toHaveBeenCalledWith(mockProduct);
    await waitFor(() => {
      expect(screen.getByText('Added ✓')).toBeInTheDocument();
    });
  });

  it('hides add button when inCollection is true', async () => {
    vi.mocked(invoke).mockImplementation((cmd) => {
      if (cmd === 'get_product_image') return Promise.resolve('data:image/png;base64,test');
      if (cmd === 'get_price_insight') return Promise.resolve(null);
      return Promise.resolve();
    });
    render(ProductCard, { props: { product: mockProduct, inCollection: true } });
    await waitFor(() => {
      expect(screen.getByText('Test Guitar')).toBeInTheDocument();
    });
    expect(screen.queryByRole('button', { name: /add to collection/i })).not.toBeInTheDocument();
  });
});
