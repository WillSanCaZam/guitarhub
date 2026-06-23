import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import GearCard from '../GearCard.svelte';
import { invoke } from '@tauri-apps/api/core';
import { addToCollection } from '$lib/stores/collection.svelte';
import { wishlistState } from '$lib/stores/wishlist.svelte';

vi.mock('$lib/stores/collection.svelte', async (importOriginal) => {
  const actual = await importOriginal<typeof import('$lib/stores/collection.svelte')>();
  return {
    ...actual,
    addToCollection: vi.fn(),
  };
});

vi.mock('$lib/stores/wishlist.svelte', async (importOriginal) => {
  const actual = await importOriginal<typeof import('$lib/stores/wishlist.svelte')>();
  return {
    ...actual,
    addToWishlist: vi.fn(),
    removeFromWishlist: vi.fn(),
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

describe('GearCard', () => {
  beforeEach(() => {
    wishlistState.items = [];
  });

  it('renders product info', async () => {
    vi.mocked(invoke).mockImplementation((cmd) => {
      if (cmd === 'get_product_image') return Promise.resolve('data:image/png;base64,test');
      if (cmd === 'get_price_insight') return Promise.resolve(null);
      return Promise.resolve();
    });
    render(GearCard, { props: { product: mockProduct } });
    await waitFor(() => {
      expect(screen.getByText('Test Guitar')).toBeInTheDocument();
    });
    expect(screen.getByText('Test Brand')).toBeInTheDocument();
    expect(screen.getByText('$999')).toBeInTheDocument();
  });

  it('renders price badge when priceInsight is present', async () => {
    vi.mocked(invoke).mockImplementation((cmd) => {
      if (cmd === 'get_product_image') return Promise.resolve('data:image/png;base64,test');
      if (cmd === 'get_price_insight') return Promise.resolve({ level: 'green', pct: 15, confidence: 85 });
      return Promise.resolve();
    });
    render(GearCard, { props: { product: mockProduct } });
    await waitFor(() => {
      expect(screen.getByText('Test Guitar')).toBeInTheDocument();
    });
    expect(screen.getByText('In Stock')).toBeInTheDocument();
  });

  it('calls addToCollection on button click', async () => {
    vi.mocked(invoke).mockImplementation((cmd) => {
      if (cmd === 'get_product_image') return Promise.resolve('data:image/png;base64,test');
      if (cmd === 'get_price_insight') return Promise.resolve(null);
      return Promise.resolve();
    });
    vi.mocked(addToCollection).mockResolvedValue(undefined);
    render(GearCard, { props: { product: mockProduct } });
    await waitFor(() => {
      expect(screen.getByText('Test Guitar')).toBeInTheDocument();
    });
    const button = screen.getByTestId('add-to-collection');
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
    render(GearCard, { props: { product: mockProduct, inCollection: true } });
    await waitFor(() => {
      expect(screen.getByText('Test Guitar')).toBeInTheDocument();
    });
    expect(screen.queryByRole('button', { name: /add to collection/i })).not.toBeInTheDocument();
  });

  it('has gear-card class instead of product-card', async () => {
    vi.mocked(invoke).mockImplementation((cmd) => {
      if (cmd === 'get_product_image') return Promise.resolve('data:image/png;base64,test');
      if (cmd === 'get_price_insight') return Promise.resolve(null);
      return Promise.resolve();
    });
    const { container } = render(GearCard, { props: { product: mockProduct } });
    await waitFor(() => {
      expect(screen.getByText('Test Guitar')).toBeInTheDocument();
    });
    const card = container.querySelector('.gear-card');
    expect(card).toBeInTheDocument();
    expect(container.querySelector('.product-card')).not.toBeInTheDocument();
  });

  it('renders condition badge when condition is provided', async () => {
    vi.mocked(invoke).mockImplementation((cmd) => {
      if (cmd === 'get_product_image') return Promise.resolve('data:image/png;base64,test');
      if (cmd === 'get_price_insight') return Promise.resolve(null);
      return Promise.resolve();
    });
    render(GearCard, { props: { product: { ...mockProduct, condition: 'Excellent' } } });
    await waitFor(() => {
      expect(screen.getByText('Test Guitar')).toBeInTheDocument();
    });
    expect(screen.getByText('Excellent')).toBeInTheDocument();
  });

  it('applies correct condition badge class', async () => {
    vi.mocked(invoke).mockImplementation((cmd) => {
      if (cmd === 'get_product_image') return Promise.resolve('data:image/png;base64,test');
      if (cmd === 'get_price_insight') return Promise.resolve(null);
      return Promise.resolve();
    });
    const { container } = render(GearCard, { props: { product: { ...mockProduct, condition: 'new' } } });
    await waitFor(() => {
      expect(screen.getByText('Test Guitar')).toBeInTheDocument();
    });
    const badge = container.querySelector('.badge-condition-new');
    expect(badge).toBeInTheDocument();
  });

  it('image gets loaded class after onload fires', async () => {
    vi.mocked(invoke).mockImplementation((cmd) => {
      if (cmd === 'get_product_image') return Promise.resolve('data:image/png;base64,test');
      if (cmd === 'get_price_insight') return Promise.resolve(null);
      return Promise.resolve();
    });
    const { container } = render(GearCard, { props: { product: mockProduct } });
    await waitFor(() => {
      expect(screen.getByText('Test Guitar')).toBeInTheDocument();
    });
    const img = container.querySelector('.product-image') as HTMLImageElement;
    expect(img).toBeInTheDocument();
    // Image starts without loaded class
    expect(img.classList.contains('loaded')).toBe(false);
    // Simulate load event
    fireEvent.load(img);
    await waitFor(() => {
      expect(img.classList.contains('loaded')).toBe(true);
    });
  });

  it('is keyboard accessible with tabindex', async () => {
    vi.mocked(invoke).mockImplementation((cmd) => {
      if (cmd === 'get_product_image') return Promise.resolve('data:image/png;base64,test');
      if (cmd === 'get_price_insight') return Promise.resolve(null);
      return Promise.resolve();
    });
    render(GearCard, { props: { product: mockProduct } });
    await waitFor(() => {
      expect(screen.getByText('Test Guitar')).toBeInTheDocument();
    });
    const card = screen.getByRole('article');
    expect(card).toHaveAttribute('tabindex', '0');
  });
});
