import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import ProductDetail from '../ProductDetail.svelte';

vi.mock('$tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

const mockProduct = {
  sku: 'test-sku',
  name: 'Test Guitar',
  brand: 'Test Brand',
  price: 1500,
  currency: 'USD',
};

describe('ProductDetail', () => {
  it('renders product name', () => {
    render(ProductDetail, { props: { product: mockProduct } });
    expect(screen.getByText('Test Guitar')).toBeInTheDocument();
  });

  it('renders brand when provided', () => {
    render(ProductDetail, { props: { product: mockProduct } });
    expect(screen.getByText('Test Brand')).toBeInTheDocument();
  });

  it('renders price with currency', () => {
    render(ProductDetail, { props: { product: mockProduct } });
    expect(screen.getByText('1500 USD')).toBeInTheDocument();
  });

  it('hides brand when not provided', () => {
    const noBrand = { ...mockProduct, brand: undefined };
    render(ProductDetail, { props: { product: noBrand } });
    expect(screen.queryByText('Test Brand')).not.toBeInTheDocument();
  });

  it('hides price when not provided', () => {
    const noPrice = { ...mockProduct, price: undefined };
    render(ProductDetail, { props: { product: noPrice } });
    expect(screen.queryByText(/1500/)).not.toBeInTheDocument();
  });

  it('renders price history section', () => {
    render(ProductDetail, { props: { product: mockProduct } });
    expect(screen.getByText('Price History')).toBeInTheDocument();
  });
});
