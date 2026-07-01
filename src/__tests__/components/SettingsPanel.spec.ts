import { mount } from '@vue/test-utils'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import SettingsPanel from '@/pages/settings/SettingsPanel.vue'
import { useSettingsStore } from '@/stores/settingsStore'
import type { ModelProvider } from '@/types'

vi.mock('@/bridge/settings')
vi.mock('@/bridge/log')

const defaultProvider: ModelProvider = {
  id: 'prov-1',
  name: 'DeepSeek',
  icon: 'Sparkles',
  base_url: 'https://api.deepseek.com/v1',
  api_key: 'sk-test',
  headers: {},
  models: ['deepseek-chat', 'deepseek-coder'],
  enabled: true,
}

function createWrapper(provider: ModelProvider = defaultProvider) {
  return mount(SettingsPanel, {
    props: { provider },
  })
}

describe('SettingsPanel.vue', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('renders provider name in input', () => {
    const wrapper = createWrapper()
    const nameInput = wrapper.find('input[class*="font-medium"]')
    expect((nameInput.element as HTMLInputElement).value).toBe('DeepSeek')
  })

  it('renders model list', () => {
    const wrapper = createWrapper()
    expect(wrapper.text()).toContain('deepseek-chat')
    expect(wrapper.text()).toContain('deepseek-coder')
  })

  it('renders API key input', () => {
    const wrapper = createWrapper()
    const apiKeyInput = wrapper.find('input[type="password"]')
    expect(apiKeyInput.exists()).toBe(true)
  })

  it('renders base URL input', () => {
    const wrapper = createWrapper()
    expect(wrapper.text()).toContain('https://api.deepseek.com/v1')
  })

  it('修改名称后 blur 调用 updateProvider', async () => {
    const wrapper = createWrapper()
    const settingsStore = useSettingsStore()
    settingsStore.updateProvider = vi.fn().mockResolvedValue(undefined)

    const nameInput = wrapper.find('input[class*="font-medium"]')
    await nameInput.setValue('New Name')
    await nameInput.trigger('blur')

    expect(settingsStore.updateProvider).toHaveBeenCalledWith('prov-1', { name: 'New Name' })
  })

  it('修改名称后 Enter 调用 updateProvider', async () => {
    const wrapper = createWrapper()
    const settingsStore = useSettingsStore()
    settingsStore.updateProvider = vi.fn().mockResolvedValue(undefined)

    const nameInput = wrapper.find('input[class*="font-medium"]')
    await nameInput.setValue('New Name')
    await nameInput.trigger('keyup.enter')

    expect(settingsStore.updateProvider).toHaveBeenCalledWith('prov-1', { name: 'New Name' })
  })

  it('点击 toggle 切换 enabled 状态', async () => {
    const wrapper = createWrapper()
    const settingsStore = useSettingsStore()
    settingsStore.updateProvider = vi.fn().mockResolvedValue(undefined)

    const toggle = wrapper.find('button[role="switch"]')
    await toggle.trigger('click')

    expect(settingsStore.updateProvider).toHaveBeenCalledWith('prov-1', { enabled: false })
  })

  it('点击拉取模型调用 fetchModels', async () => {
    const wrapper = createWrapper()
    const settingsStore = useSettingsStore()
    settingsStore.fetchModels = vi.fn().mockResolvedValue(undefined)

    const fetchBtn = wrapper.findAll('button').find(b => b.text().includes('拉取'))
    if (fetchBtn) {
      await fetchBtn.trigger('click')
      expect(settingsStore.fetchModels).toHaveBeenCalledWith('prov-1')
    }
  })
})
