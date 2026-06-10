// SPDX-License-Identifier: GPL-3.0-or-later
//
// Filter state store with URL serialisation helpers.
// `filterStore` is the single source of truth for search filter UI state.
// URL sync is a side-effect of store changes (debounced 300ms).
// On mount, URL params are restored into the store.

import { writable } from 'svelte/store';
import type { Writable } from 'svelte/store';
import type { SortOrder } from '$lib/types/search';

export interface FilterState {
  category: string | null;
  price_min: number | null;
  price_max: number | null;
  source: string | null;
  condition: string | null;
  listing_currency: string | null;
  sort: SortOrder;
}

export const DEFAULT_FILTERS: FilterState = {
  category: null,
  price_min: null,
  price_max: null,
  source: null,
  condition: null,
  listing_currency: null,
  sort: 'relevance',
};

/** Serialise non-null filter fields into URLSearchParams. */
export function filtersToParams(filters: FilterState): URLSearchParams {
  const params = new URLSearchParams();

  if (filters.category !== null) params.set('category', filters.category);
  if (filters.price_min !== null) params.set('price_min', String(filters.price_min));
  if (filters.price_max !== null) params.set('price_max', String(filters.price_max));
  if (filters.source !== null) params.set('source', filters.source);
  if (filters.condition !== null) params.set('condition', filters.condition);
  if (filters.listing_currency !== null) params.set('listing_currency', filters.listing_currency);
  // sort is always non-null per FilterState
  if (filters.sort !== 'relevance') params.set('sort', filters.sort);

  return params;
}

/** Deserialise URLSearchParams back into a FilterState. */
export function paramsToFilters(params: URLSearchParams): FilterState {
  const rawSort = params.get('sort');
  const sort: SortOrder = rawSort !== null
    && ['relevance', 'price_asc', 'price_desc', 'name_asc', 'name_desc'].includes(rawSort as SortOrder)
    ? (rawSort as SortOrder)
    : 'relevance';

  const rawPriceMin = params.get('price_min');
  const rawPriceMax = params.get('price_max');

  return {
    category: params.get('category') ?? null,
    price_min: rawPriceMin !== null ? Number(rawPriceMin) : null,
    price_max: rawPriceMax !== null ? Number(rawPriceMax) : null,
    source: params.get('source') ?? null,
    condition: params.get('condition') ?? null,
    listing_currency: params.get('listing_currency') ?? null,
    sort,
  };
}

/** Debounced write of current filter state to `window.location.search`. */
let syncTimeout: ReturnType<typeof setTimeout> | undefined;

export function syncFiltersToUrl(state: FilterState): void {
  if (syncTimeout !== undefined) clearTimeout(syncTimeout);
  syncTimeout = setTimeout(() => {
    const params = filtersToParams(state);
    const qs = params.toString();
    const newUrl = qs ? `${window.location.pathname}?${qs}` : window.location.pathname;
    window.history.replaceState(null, '', newUrl);
  }, 300);
}

/** Read current filter state from `window.location.search`. */
export function restoreFiltersFromUrl(): FilterState {
  const params = new URLSearchParams(window.location.search);
  return paramsToFilters(params);
}

/** The writable store that UI components subscribe to. */
export const filterStore: Writable<FilterState> = writable<FilterState>({ ...DEFAULT_FILTERS });
