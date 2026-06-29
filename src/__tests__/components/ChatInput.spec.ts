import { mount } from '@vue/test-utils'
import { describe, it, expect } from 'vitest'
import naive from 'naive-ui'
import ChatInput from '@/components/chat/ChatInput.vue'

function createWrapper(props: { disabled?: boolean; streaming?: boolean } = {}) {
  return mount(ChatInput, {
    props: {
      disabled: false,
      streaming: false,
      ...props,
    },
    global: { plugins: [naive] },
  })
}

describe('ChatInput.vue', () => {
  it('disabled=true 时发送按钮禁用', () => {
    const wrapper = createWrapper({ disabled: true })
    const buttons = wrapper.findAll('button')
    // 当 streaming=false 时，只渲染"发送"按钮
    expect(buttons.length).toBe(1)
    expect(buttons[0].attributes('disabled')).toBeDefined()
  })

  it('输入文字后按 Enter 触发 send 事件，值正确', async () => {
    const wrapper = createWrapper()
    const textarea = wrapper.find('textarea')
    await textarea.setValue('你好，世界！')
    await textarea.trigger('keydown', { key: 'Enter', shiftKey: false })

    expect(wrapper.emitted('send')).toBeTruthy()
    expect(wrapper.emitted('send')![0]).toEqual(['你好，世界！'])
  })

  it('streaming=true 时显示"停止生成"按钮，点击触发 stop-stream', async () => {
    const wrapper = createWrapper({ streaming: true })
    const buttons = wrapper.findAll('button')
    expect(buttons.length).toBe(1)
    expect(buttons[0].text()).toContain('停止生成')

    await buttons[0].trigger('click')
    expect(wrapper.emitted('stop-stream')).toBeTruthy()
  })

  it('输入空文字（仅空白字符）不触发 send', async () => {
    const wrapper = createWrapper()
    const textarea = wrapper.find('textarea')
    await textarea.setValue('   ')
    await textarea.trigger('keydown', { key: 'Enter', shiftKey: false })

    expect(wrapper.emitted('send')).toBeFalsy()
  })
})
