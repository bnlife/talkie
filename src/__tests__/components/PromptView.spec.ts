import { mount, flushPromises } from '@vue/test-utils'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import PromptView from '@/pages/prompt/PromptView.vue'
import { usePromptStore } from '@/stores/promptStore'
import type { Prompt } from '@/types'
import * as promptBridge from '@/bridge/prompt'

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: () => ({
    isMaximized: vi.fn().mockResolvedValue(false),
    minimize: vi.fn(),
    maximize: vi.fn(),
    unmaximize: vi.fn(),
    close: vi.fn(),
  }),
}))

vi.mock('@/bridge/prompt', () => ({
  listPrompts: vi.fn().mockResolvedValue([]),
  createPrompt: vi.fn(),
  updatePrompt: vi.fn().mockResolvedValue(undefined),
  deletePrompt: vi.fn().mockResolvedValue(undefined),
  setDefaultPrompt: vi.fn().mockResolvedValue(undefined),
}))

vi.mock('@/bridge/log', () => ({
  log: vi.fn().mockResolvedValue(undefined),
}))

function createPrompt(overrides: Partial<Prompt> = {}): Prompt {
  return {
    id: 'prompt-1',
    name: '翻译助手',
    content: '你是一个翻译助手',
    is_default: false,
    created_at: 1000,
    updated_at: 1000,
    ...overrides,
  }
}

describe('PromptView.vue', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    vi.mocked(promptBridge.listPrompts).mockResolvedValue([])
  })

  it('初始状态显示提示信息', async () => {
    const wrapper = mount(PromptView)
    await flushPromises()

    expect(wrapper.text()).toContain('选择或创建一个提示词模板')
  })

  it('点击新建提示词按钮显示编辑区域', async () => {
    const wrapper = mount(PromptView)
    await flushPromises()

    // 点击"新建提示词"区域
    const createArea = wrapper.find('.border-dashed')
    await createArea.trigger('click')
    await wrapper.vm.$nextTick()

    expect(wrapper.text()).toContain('模板名称')
    expect(wrapper.text()).toContain('提示词内容')
  })

  it('点击新建后输入名称和内容，点击保存创建新模板', async () => {
    vi.mocked(promptBridge.createPrompt).mockResolvedValue({
      id: 'new-id',
      name: '写作助手',
      content: '你是一个写作助手',
      is_default: false,
      created_at: Date.now(),
      updated_at: Date.now(),
    })

    const promptStore = usePromptStore()
    const wrapper = mount(PromptView)
    await flushPromises()

    // 点击新建
    const createArea = wrapper.find('.border-dashed')
    await createArea.trigger('click')
    await wrapper.vm.$nextTick()

    // 输入名称
    const nameInput = wrapper.find('#prompt-name')
    await nameInput.setValue('写作助手')

    // 输入内容
    const contentTextarea = wrapper.find('#prompt-content')
    await contentTextarea.setValue('你是一个写作助手')

    // 点击保存
    const saveButton = wrapper.findAll('button').find(b => b.text().includes('保存'))
    expect(saveButton).toBeTruthy()
    await saveButton!.trigger('click')
    await flushPromises()

    expect(promptBridge.createPrompt).toHaveBeenCalledWith('写作助手', '你是一个写作助手')
    expect(promptStore.prompts).toHaveLength(1)
  })

  it('选择已有模板显示其内容', async () => {
    vi.mocked(promptBridge.listPrompts).mockResolvedValue([createPrompt()])

    const wrapper = mount(PromptView)
    await flushPromises()

    // 点击模板（在 sidebar 的 div 中）
    const promptItem = wrapper.find('.truncate')
    expect(promptItem).toBeTruthy()
    await promptItem!.trigger('click')
    await wrapper.vm.$nextTick()

    // 验证输入框显示内容
    const nameInput = wrapper.find('#prompt-name')
    expect((nameInput.element as HTMLInputElement).value).toBe('翻译助手')

    const contentTextarea = wrapper.find('#prompt-content')
    expect((contentTextarea.element as HTMLTextAreaElement).value).toBe('你是一个翻译助手')
  })

  it('点击删除按钮删除模板', async () => {
    vi.mocked(promptBridge.listPrompts).mockResolvedValue([createPrompt()])

    const promptStore = usePromptStore()
    const wrapper = mount(PromptView)
    await flushPromises()

    // 选择模板
    const promptItem = wrapper.find('.truncate')
    expect(promptItem).toBeTruthy()
    await promptItem!.trigger('click')
    await wrapper.vm.$nextTick()

    // 点击删除按钮
    const deleteButton = wrapper.findAll('button').find(b => b.text().includes('删除'))
    expect(deleteButton).toBeTruthy()
    await deleteButton!.trigger('click')
    await flushPromises()

    expect(promptBridge.deletePrompt).toHaveBeenCalledWith('prompt-1')
    expect(promptStore.prompts).toHaveLength(0)
  })

  it('点击设为默认按钮设置默认模板', async () => {
    vi.mocked(promptBridge.listPrompts).mockResolvedValue([createPrompt({ is_default: false })])

    const promptStore = usePromptStore()
    const wrapper = mount(PromptView)
    await flushPromises()

    // 选择模板
    const promptItem = wrapper.find('.truncate')
    expect(promptItem).toBeTruthy()
    await promptItem!.trigger('click')
    await wrapper.vm.$nextTick()

    // 点击设为默认按钮
    const defaultButton = wrapper.findAll('button').find(b => b.text().includes('设为默认'))
    expect(defaultButton).toBeTruthy()
    await defaultButton!.trigger('click')
    await flushPromises()

    expect(promptBridge.setDefaultPrompt).toHaveBeenCalledWith('prompt-1')
    expect(promptStore.prompts[0].is_default).toBe(true)
  })
})
