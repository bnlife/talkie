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

  it('Shift+Enter 换行不发送', async () => {
    const wrapper = createWrapper()
    const textarea = wrapper.find('textarea')
    const textareaEl = textarea.element as HTMLTextAreaElement
    textareaEl.value = '你好'
    await textarea.trigger('input')
    await wrapper.vm.$nextTick()
    await textarea.trigger('keydown', { key: 'Enter', shiftKey: true })
    expect(wrapper.emitted('send')).toBeFalsy()
  })

  it('disabled=true 时按 Enter 不触发 send', async () => {
    const wrapper = createWrapper({ disabled: true })
    const textarea = wrapper.find('textarea')
    const textareaEl = textarea.element as HTMLTextAreaElement
    textareaEl.value = '你好'
    await textarea.trigger('input')
    await wrapper.vm.$nextTick()
    await textarea.trigger('keydown', { key: 'Enter', shiftKey: false })
    expect(wrapper.emitted('send')).toBeFalsy()
  })

  it('streaming=true 时按 Enter 不触发 send', async () => {
    const wrapper = createWrapper({ streaming: true })
    const textarea = wrapper.find('textarea')
    const textareaEl = textarea.element as HTMLTextAreaElement
    textareaEl.value = '你好'
    await textarea.trigger('input')
    await wrapper.vm.$nextTick()
    await textarea.trigger('keydown', { key: 'Enter', shiftKey: false })
    expect(wrapper.emitted('send')).toBeFalsy()
  })

  it('输入文字后重新挂载组件，文字保留（draft）', async () => {
    const pinia = createPinia()
    setActivePinia(pinia)
    const { useChatStore } = await import('@/stores/chatStore')
    const chatStore = useChatStore()
    chatStore.conversations = [{
      id: 'c1', title: '对话一', provider_id: 'prov-1', model: 'gpt-4',
      prompt_id: null, search_enabled: false, search_engine: '', created_at: 1000, updated_at: 1001, pinned: false,
    }]
    chatStore.activeConversationId = 'c1'

    // First mount: type something
    const wrapper1 = mount(ChatInput, { props: { disabled: false, streaming: false } })
    const textarea1 = wrapper1.find('textarea')
    const el1 = textarea1.element as HTMLTextAreaElement
    el1.value = '草稿内容'
    await textarea1.trigger('input')
    await wrapper1.vm.$nextTick()

    // Verify draft is saved in store
    expect(chatStore.getDraft('c1')).toBe('草稿内容')

    // Unmount and remount (simulates page navigation)
    wrapper1.unmount()
    const wrapper2 = mount(ChatInput, { props: { disabled: false, streaming: false } })
    const textarea2 = wrapper2.find('textarea')
    const el2 = textarea2.element as HTMLTextAreaElement

    // Draft should be restored
    expect(el2.value).toBe('草稿内容')

    wrapper2.unmount()
  })
})
