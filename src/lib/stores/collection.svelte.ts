// SPDX-License-Identifier: GPL-3.0-or-later
//
// Collection state — Svelte 5 runes implementation.
// Replaces the writable()-based collection.ts.
// Exports reactive state + async action functions.

import { invoke } from '@tauri-apps/api/core';
import type { CollectionStats, CollectionItem } from '$lib/types/collection';

export interface CollectionStore {
  stats: CollectionStats | null;
  items: CollectionItem[];
  collectedSkus: Set<string>;
  loading: boolean;
  error: string | null;
}

/** Reactive collection state — access directly in components. */
export const collectionState: CollectionStore = $state({
  stats: null,
  items: [],
  collectedSkus: new Set(),
  loading: false,
  error: null,
});

export async function loadCollectionStats() {
  collectionState.loading = true;
  collectionState.error = null;
  try {
    const stats = await invoke<CollectionStats>('get_collection_stats');
    collectionState.stats = stats;
    collectionState.loading = false;
  } catch (e) {
    collectionState.loading = false;
    collectionState.error = String(e);
  }
}

export async function loadCollection() {
  collectionState.loading = true;
  collectionState.error = null;
  try {
    const items = await invoke<CollectionItem[]>('get_collection');
    const collectedSkus = new Set<string>();
    for (const item of items) {
      if (item.sku) collectedSkus.add(item.sku);
    }
    collectionState.items = items;
    collectionState.collectedSkus = collectedSkus;
    collectionState.loading = false;
  } catch (e) {
    collectionState.loading = false;
    collectionState.error = String(e);
  }
}

export async function addToCollection(product: {
  sku: string;
  name: string;
  brand: string;
  price: number;
  currency?: string;
  condition?: string;
  image_url?: string;
}) {
  await invoke('add_to_collection', {
    input: {
      sku: product.sku,
      name: product.name,
      brand: product.brand,
      purchase_price: product.price,
      purchase_currency: product.currency ?? 'USD',
      purchase_date: Math.floor(Date.now() / 1000),
      condition: product.condition ?? 'good',
      image_url: product.image_url ?? null,
    },
  });
  await loadCollection();
  await loadCollectionStats();
}

export async function removeFromCollection(id: number) {
  await invoke('remove_from_collection', { id });
  await loadCollection();
  await loadCollectionStats();
}
