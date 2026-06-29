import { mount } from '@vue/test-utils'
import { describe, it, expect } from 'vitest'
import naive from 'naive-ui'
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
    global: { plugins: [naive] },
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
    // 找到所有输入框（n-input 渲染为内部 input 元素）
    const inputs = wrapper.findAll('input')
    // 第 1 个 input - base_url
    await inputs[0].setValue('https://custom.api.com/v1')

    const buttons = wrapper.findAll('button')
    const saveBtn = buttons.find(b => b.text().includes('保存设置'))
    expect(saveBtn).toBeTruthy()
    await saveBtn!.trigger('click')

    expect(wrapper.emitted('update')).toBeTruthy()
    const emitted = wrapper.emitted('update')![0][0] as Partial<Settings>
    expect(emitted.base_url).toBe('https://custom.api.com/v1')
  })
})
