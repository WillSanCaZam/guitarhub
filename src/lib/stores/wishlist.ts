import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { WishlistItem, WishlistItemInput } from '$lib/types/wishlist';

export interface WishlistStore {
  items: WishlistItem[];
  loading: boolean;
  error: string | null;
}

const defaultStore: WishlistStore = {
  items: [],
  loading: false,
  error: null,
};

export const wishlistStore = writable<WishlistStore>({ ...defaultStore });

export async function loadWishlist() {
  wishlistStore.update(s => ({ ...s, loading: true, error: null }));
  try {
    const items = await invoke<WishlistItem[]>('get_wishlist');
    wishlistStore.update(s => ({ ...s, items, loading: false }));
  } catch (e) {
    wishlistStore.update(s => ({ ...s, loading: false, error: String(e) }));
  }
}

export async function addToWishlist(input: WishlistItemInput) {
  await invoke('add_to_wishlist', { input });
  await loadWishlist();
}

export async function removeFromWishlist(id: number) {
  await invoke('remove_from_wishlist', { id });
  await loadWishlist();
}