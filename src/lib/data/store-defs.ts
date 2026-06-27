// SPDX-License-Identifier: GPL-3.0-or-later
//
// Hardcoded store definitions for the Store Registry.
// These mirror the Rust `StoreDef` values in
// `src-tauri/src/services/store_registry.rs`.

import type { StoreDef } from '$lib/types/stores'

export const storeDefs: StoreDef[] = [
  {
    id: 'reverb',
    name: 'Reverb',
    auth_type: 'pat',
    icon: 'reverb',
    website: 'https://reverb.com',
    token_url: 'https://reverb.com/settings/api',
  },
]
