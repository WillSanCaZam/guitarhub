import type { CollectionItem } from '$lib/types/collection';

export function calculateCollectionGainLoss(items: CollectionItem[]): number {
  return items.reduce((sum, i) => sum + ((i.estimated_value ?? 0) - (i.purchase_price ?? 0)), 0);
}

export function formatGainLoss(value: number): { text: string; colorClass: string } {
  const sign = value > 0 ? '+' : '';
  const text = `${sign}${value.toLocaleString()}`;
  const colorClass = value > 0 ? 'gain' : value < 0 ? 'loss' : 'neutral';
  return { text, colorClass };
}
