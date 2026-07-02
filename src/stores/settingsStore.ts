import { defineStore } from 'pinia'
import type { Settings, ModelProvider } from '../types'
import * as settingsBridge from '../bridge/settings'
import { log } from '../bridge/log'

let nextProviderId = 1
function genProviderId(): string {
  return `prov-${Date.now()}-${nextProviderId++}`
}

export const useSettingsStore = defineStore('settings', {
  state: (): Settings => ({
    providers: [],
    active_provider_id: '',
    temperature: 0.7,
    top_p: 1.0,
    last_active_conversation_id: undefined,
    darkMode: false,
  }),

  getters: {
    activeProvider(state): ModelProvider | undefined {
      return state.providers.find((p) => p.id === state.active_provider_id)
    },
    enabledProviders(state): ModelProvider[] {
      return state.providers.filter((p) => p.enabled)
    },
  },

  actions: {
    async loadSettings(): Promise<void> {
      await log('info', 'FE::settingsStore | load')
      const s = await settingsBridge.getSettings()
      this.providers = s.providers ?? []
      this.active_provider_id = s.active_provider_id ?? ''
      this.temperature = s.temperature ?? 0.7
      this.top_p = s.top_p ?? 1.0
      this.last_active_conversation_id = s.last_active_conversation_id
      this.darkMode = s.darkMode ?? false
    },

    async saveSettings(): Promise<void> {
      await settingsBridge.updateSettings({
        providers: this.providers,
        active_provider_id: this.active_provider_id,
        temperature: this.temperature,
        top_p: this.top_p,
        last_active_conversation_id: this.last_active_conversation_id,
        darkMode: this.darkMode,
      })
    },

    async addProvider(preset?: Partial<ModelProvider>): Promise<ModelProvider> {
      const provider: ModelProvider = {
        id: genProviderId(),
        name: preset?.name ?? '新 Provider',
        icon: preset?.icon,
        base_url: preset?.base_url ?? '',
        api_key: preset?.api_key ?? '',
        headers: preset?.headers ?? {},
        models: preset?.models ?? [],
        enabled: true,
      }
      this.providers.unshift(provider)
      await this.saveSettings()
      await log('info', `FE::settingsStore | add provider | name=${provider.name} url=${provider.base_url}`)
      return provider
    },

    async removeProvider(id: string): Promise<void> {
      if (id === this.active_provider_id) {
        await log('warn', `FE::settingsStore | skip del active | id=${id}`)
        return
      }
      this.providers = this.providers.filter((p) => p.id !== id)
      await this.saveSettings()
      await log('info', `FE::settingsStore | del provider | id=${id}`)
    },

    async updateProvider(id: string, partial: Partial<ModelProvider>): Promise<void> {
      const idx = this.providers.findIndex((p) => p.id === id)
      if (idx === -1) return
      this.providers[idx] = { ...this.providers[idx], ...partial }
      await this.saveSettings()
    },

    async setActiveProvider(id: string): Promise<void> {
      this.active_provider_id = id
      await this.saveSettings()
    },

    async fetchModels(providerId: string): Promise<string[]> {
      const provider = this.providers.find((p) => p.id === providerId)
      if (!provider) return []
      await log('info', `FE::settingsStore | fetch models | provider=${providerId}`)
      const models = await settingsBridge.fetchProviderModels(provider)
      const merged = Array.from(new Set([...provider.models, ...models]))
      provider.models = merged
      await this.saveSettings()
      return models
    },

    addModel(providerId: string, model: string): void {
      const provider = this.providers.find((p) => p.id === providerId)
      if (!provider || !model.trim()) return
      if (!provider.models.includes(model.trim())) {
        provider.models.push(model.trim())
        this.saveSettings()
      }
    },

    removeModel(providerId: string, model: string): void {
      const provider = this.providers.find((p) => p.id === providerId)
      if (!provider) return
      provider.models = provider.models.filter((m) => m !== model)
      this.saveSettings()
    },

    async testConnection(providerId: string): Promise<{ ok: boolean; error?: string }> {
      const provider = this.providers.find((p) => p.id === providerId)
      if (!provider) return { ok: false, error: 'Provider 不存在' }
      await log('info', `FE::settingsStore | test conn | provider=${providerId}`)
      return settingsBridge.testProviderConnection(provider)
    },

    async verifyModel(providerId: string, model: string): Promise<{ ok: boolean; error?: string }> {
      const provider = this.providers.find((p) => p.id === providerId)
      if (!provider) return { ok: false, error: 'Provider 不存在' }
      await log('info', `FE::settingsStore | verify model | provider=${providerId} model=${model}`)
      return settingsBridge.verifyModel(provider, model)
    },
  },
})
