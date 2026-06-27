// SPDX-License-Identifier: GPL-3.0-or-later
//
// TypeScript mirrors of the Rust IPC contract for `search_products`.
// Keep these in sync with `src-tauri/src/domain/product.rs`.

export type SortOrder = "relevance" | "price_asc" | "price_desc" | "name_asc" | "name_desc";

export interface SearchFilters {
  category: string | null;
  price_min: number | null;
  price_max: number | null;
  source: string | null;
  condition: string | null;
  listing_currency: string | null;
  store_connection_id: string | null;
}

export interface RawProduct {
  sku: string;
  source_id: string;
  name: string;
  brand: string;
  model: string;
  category: string;
  subcategory: string;
  price: number;
  currency: string;
  condition: string;
  availability: string;
  url: string;
  image_url: string;
  specs_json: string;
  seller: string;
  location: string;
  user_id?: string | null;
  rating?: number;
}

export interface SearchResult {
  products: RawProduct[];
  total: number;
  offset: number;
  limit: number;
}

/**
 * Convert a 0-based offset + limit pair into a 1-based page number.
 * Centralised so the derivation logic exists in one place and can be unit-tested.
 */
export function pageFromOffset(offset: number, limit: number): number {
  if (limit <= 0) return 1;
  return Math.floor(offset / limit) + 1;
}
