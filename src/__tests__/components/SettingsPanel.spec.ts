import { mount } from '@vue/test-utils'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import SettingsPanel from '@/pages/settings/SettingsPanel.vue'
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
})
