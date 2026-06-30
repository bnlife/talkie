import { mount } from '@vue/test-utils'
import { describe, it, expect } from 'vitest'
import SettingsPanel from '@/components/settings/SettingsPanel.vue'
import type { Settings } from '@/types'

const defaultSettings: Settings = {
  base_url: 'https://api.openai.com/v1',
  api_key: 'sk-test',
  model: 'gpt-3.5-turbo',
  temperature: 0.7,
}

function createWrapper(settings: Settings = defaultSettings) {
  return mount(SettingsPanel, {
    props: { settings },
  })
}

describe('SettingsPanel.vue', () => {
  it('点击"测试连接"按钮触发 test-connection 事件', async () => {
    const wrapper = createWrapper()
    const buttons = wrapper.findAll('button')
    // 查找文本包含"测试连接"的按钮
    const testBtn = buttons.find(b => b.text().includes('测试连接'))
    expect(testBtn).toBeTruthy()
    await testBtn!.trigger('click')

    expect(wrapper.emitted('test-connection')).toBeTruthy()
  })

  it('修改值后点击"保存"触发 update 事件，携带修改后的值', async () => {
    const wrapper = createWrapper()
    // 直接设置原生 input 值并触发 input 事件，确保通过 useVModel 传播到 form 响应式状态
    const input = wrapper.findAll('input')[0]
    const nativeInput = input.element as HTMLInputElement
    nativeInput.value = 'https://custom.api.com/v1'
    await input.trigger('input')
    await wrapper.vm.$nextTick()

    // 提交表单（保存按钮 type="submit" 在 jsdom 中点击不会自动触发 form submit）
    const form = wrapper.find('form')
    await form.trigger('submit')

    expect(wrapper.emitted('update')).toBeTruthy()
    const emitted = wrapper.emitted('update')![0][0] as Partial<Settings>
    expect(emitted.base_url).toBe('https://custom.api.com/v1')
  })
})
