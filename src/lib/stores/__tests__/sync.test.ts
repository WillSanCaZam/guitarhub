import { describe, it, expect, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { syncResult } from '../sync';
import type { SyncResult } from '../sync';

describe('syncResult store', () => {
  beforeEach(() => {
    syncResult.set(null);
  });

  it('has null initial state', () => {
    expect(get(syncResult)).toBeNull();
  });

  it('sets a sync result', () => {
    const result: SyncResult = {
      source_id: 'reverb',
      products_loaded: 50,
      products_updated: 10,
      state: 'done',
      progress: 100,
    };
    syncResult.set(result);
    expect(get(syncResult)).toEqual(result);
  });

  it('clears sync result', () => {
    syncResult.set({ source_id: 'reverb', state: 'done' });
    syncResult.set(null);
    expect(get(syncResult)).toBeNull();
  });

  it('updates with price drops', () => {
    const result: SyncResult = {
      drops: [
        { sku: 'SKU-001', previous_price: 1200, new_price: 999, channel: 'ntfy', reason: 'Price dropped' },
      ],
      drops_sent: 1,
    };
    syncResult.set(result);
    const state = get(syncResult);
    expect(state?.drops).toHaveLength(1);
    expect(state?.drops_sent).toBe(1);
  });

  it('handles partial result fields', () => {
    const partial: SyncResult = { state: 'syncing', progress: 45 };
    syncResult.set(partial);
    const state = get(syncResult);
    expect(state?.state).toBe('syncing');
    expect(state?.progress).toBe(45);
    expect(state?.source_id).toBeUndefined();
  });
});
