import { describe, it, expect, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { dashboardStats } from '../dashboard';
import type { DashboardStats } from '../dashboard';

describe('dashboardStats store', () => {
  beforeEach(() => {
    dashboardStats.set({
      totalProducts: 0,
      wishlistCount: 0,
      recentSearches: [],
      loading: true,
      error: null,
    });
  });

  it('has correct initial state', () => {
    const state = get(dashboardStats);
    expect(state.totalProducts).toBe(0);
    expect(state.wishlistCount).toBe(0);
    expect(state.recentSearches).toEqual([]);
    expect(state.loading).toBe(true);
    expect(state.error).toBeNull();
  });

  it('updates totalProducts', () => {
    dashboardStats.update(s => ({ ...s, totalProducts: 42 }));
    expect(get(dashboardStats).totalProducts).toBe(42);
  });

  it('updates loading state', () => {
    dashboardStats.update(s => ({ ...s, loading: false }));
    expect(get(dashboardStats).loading).toBe(false);
  });

  it('sets error state', () => {
    dashboardStats.update(s => ({ ...s, error: 'Failed to load' }));
    expect(get(dashboardStats).error).toBe('Failed to load');
  });

  it('updates recentSearches', () => {
    const searches = ['fender strat', 'gibson les paul'];
    dashboardStats.update(s => ({ ...s, recentSearches: searches }));
    expect(get(dashboardStats).recentSearches).toEqual(searches);
  });

  it('allows full state replacement', () => {
    const newState: DashboardStats = {
      totalProducts: 100,
      wishlistCount: 15,
      recentSearches: ['test'],
      loading: false,
      error: null,
    };
    dashboardStats.set(newState);
    expect(get(dashboardStats)).toEqual(newState);
  });
});
