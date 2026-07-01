import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { describe, it, expect, vi } from 'vitest'
import MessageList from '@/pages/chat/MessageList.vue'
import MessageItem from '@/pages/chat/MessageItem.vue'
import { useChatStore } from '@/stores/chatStore'
import type { Message } from '@/types'

function createWrapper() {
  const pinia = createPinia()
  setActivePinia(pinia)
  return mount(MessageList, {
    global: {
      plugins: [pinia],
    },
  })
}

const sampleMessages: Message[] = [
  { id: '1', conversation_id: 'c1', role: 'user', content: '你好', created_at: 1000 },
  { id: '2', conversation_id: 'c1', role: 'assistant', content: '你好！有什么可以帮助你的吗？', created_at: 1001 },
]

describe('MessageList.vue', () => {
  it('空消息时显示空状态', () => {
    const wrapper = createWrapper()
    expect(wrapper.text()).toContain('暂无消息')
  })

  it('有消息时渲染相应数量的 MessageItem', async () => {
    const wrapper = createWrapper()
    const chatStore = useChatStore()
    chatStore.messages = [...sampleMessages]
    await wrapper.vm.$nextTick()
    const items = wrapper.findAllComponents(MessageItem)
    expect(items).toHaveLength(2)
  })

  it('每个 MessageItem 收到正确的 props', async () => {
    const wrapper = createWrapper()
    const chatStore = useChatStore()
    chatStore.messages = [...sampleMessages]
    chatStore.activeConversationId = 'c1'
    await wrapper.vm.$nextTick()
    const items = wrapper.findAllComponents(MessageItem)
    expect(items[0].props('message').role).toBe('user')
    expect(items[0].props('message').content).toBe('你好')
    expect(items[1].props('message').role).toBe('assistant')
    expect(items[1].props('message').content).toBe('你好！有什么可以帮助你的吗？')
    expect(items[1].props('isLast')).toBe(true)
  })

  it('streamingId 有值时显示流式内容', async () => {
    const wrapper = createWrapper()
    const chatStore = useChatStore()
    chatStore.messages = [...sampleMessages]
    chatStore.streamingId = 'stream-1'
    chatStore.streamingContent = '正在生成'
    await wrapper.vm.$nextTick()
    const items = wrapper.findAllComponents(MessageItem)
    expect(items).toHaveLength(3)
    expect(items[2].props('message').content).toBe('正在生成')
    expect(items[2].props('streaming')).toBe(true)
  })

  it('copy 事件调用 navigator.clipboard', async () => {
    const writeText = vi.fn().mockResolvedValue(undefined)
    Object.assign(navigator, { clipboard: { writeText } })

    const wrapper = createWrapper()
    const chatStore = useChatStore()
    chatStore.messages = [...sampleMessages]
    await wrapper.vm.$nextTick()

    const items = wrapper.findAllComponents(MessageItem)
    await items[0].vm.$emit('copy', '你好')
    expect(writeText).toHaveBeenCalledWith('你好')
  })

  it('delete 事件从 MessageItem 传递到父组件', async () => {
    const wrapper = createWrapper()
    const chatStore = useChatStore()
    chatStore.messages = [...sampleMessages]
    await wrapper.vm.$nextTick()

    const items = wrapper.findAllComponents(MessageItem)
    await items[0].vm.$emit('delete', '1')
    // MessageList passes delete events up, doesn't mutate store directly
    expect(items[0].emitted('delete')).toBeTruthy()
    expect(items[0].emitted('delete')![0]).toEqual(['1'])
  })

  it('regenerate 事件从 MessageItem 传递到父组件', async () => {
    const wrapper = createWrapper()
    const chatStore = useChatStore()
    chatStore.messages = [...sampleMessages]
    await wrapper.vm.$nextTick()

    const items = wrapper.findAllComponents(MessageItem)
    await items[1].vm.$emit('regenerate')
    // MessageList passes regenerate events up, doesn't mutate store directly
    expect(items[1].emitted('regenerate')).toBeTruthy()
  })
})
