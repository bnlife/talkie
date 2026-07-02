import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { usePromptStore } from '../../stores/promptStore'
import * as promptBridge from '../../bridge/prompt'
import { createPrompt } from './helpers'

vi.mock('../../bridge/prompt')

describe('promptStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  describe('loadPrompts', () => {
    it('calls listPrompts and sets prompts state', async () => {
      const prompts = [createPrompt()]
      vi.mocked(promptBridge.listPrompts).mockResolvedValue(prompts)

      const store = usePromptStore()
      await store.loadPrompts()

      expect(promptBridge.listPrompts).toHaveBeenCalledOnce()
      expect(store.prompts).toEqual(prompts)
    })
  })

  describe('createPrompt', () => {
    it('calls bridge, prepends prompt, and sets active', async () => {
      const newPrompt = createPrompt({ id: 'new-id', name: '写作助手' })
      vi.mocked(promptBridge.createPrompt).mockResolvedValue(newPrompt)

      const store = usePromptStore()
      store.prompts = [createPrompt({ id: 'old-id' })]

      const result = await store.createPrompt('写作助手', '你是一个写作助手')

      expect(promptBridge.createPrompt).toHaveBeenCalledWith('写作助手', '你是一个写作助手')
      expect(store.prompts).toHaveLength(2)
      expect(store.prompts[0].id).toBe('new-id')
      expect(store.activePromptId).toBe('new-id')
      expect(result).toEqual(newPrompt)
    })
  })

  describe('updatePrompt', () => {
    it('calls bridge and updates local state', async () => {
      vi.mocked(promptBridge.updatePrompt).mockResolvedValue(undefined)

      const store = usePromptStore()
      store.prompts = [createPrompt({ id: 'prompt-1', name: '旧名称', content: '旧内容' })]

      await store.updatePrompt('prompt-1', '新名称', '新内容')

      expect(promptBridge.updatePrompt).toHaveBeenCalledWith('prompt-1', '新名称', '新内容')
      expect(store.prompts[0].name).toBe('新名称')
      expect(store.prompts[0].content).toBe('新内容')
    })
  })

  describe('deletePrompt', () => {
    it('removes prompt and clears active when active', async () => {
      vi.mocked(promptBridge.deletePrompt).mockResolvedValue(undefined)

      const store = usePromptStore()
      store.prompts = [createPrompt({ id: 'p1' }), createPrompt({ id: 'p2' })]
      store.activePromptId = 'p1'

      await store.deletePrompt('p1')

      expect(promptBridge.deletePrompt).toHaveBeenCalledWith('p1')
      expect(store.prompts.map(p => p.id)).toEqual(['p2'])
      expect(store.activePromptId).toBeNull()
    })

    it('does not clear active when deleting inactive prompt', async () => {
      vi.mocked(promptBridge.deletePrompt).mockResolvedValue(undefined)

      const store = usePromptStore()
      store.prompts = [createPrompt({ id: 'p1' }), createPrompt({ id: 'p2' })]
      store.activePromptId = 'p2'

      await store.deletePrompt('p1')

      expect(store.activePromptId).toBe('p2')
    })
  })

  describe('setDefaultPrompt', () => {
    it('calls bridge and updates local is_default flags', async () => {
      vi.mocked(promptBridge.setDefaultPrompt).mockResolvedValue(undefined)

      const store = usePromptStore()
      store.prompts = [
        createPrompt({ id: 'p1', is_default: true }),
        createPrompt({ id: 'p2', is_default: false }),
      ]

      await store.setDefaultPrompt('p2')

      expect(promptBridge.setDefaultPrompt).toHaveBeenCalledWith('p2')
      expect(store.prompts[0].is_default).toBe(false)
      expect(store.prompts[1].is_default).toBe(true)
    })
  })

  describe('selectPrompt', () => {
    it('sets activePromptId', () => {
      const store = usePromptStore()
      store.selectPrompt('prompt-1')
      expect(store.activePromptId).toBe('prompt-1')
    })

    it('can set activePromptId to null', () => {
      const store = usePromptStore()
      store.activePromptId = 'prompt-1'
      store.selectPrompt(null)
      expect(store.activePromptId).toBeNull()
    })
  })

  describe('getters', () => {
    it('activePrompt returns the active prompt', () => {
      const store = usePromptStore()
      store.prompts = [createPrompt({ id: 'p1' }), createPrompt({ id: 'p2' })]
      store.activePromptId = 'p2'

      expect(store.activePrompt?.id).toBe('p2')
    })

    it('activePrompt returns undefined when no active', () => {
      const store = usePromptStore()
      store.prompts = [createPrompt({ id: 'p1' })]
      store.activePromptId = null

      expect(store.activePrompt).toBeUndefined()
    })

    it('defaultPrompt returns the default prompt', () => {
      const store = usePromptStore()
      store.prompts = [
        createPrompt({ id: 'p1', is_default: false }),
        createPrompt({ id: 'p2', is_default: true }),
      ]

      expect(store.defaultPrompt?.id).toBe('p2')
    })

    it('defaultPrompt returns undefined when no default', () => {
      const store = usePromptStore()
      store.prompts = [
        createPrompt({ id: 'p1', is_default: false }),
        createPrompt({ id: 'p2', is_default: false }),
      ]

      expect(store.defaultPrompt).toBeUndefined()
    })
  })
})
