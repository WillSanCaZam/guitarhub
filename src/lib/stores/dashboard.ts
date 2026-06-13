import { writable } from 'svelte/store';
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

export const dashboardStats = writable<DashboardStats>({ ...defaultStats });

export async function loadDashboard() {
  dashboardStats.update(s => ({ ...s, loading: true, error: null }));
  try {
    const [totalProducts, wishlistCount, recentSearches] = await Promise.all([
      invoke<number>('get_total_products'),
      invoke<number>('get_wishlist_count'),
      invoke<string[]>('get_recent_searches')
    ]);
    dashboardStats.set({
      totalProducts,
      wishlistCount,
      recentSearches,
      loading: false,
      error: null,
    });
  } catch (e) {
    dashboardStats.update(s => ({ ...s, loading: false, error: String(e) }));
  }
  loadCollectionStats();
  loadCollection();
}
