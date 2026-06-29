import { mount } from '@vue/test-utils'
import { describe, it, expect } from 'vitest'
import naive from 'naive-ui'
import Sidebar from '@/components/chat/Sidebar.vue'
import type { Conversation } from '@/types'

const sampleConversations: Conversation[] = [
  { id: 'c1', title: '对话一', model: 'gpt-3.5-turbo', system_prompt: '', created_at: 1000, updated_at: 1001 },
  { id: 'c2', title: '对话二', model: 'gpt-4', system_prompt: '', created_at: 1002, updated_at: 1003 },
  { id: 'c3', title: '测试对话', model: 'deepseek-chat', system_prompt: '', created_at: 1004, updated_at: 1005 },
]

function createWrapper(props: {
  conversations?: Conversation[]
  activeId?: string | null
} = {}) {
  return mount(Sidebar, {
    props: {
      conversations: [],
      activeId: null,
      ...props,
    },
    global: { plugins: [naive] },
  })
}

describe('Sidebar.vue', () => {
  it('点击"新建对话"按钮触发 create 事件', async () => {
    const wrapper = createWrapper()
    const buttons = wrapper.findAll('button')
    const createBtn = buttons.find(b => b.text().includes('新建对话'))
    expect(createBtn).toBeTruthy()
    await createBtn!.trigger('click')

    expect(wrapper.emitted('create')).toBeTruthy()
  })

  it('传入对话列表后渲染列表文本', () => {
    const wrapper = createWrapper({ conversations: sampleConversations })
    expect(wrapper.text()).toContain('对话一')
    expect(wrapper.text()).toContain('对话二')
    expect(wrapper.text()).toContain('测试对话')
  })
})
