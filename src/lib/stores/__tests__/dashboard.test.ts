import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { dashboardStats, loadDashboard } from '../dashboard';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('$lib/stores/collection.svelte', () => ({
  loadCollectionStats: vi.fn().mockResolvedValue(undefined),
  loadCollection: vi.fn().mockResolvedValue(undefined),
}));

describe('loadDashboard', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    dashboardStats.set({ totalProducts: 0, wishlistCount: 0, recentSearches: [], loading: true, error: null });
  });

  it('sets loading true then updates stats on success', async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce(42)   // get_total_products
      .mockResolvedValueOnce(7)    // get_wishlist_count
      .mockResolvedValueOnce(['guitar', 'bass']); // get_recent_searches

    await loadDashboard();

    const state = getSnapshot(dashboardStats);
    expect(state.totalProducts).toBe(42);
    expect(state.wishlistCount).toBe(7);
    expect(state.recentSearches).toEqual(['guitar', 'bass']);
    expect(state.loading).toBe(false);
    expect(state.error).toBeNull();
  });

  it('sets error on failure', async () => {
    vi.mocked(invoke).mockRejectedValue(new Error('IPC failed'));

    await loadDashboard();

    const state = getSnapshot(dashboardStats);
    expect(state.loading).toBe(false);
    expect(state.error).toBe('Error: IPC failed');
  });
});

/** Read current value from a writable store. */
function getSnapshot<T>(store: { subscribe: (fn: (v: T) => void) => () => void }): T {
  let value: T;
  const unsub = store.subscribe((v) => { value = v; });
  unsub();
  return value!;
}
