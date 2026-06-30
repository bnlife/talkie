import { mount } from '@vue/test-utils'
import { describe, it, expect } from 'vitest'
import MessageList from '@/components/chat/MessageList.vue'
import type { Message } from '@/types'

function createWrapper(props: {
  messages?: Message[]
  streamingId?: string | null
  streamingContent?: string
} = {}) {
  return mount(MessageList, {
    props: {
      messages: [],
      streamingId: null,
      ...props,
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
    // n-empty 组件渲染的描述文本
    expect(wrapper.text()).toContain('暂无消息')
  })

  it('有消息时渲染相应数量', () => {
    const wrapper = createWrapper({ messages: sampleMessages })
    // 每条消息会渲染一个 MessageItem，MessageItem 内部会显示角色标签和内容
    expect(wrapper.text()).toContain('你好')
    expect(wrapper.text()).toContain('你好！有什么可以帮助你的吗？')
  })

  it('streamingId 有值时显示流式内容', () => {
    const wrapper = createWrapper({
      messages: sampleMessages,
      streamingId: 'stream-1',
      streamingContent: '正在生成',
    })
    expect(wrapper.text()).toContain('正在生成')
  })
})
