// SPDX-License-Identifier: GPL-3.0-or-later
//
// Frontend types for store connections IPC contract.
// Keep in sync with `src-tauri/src/domain/store.rs`.

export interface StoreDef {
  id: string
  name: string
  auth_type: 'pat' | 'oauth'
  icon: string
  website: string
  token_url: string
}

export interface Connection {
  id: number
  store_id: string
  label: string
  username: string | null
  connected_at: number
  synced_at: number | null
  is_active: boolean
}
