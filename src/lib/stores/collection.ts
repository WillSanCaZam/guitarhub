import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { CollectionStats, CollectionItem } from '$lib/types/collection';

export interface CollectionStore {
  stats: CollectionStats | null;
  items: CollectionItem[];
  collectedSkus: Set<string>;
  loading: boolean;
  error: string | null;
}

const defaultStore: CollectionStore = {
  stats: null,
  items: [],
  collectedSkus: new Set(),
  loading: false,
  error: null,
};

export const collectionStore = writable<CollectionStore>({ ...defaultStore });

export async function loadCollectionStats() {
  collectionStore.update(s => ({ ...s, loading: true, error: null }));
  try {
    const stats = await invoke<CollectionStats>('get_collection_stats');
    collectionStore.update(s => ({ ...s, stats, loading: false }));
  } catch (e) {
    collectionStore.update(s => ({ ...s, loading: false, error: String(e) }));
  }
}

export async function loadCollection() {
  collectionStore.update(s => ({ ...s, loading: true, error: null }));
  try {
    const items = await invoke<CollectionItem[]>('get_collection');
    const collectedSkus = new Set<string>();
    for (const item of items) {
      if (item.sku) collectedSkus.add(item.sku);
    }
    collectionStore.update(s => ({ ...s, items, collectedSkus, loading: false }));
  } catch (e) {
    collectionStore.update(s => ({ ...s, loading: false, error: String(e) }));
  }
}

export async function addToCollection(product: {
  sku: string;
  name: string;
  brand: string;
  price: number;
  currency?: string;
  image_url?: string;
}) {
  await invoke('add_to_collection', {
    input: {
      sku: product.sku,
      name: product.name,
      brand: product.brand,
      purchase_price: product.price,
      purchase_currency: product.currency || 'USD',
      purchase_date: Math.floor(Date.now() / 1000),
      condition: 'good',
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
