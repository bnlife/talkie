import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import type { Settings } from '../../types'
import { useSettingsStore } from '../../stores/settingsStore'
import * as settingsBridge from '../../bridge/settings'
import { createProvider } from './helpers'

vi.mock('../../bridge/settings')

describe('settingsStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('has correct default values', () => {
    const store = useSettingsStore()
    expect(store.providers).toEqual([])
    expect(store.active_provider_id).toBe('')
    expect(store.temperature).toBe(0.7)
  })

  describe('loadSettings', () => {
    it('fetches remote settings and applies them to state', async () => {
      const remote: Settings = {
        providers: [createProvider()],
        active_provider_id: 'prov-1',
        temperature: 0.5,
        top_p: 0.9,
      }
      vi.mocked(settingsBridge.getSettings).mockResolvedValue(remote)

      const store = useSettingsStore()
      await store.loadSettings()

      expect(settingsBridge.getSettings).toHaveBeenCalledOnce()
      expect(store.providers).toHaveLength(1)
      expect(store.providers[0].id).toBe('prov-1')
      expect(store.active_provider_id).toBe('prov-1')
      expect(store.temperature).toBe(0.5)
    })
  })

  describe('addProvider', () => {
    it('prepends provider to list without changing active', async () => {
      vi.mocked(settingsBridge.updateSettings).mockResolvedValue(undefined)

      const store = useSettingsStore()
      store.providers = [createProvider({ id: 'old' })]
      store.active_provider_id = 'old'

      const p = await store.addProvider({ name: 'OpenAI', base_url: 'https://api.openai.com/v1' })

      expect(store.providers).toHaveLength(2)
      expect(store.providers[0].name).toBe('OpenAI')
      expect(store.active_provider_id).toBe('old')
      expect(settingsBridge.updateSettings).toHaveBeenCalled()
    })
  })

  describe('removeProvider', () => {
    it('removes provider from list', async () => {
      vi.mocked(settingsBridge.updateSettings).mockResolvedValue(undefined)

      const store = useSettingsStore()
      store.providers = [createProvider({ id: 'p1' }), createProvider({ id: 'p2' })]
      store.active_provider_id = 'p2'

      await store.removeProvider('p1')

      expect(store.providers).toHaveLength(1)
      expect(store.providers[0].id).toBe('p2')
    })

    it('refuses to remove active provider', async () => {
      vi.mocked(settingsBridge.updateSettings).mockResolvedValue(undefined)

      const store = useSettingsStore()
      store.providers = [createProvider({ id: 'p1' }), createProvider({ id: 'p2' })]
      store.active_provider_id = 'p1'

      await store.removeProvider('p1')

      expect(store.providers).toHaveLength(2)
      expect(settingsBridge.updateSettings).not.toHaveBeenCalled()
    })
  })

  describe('updateProvider', () => {
    it('merges partial update into provider', async () => {
      vi.mocked(settingsBridge.updateSettings).mockResolvedValue(undefined)

      const store = useSettingsStore()
      store.providers = [createProvider({ id: 'p1', name: 'Old' })]

      await store.updateProvider('p1', { name: 'New' })

      expect(store.providers[0].name).toBe('New')
      expect(settingsBridge.updateSettings).toHaveBeenCalled()
    })
  })

  describe('setActiveProvider', () => {
    it('changes active_provider_id', async () => {
      vi.mocked(settingsBridge.updateSettings).mockResolvedValue(undefined)

      const store = useSettingsStore()
      store.providers = [createProvider({ id: 'p1' }), createProvider({ id: 'p2' })]

      await store.setActiveProvider('p2')

      expect(store.active_provider_id).toBe('p2')
    })
  })

  describe('fetchModels', () => {
    it('calls bridge and updates provider models', async () => {
      vi.mocked(settingsBridge.fetchProviderModels).mockResolvedValue(['gpt-4o', 'gpt-4o-mini'])
      vi.mocked(settingsBridge.updateSettings).mockResolvedValue(undefined)

      const store = useSettingsStore()
      store.providers = [createProvider({ id: 'p1', models: [] })]

      await store.fetchModels('p1')

      expect(settingsBridge.fetchProviderModels).toHaveBeenCalledWith(store.providers[0])
      expect(store.providers[0].models).toEqual(['gpt-4o', 'gpt-4o-mini'])
    })
  })

  describe('addModel', () => {
    it('appends model to provider', () => {
      vi.mocked(settingsBridge.updateSettings).mockResolvedValue(undefined)

      const store = useSettingsStore()
      store.providers = [createProvider({ id: 'p1', models: ['gpt-4'] })]

      store.addModel('p1', 'gpt-4o')

      expect(store.providers[0].models).toEqual(['gpt-4', 'gpt-4o'])
    })

    it('does not add duplicate model', () => {
      vi.mocked(settingsBridge.updateSettings).mockResolvedValue(undefined)

      const store = useSettingsStore()
      store.providers = [createProvider({ id: 'p1', models: ['gpt-4'] })]

      store.addModel('p1', 'gpt-4')

      expect(store.providers[0].models).toEqual(['gpt-4'])
    })
  })

  describe('removeModel', () => {
    it('removes model from provider', () => {
      vi.mocked(settingsBridge.updateSettings).mockResolvedValue(undefined)

      const store = useSettingsStore()
      store.providers = [createProvider({ id: 'p1', models: ['gpt-4', 'gpt-4o'] })]

      store.removeModel('p1', 'gpt-4')

      expect(store.providers[0].models).toEqual(['gpt-4o'])
    })
  })

  describe('testConnection', () => {
    it('calls bridge with provider and returns result', async () => {
      vi.mocked(settingsBridge.testProviderConnection).mockResolvedValue({ ok: true })

      const store = useSettingsStore()
      store.providers = [createProvider({ id: 'p1' })]

      const result = await store.testConnection('p1')

      expect(settingsBridge.testProviderConnection).toHaveBeenCalledWith(store.providers[0])
      expect(result).toEqual({ ok: true })
    })

    it('returns error for non-existent provider', async () => {
      const store = useSettingsStore()

      const result = await store.testConnection('non-existent')

      expect(result.ok).toBe(false)
      expect(result.error).toBe('Provider 不存在')
    })
  })

  describe('getters', () => {
    it('activeProvider returns the active provider', () => {
      const store = useSettingsStore()
      store.providers = [createProvider({ id: 'p1' }), createProvider({ id: 'p2' })]
      store.active_provider_id = 'p2'

      expect(store.activeProvider?.id).toBe('p2')
    })

    it('activeProvider returns undefined when no active', () => {
      const store = useSettingsStore()
      store.providers = [createProvider({ id: 'p1' })]
      store.active_provider_id = ''

      expect(store.activeProvider).toBeUndefined()
    })

    it('enabledProviders filters disabled providers', () => {
      const store = useSettingsStore()
      store.providers = [
        createProvider({ id: 'p1', enabled: true }),
        createProvider({ id: 'p2', enabled: false }),
        createProvider({ id: 'p3', enabled: true }),
      ]

      expect(store.enabledProviders).toHaveLength(2)
      expect(store.enabledProviders.map(p => p.id)).toEqual(['p1', 'p3'])
    })
  })
})
