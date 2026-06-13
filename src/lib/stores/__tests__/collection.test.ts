import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import {
  collectionState,
  addToCollection,
  loadCollection,
  loadCollectionStats,
  removeFromCollection,
} from '../collection.svelte';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('collection runes store', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Reset state to defaults
    collectionState.stats = null;
    collectionState.items = [];
    collectionState.collectedSkus = new Set();
    collectionState.loading = false;
    collectionState.error = null;
  });

  describe('initial state', () => {
    it('has correct default shape', () => {
      expect(collectionState.stats).toBeNull();
      expect(collectionState.items).toEqual([]);
      expect(collectionState.collectedSkus.size).toBe(0);
      expect(collectionState.loading).toBe(false);
      expect(collectionState.error).toBeNull();
    });
  });

  describe('loadCollectionStats', () => {
    it('sets loading true then updates stats on success', async () => {
      const mockStats = { total_items: 5, total_value: 1200, top_item_name: 'Fender', top_item_value: 800 };
      vi.mocked(invoke).mockResolvedValue(mockStats);

      const promise = loadCollectionStats();
      // During execution, loading should be true
      expect(collectionState.loading).toBe(true);

      await promise;
      expect(collectionState.stats).toEqual(mockStats);
      expect(collectionState.loading).toBe(false);
      expect(collectionState.error).toBeNull();
    });

    it('sets error on failure', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('IPC failed'));

      await loadCollectionStats();
      expect(collectionState.stats).toBeNull();
      expect(collectionState.loading).toBe(false);
      expect(collectionState.error).toBe('Error: IPC failed');
    });
  });

  describe('loadCollection', () => {
    it('updates items and builds collectedSkus set', async () => {
      const mockItems = [
        { id: 1, sku: 'SKU-001', name: 'Guitar 1', brand: 'Fender', purchase_price: 1000, purchase_currency: 'USD', purchase_date: null, condition: 'good', serial_number: null, notes: null, image_url: null, added_at: 0, estimated_value: 1200 },
        { id: 2, sku: 'SKU-002', name: 'Guitar 2', brand: 'Gibson', purchase_price: 800, purchase_currency: 'USD', purchase_date: null, condition: 'good', serial_number: null, notes: null, image_url: null, added_at: 0, estimated_value: 700 },
        { id: 3, sku: null, name: 'Guitar 3', brand: 'Ibanez', purchase_price: 500, purchase_currency: 'USD', purchase_date: null, condition: 'good', serial_number: null, notes: null, image_url: null, added_at: 0, estimated_value: 500 },
      ];
      vi.mocked(invoke).mockResolvedValue(mockItems);

      await loadCollection();

      expect(collectionState.items).toEqual(mockItems);
      expect(collectionState.collectedSkus.size).toBe(2);
      expect(collectionState.collectedSkus.has('SKU-001')).toBe(true);
      expect(collectionState.collectedSkus.has('SKU-002')).toBe(true);
      expect(collectionState.loading).toBe(false);
    });

    it('handles empty collection', async () => {
      vi.mocked(invoke).mockResolvedValue([]);

      await loadCollection();

      expect(collectionState.items).toEqual([]);
      expect(collectionState.collectedSkus.size).toBe(0);
      expect(collectionState.loading).toBe(false);
    });

    it('sets error on failure', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Network error'));

      await loadCollection();
      expect(collectionState.items).toEqual([]);
      expect(collectionState.loading).toBe(false);
      expect(collectionState.error).toBe('Error: Network error');
    });
  });

  describe('addToCollection', () => {
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

  describe('removeFromCollection', () => {
    it('calls invoke with correct id and reloads', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined) // remove_from_collection
        .mockResolvedValueOnce([])       // get_collection
        .mockResolvedValueOnce({ total_items: 0, total_value: 0, top_item_name: null, top_item_value: 0 }); // get_collection_stats

      await removeFromCollection(42);

      expect(invoke).toHaveBeenCalledWith('remove_from_collection', { id: 42 });
      expect(collectionState.loading).toBe(false);
    });
  });
});
