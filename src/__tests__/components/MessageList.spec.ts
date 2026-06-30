import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { describe, it, expect } from 'vitest'
import MessageList from '@/pages/chat/MessageList.vue'
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

  it('有消息时渲染相应数量', async () => {
    const wrapper = createWrapper()
    const chatStore = useChatStore()
    chatStore.messages = [...sampleMessages]
    await wrapper.vm.$nextTick()
    expect(wrapper.text()).toContain('你好')
    expect(wrapper.text()).toContain('你好！有什么可以帮助你的吗？')
  })

  it('streamingId 有值时显示流式内容', async () => {
    const wrapper = createWrapper()
    const chatStore = useChatStore()
    chatStore.messages = [...sampleMessages]
    chatStore.streamingId = 'stream-1'
    chatStore.streamingContent = '正在生成'
    await wrapper.vm.$nextTick()
    expect(wrapper.text()).toContain('正在生成')
  })
})
