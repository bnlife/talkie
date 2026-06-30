import { mount } from '@vue/test-utils'
import { describe, it, expect } from 'vitest'
import ChatInput from '../../pages/chat/ChatInput.vue'

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
    // streaming=false 时只渲染发送按钮（icon-only，无文本）
    const sendBtn = wrapper.find('button')
    expect(sendBtn.exists()).toBe(true)
    expect(sendBtn.attributes('disabled')).toBeDefined()
  })

  it('输入文字后按 Enter 触发 send 事件，值正确', async () => {
    const wrapper = createWrapper()
    // 找到原生 textarea，设置值并触发 input 事件更新 v-model
    const textarea = wrapper.find('textarea')
    const textareaEl = textarea.element as HTMLTextAreaElement
    textareaEl.value = '你好'
    await textarea.trigger('input')
    await wrapper.vm.$nextTick()
    // @keydown 绑定在 <Textarea> 组件上，会透传到原生 textarea
    await textarea.trigger('keydown', { key: 'Enter', shiftKey: false })
    const emitted = wrapper.emitted('send')
    expect(emitted).toBeTruthy()
    if (emitted) {
      expect(emitted[0]).toEqual(['你好'])
    }
  })

  it('streaming=true 时显示停止按钮，点击触发 stop-stream', async () => {
    const wrapper = createWrapper({ streaming: true })
    // streaming=true 时只渲染停止按钮（t-button variant="outline" shape="square"）
    const stopBtn = wrapper.find('button')
    expect(stopBtn.exists()).toBe(true)
    await stopBtn.trigger('click')
    expect(wrapper.emitted('stop-stream')).toBeTruthy()
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
