import { invoke } from '@tauri-apps/api/core'
import type { Settings } from '../types'

export async function getSettings(): Promise<Settings> {
  return invoke<Settings>('get_settings')
}

export async function updateSettings(settings: Partial<Settings>): Promise<void> {
  return invoke<void>('update_settings', { settings })
}

export async function testConnection(settings: Settings): Promise<{ ok: boolean; error?: string }> {
  return invoke<{ ok: boolean; error?: string }>('test_connection', { settings })
    .then(() => ({ ok: true }))
    .catch((e) => ({ ok: false, error: String(e) }))
}
