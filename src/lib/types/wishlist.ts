// SPDX-License-Identifier: GPL-3.0-or-later
//
// Frontend types for wishlist IPC contract.
// Keep in sync with `src-tauri/src/repository/wishlist.rs`.

export interface WishlistItem {
  id: number;
  sku: string | null;
  name: string | null;
  brand: string | null;
  price: number | null;
  currency: string | null;
  image_url: string | null;
  product_url: string | null;
  notes: string | null;
  added_at: number | null;
}

export interface WishlistItemInput {
  sku?: string | null;
  name?: string | null;
  brand?: string | null;
  price?: number | null;
  currency?: string | null;
  image_url?: string | null;
  product_url?: string | null;
  notes?: string | null;
}