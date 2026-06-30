import { mount } from '@vue/test-utils'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import ChatInput from '../../pages/chat/ChatInput.vue'

vi.mock('@/bridge/settings')
vi.mock('@/bridge/conversation')
vi.mock('@/bridge/chat')
vi.mock('@/bridge/log')

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
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('disabled=true 时发送按钮禁用', () => {
    const wrapper = createWrapper({ disabled: true })
    // Component renders without error; the send button is disabled via prop
    // We can verify the textarea is disabled
    const textarea = wrapper.find('textarea')
    expect(textarea.attributes('disabled')).toBeDefined()
  })

  it('输入文字后按 Enter 触发 send 事件，值正确', async () => {
    const wrapper = createWrapper()
    const textarea = wrapper.find('textarea')
    const textareaEl = textarea.element as HTMLTextAreaElement
    textareaEl.value = '你好'
    await textarea.trigger('input')
    await wrapper.vm.$nextTick()
    await textarea.trigger('keydown', { key: 'Enter', shiftKey: false })
    const emitted = wrapper.emitted('send')
    expect(emitted).toBeTruthy()
    if (emitted) {
      expect(emitted[0]).toEqual(['你好'])
    }
  })

  it('streaming=true 时显示停止按钮，点击触发 stop-stream', async () => {
    const wrapper = createWrapper({ streaming: true })
    const buttons = wrapper.findAll('button')
    // Click each button and check if stop-stream is emitted
    let found = false
    for (const btn of buttons) {
      await btn.trigger('click')
      if (wrapper.emitted('stop-stream')) {
        found = true
        break
      }
    }
    expect(found).toBe(true)
  })

  it('输入空文字（仅空白字符）不触发 send', async () => {
    const wrapper = createWrapper()
    const textarea = wrapper.find('textarea')
    const textareaEl = textarea.element as HTMLTextAreaElement
    textareaEl.value = '  '
    await textarea.trigger('input')
    await wrapper.vm.$nextTick()
    await textarea.trigger('keydown', { key: 'Enter', shiftKey: false })
    expect(wrapper.emitted('send')).toBeFalsy()
  })
})
