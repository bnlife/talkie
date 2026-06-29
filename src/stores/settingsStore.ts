import { defineStore } from 'pinia'
import type { Settings } from '../types'
import * as settingsBridge from '../bridge/settings'

export const useSettingsStore = defineStore('settings', {
  state: (): Settings => ({
    base_url: 'https://api.deepseek.com/v1',
    api_key: '',
    model: 'deepseek-chat',
    temperature: 0.7,
  }),

  actions: {
    async loadSettings(): Promise<void> {
      const s = await settingsBridge.getSettings()
      Object.assign(this, s)
    },

    async updateSettings(partial: Partial<Settings>): Promise<void> {
      Object.assign(this, partial)
      await settingsBridge.updateSettings(partial)
    },

    async testConnection(): Promise<{ ok: boolean; error?: string }> {
      return settingsBridge.testConnection({
        base_url: this.base_url,
        api_key: this.api_key,
        model: this.model,
        temperature: this.temperature,
      })
    },
  },
})
