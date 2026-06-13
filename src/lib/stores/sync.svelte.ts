export interface SyncResult {
  source_id?: string;
  products_loaded?: number;
  products_updated?: number;
  state?: string;
  progress?: number;
  drops?: Array<{
    sku: string;
    previous_price: number;
    new_price: number;
    channel: string;
    reason: string;
  }>;
  drops_sent?: number;
}

export const syncState: SyncResult | null = $state(null);
