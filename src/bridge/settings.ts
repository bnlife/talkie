import { invoke } from '@tauri-apps/api/core'
import type { Settings, ModelProvider } from '../types'

export async function getSettings(): Promise<Settings> {
  return invoke<Settings>('get_settings')
}

export async function updateSettings(settings: Partial<Settings>): Promise<void> {
  return invoke<void>('update_settings', { settings })
}

export async function testProviderConnection(provider: ModelProvider): Promise<{ ok: boolean; error?: string }> {
  return invoke<string>('test_provider_connection', { provider })
    .then(() => ({ ok: true }))
    .catch((e) => ({ ok: false, error: String(e) }))
}

export async function fetchProviderModels(provider: ModelProvider): Promise<string[]> {
  return invoke<string[]>('fetch_provider_models', { provider })
}

export async function openUrl(url: string): Promise<void> {
  return invoke<void>('open_url', { url })
}
