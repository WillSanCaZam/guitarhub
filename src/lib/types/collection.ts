// SPDX-License-Identifier: GPL-3.0-or-later
//
// Frontend types for collection_items IPC contract.
// Keep in sync with `src-tauri/src/repository/collection.rs`.

export interface CollectionStats {
  total_items: number;
  total_value: number;
  top_item_name: string | null;
  top_item_value: number;
}

export interface CollectionItem {
  id: number;
  sku: string | null;
  name: string;
  brand: string | null;
  purchase_price: number | null;
  purchase_currency: string;
  purchase_date: number | null;
  condition: string;
  serial_number: string | null;
  notes: string | null;
  image_url: string | null;
  added_at: number;
  estimated_value: number | null;
}
