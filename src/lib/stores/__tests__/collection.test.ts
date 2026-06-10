import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { addToCollection } from '../collection';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('addToCollection', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('passes condition from product to Tauri command', async () => {
    vi.mocked(invoke).mockResolvedValue(1);

    await addToCollection({
      sku: 'SKU-001',
      name: 'Test Guitar',
      brand: 'Fender',
      price: 1500,
      currency: 'EUR',
      condition: 'excellent',
      image_url: 'https://example.com/img.jpg',
    });

    expect(invoke).toHaveBeenCalledWith('add_to_collection', {
      input: expect.objectContaining({
        condition: 'excellent',
      }),
    });
  });

  it('defaults condition to "good" when not provided', async () => {
    vi.mocked(invoke).mockResolvedValue(1);

    await addToCollection({
      sku: 'SKU-002',
      name: 'Test Guitar',
      brand: 'Gibson',
      price: 2000,
    });

    expect(invoke).toHaveBeenCalledWith('add_to_collection', {
      input: expect.objectContaining({
        condition: 'good',
      }),
    });
  });

  it('uses nullish coalescing for currency — empty string is preserved', async () => {
    vi.mocked(invoke).mockResolvedValue(1);

    await addToCollection({
      sku: 'SKU-003',
      name: 'Test Guitar',
      brand: 'Ibanez',
      price: 800,
      currency: '',
    });

    // With ??, empty string "" is preserved (not replaced with 'USD')
    expect(invoke).toHaveBeenCalledWith('add_to_collection', {
      input: expect.objectContaining({
        purchase_currency: '',
      }),
    });
  });

  it('falls back to USD when currency is null/undefined', async () => {
    vi.mocked(invoke).mockResolvedValue(1);

    await addToCollection({
      sku: 'SKU-004',
      name: 'Test Guitar',
      brand: 'ESP',
      price: 900,
    });

    expect(invoke).toHaveBeenCalledWith('add_to_collection', {
      input: expect.objectContaining({
        purchase_currency: 'USD',
      }),
    });
  });
});