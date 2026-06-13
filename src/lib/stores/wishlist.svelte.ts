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

export const wishlistState: WishlistStore = $state({ ...defaultStore });

export async function loadWishlist() {
  wishlistState.loading = true;
  wishlistState.error = null;
  try {
    const items = await invoke<WishlistItem[]>('get_wishlist');
    wishlistState.items = items;
    wishlistState.loading = false;
  } catch (e) {
    wishlistState.loading = false;
    wishlistState.error = String(e);
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
