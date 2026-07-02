import { mount } from '@vue/test-utils'
import { describe, it, expect } from 'vitest'
import Sidebar from '@/pages/chat/Sidebar.vue'
import type { ConversationView } from '@/types'

const sampleConversations: ConversationView[] = [
  { id: 'c1', title: '对话一', conversation_id: 'c1', provider_id: 'prov-1', model: 'gpt-3.5-turbo', prompt_id: null, search_enabled: false, search_engine: '', created_at: 1000, updated_at: 1001, pinned: false },
  { id: 'c2', title: '对话二', conversation_id: 'c2', provider_id: 'prov-1', model: 'gpt-4', prompt_id: null, search_enabled: false, search_engine: '', created_at: 1002, updated_at: 1003, pinned: false },
  { id: 'c3', title: '测试对话', conversation_id: 'c3', provider_id: 'prov-2', model: 'deepseek-chat', prompt_id: null, search_enabled: false, search_engine: '', created_at: 1004, updated_at: 1005, pinned: false },
]

function createWrapper(props: {
  conversations?: ConversationView[]
  activeId?: string | null
  searchQuery?: string
} = {}) {
  return mount(Sidebar, {
    props: {
      conversations: [],
      activeId: null,
      searchQuery: '',
      ...props,
    },
  })
}

describe('Sidebar.vue', () => {
  it('点击"新建对话"按钮触发 create 事件', async () => {
    const wrapper = createWrapper()
    const createBtn = wrapper.findAll('div').filter(w => w.classes().includes('cursor-pointer'))[0]
    expect(createBtn).toBeDefined()
    await createBtn!.trigger('click')
    expect(wrapper.emitted('create')).toBeTruthy()
  })

  it('传入对话列表后渲染列表文本', () => {
    const wrapper = createWrapper({ conversations: sampleConversations })
    expect(wrapper.text()).toContain('对话一')
    expect(wrapper.text()).toContain('对话二')
    expect(wrapper.text()).toContain('测试对话')
  })

  it('搜索框输入文字 -> 只显示匹配的对话', async () => {
    const wrapper = createWrapper({ conversations: sampleConversations })
    await wrapper.setProps({ searchQuery: '一' })
    await wrapper.vm.$nextTick()
    expect(wrapper.text()).toContain('对话一')
    expect(wrapper.text()).not.toContain('对话二')
    expect(wrapper.text()).not.toContain('测试对话')
  })

  it('搜索框清空 -> 显示全部对话', async () => {
    const wrapper = createWrapper({ conversations: sampleConversations })
    await wrapper.setProps({ searchQuery: '一' })
    await wrapper.vm.$nextTick()
    expect(wrapper.text()).toContain('对话一')
    await wrapper.setProps({ searchQuery: '' })
    await wrapper.vm.$nextTick()
    expect(wrapper.text()).toContain('对话一')
    expect(wrapper.text()).toContain('对话二')
    expect(wrapper.text()).toContain('测试对话')
  })

  it('不匹配的搜索词 -> 不显示任何对话', async () => {
    const wrapper = createWrapper({ conversations: sampleConversations })
    await wrapper.setProps({ searchQuery: 'zzz_not_exists' })
    await wrapper.vm.$nextTick()
    expect(wrapper.text()).not.toContain('对话一')
    expect(wrapper.text()).not.toContain('对话二')
    expect(wrapper.text()).not.toContain('测试对话')
  })

  it('点击对话项触发 select 事件', async () => {
    const wrapper = createWrapper({ conversations: sampleConversations, activeId: 'c1' })
    const convItems = wrapper.findAll('.flex.items-center.justify-between')
    const convItem = convItems.find(i => i.text().includes('对话一'))
    expect(convItem).toBeTruthy()
    if (convItem) {
      await convItem.trigger('click')
      expect(wrapper.emitted('select')).toBeTruthy()
    }
  })

  it('点击删除按钮触发 close 事件', async () => {
    const wrapper = createWrapper({ conversations: sampleConversations, activeId: 'c1' })
    const deleteBtn = wrapper.findAll('button').find(b => b.classes().includes('hover:bg-foreground/5'))
    if (deleteBtn) {
      await deleteBtn.trigger('click')
      expect(wrapper.emitted('close')).toBeTruthy()
    }
  })

  it('右键对话项显示菜单，点击置顶触发 pin 事件', async () => {
    const wrapper = createWrapper({ conversations: sampleConversations, activeId: 'c1' })
    const convItems = wrapper.findAll('.flex.items-center.justify-between')
    const convItem = convItems.find(i => i.text().includes('对话一'))
    expect(convItem).toBeTruthy()
    if (convItem) {
      await convItem.trigger('contextmenu')
      await wrapper.vm.$nextTick()
      const pinBtn = wrapper.findAll('button').find(b => b.text().includes('置顶'))
      if (pinBtn) {
        await pinBtn.trigger('click')
        expect(wrapper.emitted('pin')).toBeTruthy()
      }
    }
  })

  it('右键已置顶对话，菜单显示取消置顶', async () => {
    const pinnedConversations = [{ ...sampleConversations[0], pinned: true }, ...sampleConversations.slice(1)]
    const wrapper = createWrapper({ conversations: pinnedConversations, activeId: 'c1' })
    const convItems = wrapper.findAll('.flex.items-center.justify-between')
    const convItem = convItems.find(i => i.text().includes('对话一'))
    expect(convItem).toBeTruthy()
    if (convItem) {
      await convItem.trigger('contextmenu')
      await wrapper.vm.$nextTick()
      // Context menu uses Teleport to body, so search in document.body
      const unpinBtn = Array.from(document.body.querySelectorAll('button')).find(b => b.textContent?.includes('取消置顶'))
      expect(unpinBtn).toBeTruthy()
    }
  })

  it('右键菜单点击重命名，输入新标题后触发 rename 事件', async () => {
    const wrapper = createWrapper({ conversations: sampleConversations, activeId: 'c1' })
    const convItems = wrapper.findAll('.flex.items-center.justify-between')
    const convItem = convItems.find(i => i.text().includes('对话一'))
    expect(convItem).toBeTruthy()
    if (convItem) {
      await convItem.trigger('contextmenu')
      await wrapper.vm.$nextTick()
      const renameBtn = wrapper.findAll('button').find(b => b.text().includes('重命名'))
      if (renameBtn) {
        await renameBtn.trigger('click')
        await wrapper.vm.$nextTick()
        const input = wrapper.find('input[data-rename-input]')
        expect(input.exists()).toBe(true)
      }
    }
  })

})
