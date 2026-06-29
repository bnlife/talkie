import { mount } from '@vue/test-utils'
import { describe, it, expect } from 'vitest'
import naive from 'naive-ui'
import Sidebar from '@/components/chat/Sidebar.vue'
import type { Conversation } from '@/types'

const sampleConversations: Conversation[] = [
  { id: 'c1', title: '对话一', model: 'gpt-3.5-turbo', system_prompt: '', created_at: 1000, updated_at: 1001, pinned: false },
  { id: 'c2', title: '对话二', model: 'gpt-4', system_prompt: '', created_at: 1002, updated_at: 1003, pinned: false },
  { id: 'c3', title: '测试对话', model: 'deepseek-chat', system_prompt: '', created_at: 1004, updated_at: 1005, pinned: false },
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

  // ======== 搜索框过滤测试 ========

  it('搜索框输入文字 → 只显示匹配的对话', async () => {
    const wrapper = createWrapper({ conversations: sampleConversations })
    const input = wrapper.find('input')
    await input.setValue('一')
    // '一' 只匹配 "对话一"
    expect(wrapper.text()).toContain('对话一')
    expect(wrapper.text()).not.toContain('对话二')
    expect(wrapper.text()).not.toContain('测试对话')
  })

  it('搜索框清空 → 显示全部对话', async () => {
    const wrapper = createWrapper({ conversations: sampleConversations })
    const input = wrapper.find('input')
    // 先过滤，再清空
    await input.setValue('一')
    expect(wrapper.text()).toContain('对话一')
    await input.setValue('')
    // 清空后全部恢复
    expect(wrapper.text()).toContain('对话一')
    expect(wrapper.text()).toContain('对话二')
    expect(wrapper.text()).toContain('测试对话')
  })

  it('不匹配的搜索词 → 不显示任何对话', async () => {
    const wrapper = createWrapper({ conversations: sampleConversations })
    const input = wrapper.find('input')
    await input.setValue('zzz_not_exists')
    expect(wrapper.text()).not.toContain('对话一')
    expect(wrapper.text()).not.toContain('对话二')
    expect(wrapper.text()).not.toContain('测试对话')
  })

  // ======== 事件发射验证 ========
  // 说明：右键上下文菜单使用固定定位 div，jsdom 下能正常渲染
  // 1) 点击对话项触发 select 事件
  // 2) 点击设置按钮触发 open-settings 事件

  it('点击对话项触发 select 事件', async () => {
    const wrapper = createWrapper({ conversations: sampleConversations, activeId: 'c1' })
    // 自定义 div 渲染后，点击 .conv-item 会触发 select
    const convItem = wrapper.find('.conv-item')
    expect(convItem.exists()).toBe(true)
    await convItem.trigger('click')
    expect(wrapper.emitted('select')).toBeTruthy()
  })

  it('点击设置按钮触发 open-settings 事件', async () => {
    const wrapper = createWrapper()
    const settingsBtn = wrapper.findAll('button').find(b => b.text().includes('设置'))
    expect(settingsBtn).toBeTruthy()
    await settingsBtn!.trigger('click')
    expect(wrapper.emitted('open-settings')).toBeTruthy()
  })
})
