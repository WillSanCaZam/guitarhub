import { writable } from 'svelte/store';

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
