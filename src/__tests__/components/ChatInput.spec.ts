import { mount } from '@vue/test-utils'
import { describe, it, expect } from 'vitest'
import ChatInput from '../../components/chat/ChatInput.vue'

function createWrapper(props: { disabled?: boolean; streaming?: boolean } = {}) {
  return mount(ChatInput, {
    props: {
      disabled: false,
      streaming: false,
      ...props,
    },
  })
}

describe('ChatInput.vue', () => {
  it('disabled=true 时发送按钮禁用', () => {
    const wrapper = createWrapper({ disabled: true })
    const buttons = wrapper.findAll('button')
    const sendBtn = buttons.find(b => b.text().includes('发送'))
    expect(sendBtn).toBeDefined()
    if (sendBtn) {
      expect(sendBtn.attributes('disabled')).toBeDefined()
    }
  })

  it('输入文字后按 Enter 触发 send 事件，值正确', async () => {
    const wrapper = createWrapper()
    const textarea = wrapper.find('textarea')
    await textarea.setValue('你好')
    await textarea.trigger('keydown', { key: 'Enter', shiftKey: false })
    const emitted = wrapper.emitted('send')
    expect(emitted).toBeTruthy()
    if (emitted) {
      expect(emitted[0]).toEqual(['你好'])
    }
  })

  it('streaming=true 时显示停止按钮，点击触发 stop-stream', async () => {
    const wrapper = createWrapper({ streaming: true })
    const stopBtn = wrapper.find('button[class*="destructive"]')
    expect(stopBtn.exists()).toBe(true)
    await stopBtn.trigger('click')
    expect(wrapper.emitted('stop-stream')).toBeTruthy()
  })

  it('输入空文字（仅空白字符）不触发 send', async () => {
    const wrapper = createWrapper()
    const textarea = wrapper.find('textarea')
    await textarea.setValue('  ')
    await textarea.trigger('keydown', { key: 'Enter', shiftKey: false })
    expect(wrapper.emitted('send')).toBeFalsy()
  })
})
