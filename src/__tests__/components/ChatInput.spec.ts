import { mount } from '@vue/test-utils'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import ChatInput from '../../pages/chat/ChatInput.vue'

vi.mock('@/bridge/settings')
vi.mock('@/bridge/conversation')
vi.mock('@/bridge/chat')
vi.mock('@/bridge/log')
vi.mock('vue-sonner', () => ({
  toast: { success: vi.fn(), error: vi.fn(), warning: vi.fn(), info: vi.fn() },
}))

function createWrapper(props: { disabled?: boolean; streaming?: boolean } = {}) {
  return mount(ChatInput, {
    props: {
      disabled: false,
      streaming: false,
      ...props,
    },
  })
}

function makeFile(name: string, content: string, type = ''): File {
  return new File([content], name, { type })
}

describe('ChatInput.vue', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('disabled=true 时发送按钮禁用', () => {
    const wrapper = createWrapper({ disabled: true })
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
    expect(emitted![0][0]).toBe('你好')
    expect(emitted![0][1]).toBe('你好')
  })

  it('streaming=true 时显示停止按钮，点击触发 stop-stream', async () => {
    const wrapper = createWrapper({ streaming: true })
    const buttons = wrapper.findAll('button')
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
      id: 'c1', title: '对话一', conversation_id: 'c1', provider_id: 'prov-1', model: 'gpt-4',
      prompt_id: null, search_enabled: false, search_engine: '', created_at: 1000, updated_at: 1001, pinned: false,
    }]
    chatStore.activeConversationId = 'c1'

    const wrapper1 = mount(ChatInput, { props: { disabled: false, streaming: false } })
    const textarea1 = wrapper1.find('textarea')
    const el1 = textarea1.element as HTMLTextAreaElement
    el1.value = '草稿内容'
    await textarea1.trigger('input')
    await wrapper1.vm.$nextTick()

    expect(chatStore.getDraft('c1')).toBe('草稿内容')

    wrapper1.unmount()
    const wrapper2 = mount(ChatInput, { props: { disabled: false, streaming: false } })
    const textarea2 = wrapper2.find('textarea')
    const el2 = textarea2.element as HTMLTextAreaElement

    expect(el2.value).toBe('草稿内容')

    wrapper2.unmount()
  })

  it('附件按钮存在且可点击', () => {
    const wrapper = createWrapper()
    const attachBtn = wrapper.find('button svg.lucide-paperclip')
    expect(attachBtn.exists()).toBe(true)
  })

  it('disabled=true 时附件按钮禁用', () => {
    const wrapper = createWrapper({ disabled: true })
    const buttons = wrapper.findAll('button')
    const attachBtn = buttons.find(b => b.find('.lucide-paperclip').exists())
    expect(attachBtn).toBeDefined()
    expect(attachBtn!.attributes('disabled')).toBeDefined()
  })

  it('有附件 + 空文字时发送按钮可用', async () => {
    const wrapper = createWrapper()
    const vm = wrapper.vm as any

    vm.addFiles([makeFile('test.txt', 'hello', 'text/plain')])
    await wrapper.vm.$nextTick()

    const sendBtn = wrapper.find('button svg.lucide-send')
    expect(sendBtn.exists()).toBe(true)
    const btn = sendBtn.element.closest('button') as HTMLButtonElement
    expect(btn.disabled).toBe(false)
  })

  it('点击删除按钮移除附件', async () => {
    const wrapper = createWrapper()
    const vm = wrapper.vm as any

    vm.addFiles([makeFile('a.txt', 'aaa', 'text/plain'), makeFile('b.txt', 'bbb', 'text/plain')])
    await wrapper.vm.$nextTick()
    expect(wrapper.text()).toContain('a.txt')
    expect(wrapper.text()).toContain('b.txt')

    const removeBtns = wrapper.findAll('button svg.lucide-x')
    expect(removeBtns.length).toBe(2)
    await removeBtns[0].element.closest('button')!.click()
    await wrapper.vm.$nextTick()

    expect(wrapper.text()).not.toContain('a.txt')
    expect(wrapper.text()).toContain('b.txt')
  })

  it('有附件时发送事件包含附件元数据', async () => {
    const wrapper = createWrapper()
    const vm = wrapper.vm as any

    vm.addFiles([makeFile('code.rs', 'fn main() {}', 'text/plain')])
    await wrapper.vm.$nextTick()

    const textarea = wrapper.find('textarea')
    const textareaEl = textarea.element as HTMLTextAreaElement
    textareaEl.value = '分析代码'
    await textarea.trigger('input')
    await wrapper.vm.$nextTick()

    await textarea.trigger('keydown', { key: 'Enter', shiftKey: false })
    await new Promise(r => setTimeout(r, 50))
    await wrapper.vm.$nextTick()

    const emitted = wrapper.emitted('send')
    expect(emitted).toBeTruthy()
    expect(emitted![0][0]).toBe('分析代码')
    expect(emitted![0][1]).toContain('分析代码')
    expect(emitted![0][1]).toContain('📎 附件: code.rs')
    expect(emitted![0][2]).toEqual([{ name: 'code.rs', size: 12, content: 'fn main() {}' }])
  })

  it('无附件时发送事件 attachments 为 undefined', async () => {
    const wrapper = createWrapper()

    const textarea = wrapper.find('textarea')
    const textareaEl = textarea.element as HTMLTextAreaElement
    textareaEl.value = '纯文字'
    await textarea.trigger('input')
    await wrapper.vm.$nextTick()

    await textarea.trigger('keydown', { key: 'Enter', shiftKey: false })

    const emitted = wrapper.emitted('send')
    expect(emitted).toBeTruthy()
    expect(emitted![0][0]).toBe('纯文字')
    expect(emitted![0][1]).toBe('纯文字')
    expect(emitted![0][2]).toBeUndefined()
  })
})
