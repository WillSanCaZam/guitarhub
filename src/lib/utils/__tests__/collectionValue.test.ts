import { describe, it, expect } from 'vitest';
import { calculateCollectionGainLoss, formatGainLoss } from '../collectionValue';
import type { CollectionItem } from '../types/collection';

const makeItem = (purchase_price: number | null, estimated_value: number | null): CollectionItem => ({
  id: 1,
  sku: null,
  name: 'Test',
  brand: null,
  purchase_price,
  purchase_currency: 'USD',
  purchase_date: null,
  condition: 'good',
  serial_number: null,
  notes: null,
  image_url: null,
  added_at: 0,
  estimated_value,
});

describe('calculateCollectionGainLoss', () => {
  it('returns positive gain when estimated value exceeds purchase price', () => {
    const items = [makeItem(1000, 1200)];
    expect(calculateCollectionGainLoss(items)).toBe(200);
  });

  it('returns negative loss when purchase price exceeds estimated value', () => {
    const items = [makeItem(1000, 800)];
    expect(calculateCollectionGainLoss(items)).toBe(-200);
  });

  it('treats null prices as zero', () => {
    const items = [makeItem(null, 500)];
    expect(calculateCollectionGainLoss(items)).toBe(500);
  });

  it('sums across multiple items', () => {
    const items = [makeItem(1000, 1200), makeItem(500, 400)];
    expect(calculateCollectionGainLoss(items)).toBe(100);
  });
});

describe('formatGainLoss', () => {
  it('formats positive with + sign and gain class', () => {
    const result = formatGainLoss(1500);
    expect(result.text).toBe('+1,500');
    expect(result.colorClass).toBe('gain');
  });

  it('formats negative with loss class', () => {
    const result = formatGainLoss(-500);
    expect(result.text).toBe('-500');
    expect(result.colorClass).toBe('loss');
  });

  it('formats zero with neutral class', () => {
    const result = formatGainLoss(0);
    expect(result.text).toBe('0');
    expect(result.colorClass).toBe('neutral');
  });
});
