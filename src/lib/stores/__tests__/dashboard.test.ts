import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { dashboardState, loadDashboard } from '../dashboard.svelte';

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
    Object.assign(dashboardState, { totalProducts: 0, wishlistCount: 0, recentSearches: [], loading: true, error: null });
  });

  it('sets loading true then updates stats on success', async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce(42)   // get_total_products
      .mockResolvedValueOnce(7)    // get_wishlist_count
      .mockResolvedValueOnce(['guitar', 'bass']); // get_recent_searches

    await loadDashboard();

    expect(dashboardState.totalProducts).toBe(42);
    expect(dashboardState.wishlistCount).toBe(7);
    expect(dashboardState.recentSearches).toEqual(['guitar', 'bass']);
    expect(dashboardState.loading).toBe(false);
    expect(dashboardState.error).toBeNull();
  });

  it('sets error on failure', async () => {
    vi.mocked(invoke).mockRejectedValue(new Error('IPC failed'));

    await loadDashboard();

    expect(dashboardState.loading).toBe(false);
    expect(dashboardState.error).toBe('Error: IPC failed');
  });
});
