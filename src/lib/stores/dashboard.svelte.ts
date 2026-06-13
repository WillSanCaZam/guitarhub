import { invoke } from '@tauri-apps/api/core';
import { loadCollectionStats, loadCollection } from './collection.svelte';

export interface DashboardStats {
  totalProducts: number;
  wishlistCount: number;
  recentSearches: string[];
  loading: boolean;
  error: string | null;
}

const defaultStats: DashboardStats = {
  totalProducts: 0,
  wishlistCount: 0,
  recentSearches: [],
  loading: true,
  error: null,
};

export const dashboardState: DashboardStats = $state({ ...defaultStats });

export async function loadDashboard() {
  dashboardState.loading = true;
  dashboardState.error = null;
  try {
    const [totalProducts, wishlistCount, recentSearches] = await Promise.all([
      invoke<number>('get_total_products'),
      invoke<number>('get_wishlist_count'),
      invoke<string[]>('get_recent_searches')
    ]);
    dashboardState.totalProducts = totalProducts;
    dashboardState.wishlistCount = wishlistCount;
    dashboardState.recentSearches = recentSearches;
    dashboardState.loading = false;
  } catch (e) {
    dashboardState.loading = false;
    dashboardState.error = String(e);
  }
  loadCollectionStats();
  loadCollection();
}
