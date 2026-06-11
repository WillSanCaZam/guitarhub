import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { wishlistStore, loadWishlist, addToWishlist, removeFromWishlist } from '../wishlist';
import { invoke } from '@tauri-apps/api/core';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('wishlistStore', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    wishlistStore.set({ items: [], loading: false, error: null });
  });

  it('has correct initial state', () => {
    const state = get(wishlistStore);
    expect(state.items).toEqual([]);
    expect(state.loading).toBe(false);
    expect(state.error).toBeNull();
  });

  it('loadWishlist sets loading then items', async () => {
    const mockItems = [
      { id: 1, sku: 'SKU-001', name: 'Guitar', brand: 'Fender', price: 1500, currency: 'USD' },
    ];
    vi.mocked(invoke).mockResolvedValue(mockItems);

    await loadWishlist();

    const state = get(wishlistStore);
    expect(state.items).toEqual(mockItems);
    expect(state.loading).toBe(false);
    expect(state.error).toBeNull();
  });

  it('loadWishlist sets error on failure', async () => {
    vi.mocked(invoke).mockRejectedValue(new Error('Network error'));

    await loadWishlist();

    const state = get(wishlistStore);
    expect(state.error).toBe('Error: Network error');
    expect(state.loading).toBe(false);
  });

  it('addToWishlist invokes Tauri and reloads', async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);

    await addToWishlist({ sku: 'SKU-002', name: 'Pedal', brand: 'Boss', price: 100 });

    expect(invoke).toHaveBeenCalledWith('add_to_wishlist', {
      input: { sku: 'SKU-002', name: 'Pedal', brand: 'Boss', price: 100 },
    });
    // Second call is loadWishlist's invoke
    expect(invoke).toHaveBeenCalledTimes(2);
  });

  it('removeFromWishlist invokes Tauri and reloads', async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);

    await removeFromWishlist(42);

    expect(invoke).toHaveBeenCalledWith('remove_from_wishlist', { id: 42 });
    expect(invoke).toHaveBeenCalledTimes(2);
  });
});
