import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import type { Conversation, Message, Settings, Prompt, ModelProvider } from '../types'

// Mock bridge modules before importing stores
vi.mock('../bridge/chat')
vi.mock('../bridge/conversation')
vi.mock('../bridge/settings')
vi.mock('../bridge/log')
vi.mock('../bridge/prompt')

import { useChatStore } from '../stores/chatStore'
import { useSettingsStore } from '../stores/settingsStore'
import { usePromptStore } from '../stores/promptStore'
import * as chatBridge from '../bridge/chat'
import * as conversationBridge from '../bridge/conversation'
import * as settingsBridge from '../bridge/settings'
import * as promptBridge from '../bridge/prompt'

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------
function createConv(overrides: Partial<Conversation> = {}): Conversation {
  return {
    id: 'conv-1',
    title: 'Test',
    provider_id: 'prov-1',
    model: 'deepseek-chat',
    system_prompt: '',
    created_at: 0,
    updated_at: 0,
    pinned: false,
    ...overrides,
  }
}

function createMsg(overrides: Partial<Message> = {}): Message {
  return {
    id: 'msg-1',
    conversation_id: 'conv-1',
    role: 'user',
    content: 'hello',
    created_at: 100,
    ...overrides,
  }
}

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

function createProvider(overrides: Partial<ModelProvider> = {}): ModelProvider {
  return {
    id: 'prov-1',
    name: 'DeepSeek',
    icon: 'Sparkles',
    base_url: 'https://api.deepseek.com/v1',
    api_key: 'sk-test',
    headers: {},
    models: ['deepseek-chat'],
    enabled: true,
    ...overrides,
  }
}

// ---------------------------------------------------------------------------
// chatStore
// ---------------------------------------------------------------------------
describe('chatStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  describe('loadConversations', () => {
    it('calls listConversations and sets conversations state', async () => {
      const convs = [createConv()]
      vi.mocked(conversationBridge.listConversations).mockResolvedValue(convs)

      const store = useChatStore()
      await store.loadConversations()

      expect(conversationBridge.listConversations).toHaveBeenCalledOnce()
      expect(store.conversations).toEqual(convs)
    })

    it('restores last active conversation when last_active_conversation_id exists', async () => {
      const conv = createConv({ id: 'last-id' })
      vi.mocked(conversationBridge.listConversations).mockResolvedValue([conv])
      vi.mocked(chatBridge.getMessages).mockResolvedValue([])
      vi.mocked(settingsBridge.updateSettings).mockResolvedValue(undefined)

      const settingsStore = useSettingsStore()
      settingsStore.last_active_conversation_id = 'last-id'

      const store = useChatStore()
      await store.loadConversations()

      expect(store.activeConversationId).toBe('last-id')
      expect(chatBridge.getMessages).toHaveBeenCalledWith('last-id')
    })

    it('does not restore when last_active_conversation_id is not set', async () => {
      const convs = [createConv({ id: 'c1' })]
      vi.mocked(conversationBridge.listConversations).mockResolvedValue(convs)

      const store = useChatStore()
      await store.loadConversations()

      expect(store.activeConversationId).toBeNull()
    })

    it('skips restoration when saved conversation no longer exists', async () => {
      vi.mocked(conversationBridge.listConversations).mockResolvedValue([
        createConv({ id: 'conv-1' }),
      ])

      const settingsStore = useSettingsStore()
      settingsStore.last_active_conversation_id = 'deleted-conv'

      const store = useChatStore()
      await store.loadConversations()

      expect(store.activeConversationId).toBeNull()
      expect(chatBridge.getMessages).not.toHaveBeenCalled()
    })
  })

  describe('createConversation', () => {
    it('calls bridge with active provider_id, prepends conversation, and sets active', async () => {
      const newConv = createConv({ id: 'new-id', title: 'New Chat', provider_id: 'prov-1' })
      vi.mocked(conversationBridge.createConversation).mockResolvedValue(newConv)

      const settingsStore = useSettingsStore()
      settingsStore.providers = [createProvider({ id: 'prov-1' })]
      settingsStore.active_provider_id = 'prov-1'

      const store = useChatStore()
      store.conversations = [createConv({ id: 'old-id' })]
      store.messages = [createMsg()]

      await store.createConversation()

      expect(conversationBridge.createConversation).toHaveBeenCalledWith('prov-1')
      expect(store.conversations).toHaveLength(2)
      expect(store.conversations[0].id).toBe('new-id')
      expect(store.activeConversationId).toBe('new-id')
      expect(store.messages).toEqual([])
    })
  })

  describe('deleteConversation', () => {
    it('removes conversation and clears state when active', async () => {
      vi.mocked(conversationBridge.deleteConversation).mockResolvedValue(undefined)

      const store = useChatStore()
      store.conversations = [createConv({ id: 'c1' }), createConv({ id: 'c2' })]
      store.activeConversationId = 'c1'
      store.messages = [createMsg()]

      await store.deleteConversation('c1')

      expect(conversationBridge.deleteConversation).toHaveBeenCalledWith('c1')
      expect(store.conversations.map((c) => c.id)).toEqual(['c2'])
      expect(store.activeConversationId).toBeNull()
      expect(store.messages).toEqual([])
    })

    it('does not clear state when deleting inactive conversation', async () => {
      vi.mocked(conversationBridge.deleteConversation).mockResolvedValue(undefined)

      const store = useChatStore()
      store.conversations = [createConv({ id: 'c1' }), createConv({ id: 'c2' })]
      store.activeConversationId = 'c2'

      await store.deleteConversation('c1')

      expect(store.activeConversationId).toBe('c2')
    })
  })

  describe('switchConversation', () => {
    it('does nothing when switching to already-active conversation', async () => {
      const store = useChatStore()
      store.activeConversationId = 'same'

      await store.switchConversation('same')

      expect(chatBridge.getMessages).not.toHaveBeenCalled()
    })

    it('switches conversation and loads messages', async () => {
      const msgs = [createMsg()]
      vi.mocked(chatBridge.getMessages).mockResolvedValue(msgs)
      vi.mocked(settingsBridge.updateSettings).mockResolvedValue(undefined)

      const store = useChatStore()
      store.activeConversationId = 'old'

      await store.switchConversation('new')

      expect(store.activeConversationId).toBe('new')
      expect(chatBridge.getMessages).toHaveBeenCalledWith('new')
      expect(store.messages).toEqual(msgs)
    })

    it('persists last_active_conversation_id when switching', async () => {
      vi.mocked(chatBridge.getMessages).mockResolvedValue([])
      vi.mocked(settingsBridge.updateSettings).mockResolvedValue(undefined)

      const store = useChatStore()
      store.activeConversationId = 'old'

      await store.switchConversation('target-id')

      const settingsStore = useSettingsStore()
      expect(settingsStore.last_active_conversation_id).toBe('target-id')
    })

    it('does not persist if already on the same conversation', async () => {
      const store = useChatStore()
      store.activeConversationId = 'same-id'

      await store.switchConversation('same-id')

      expect(settingsBridge.updateSettings).not.toHaveBeenCalled()
    })
  })

  describe('pinConversation', () => {
    it('calls bridge and updates local pinned to true', async () => {
      vi.mocked(conversationBridge.pinConversation).mockResolvedValue(undefined)

      const store = useChatStore()
      store.conversations = [createConv({ id: 'c1', pinned: false })]

      await store.pinConversation('c1')

      expect(conversationBridge.pinConversation).toHaveBeenCalledWith('c1')
      expect(store.conversations[0].pinned).toBe(true)
    })
  })

  describe('unpinConversation', () => {
    it('calls bridge and updates local pinned to false', async () => {
      vi.mocked(conversationBridge.unpinConversation).mockResolvedValue(undefined)

      const store = useChatStore()
      store.conversations = [createConv({ id: 'c1', pinned: true })]

      await store.unpinConversation('c1')

      expect(conversationBridge.unpinConversation).toHaveBeenCalledWith('c1')
      expect(store.conversations[0].pinned).toBe(false)
    })
  })

  describe('sendMessage', () => {
    it('returns early if no active conversation', async () => {
      const store = useChatStore()
      store.activeConversationId = null

      await store.sendMessage('hello')

      expect(chatBridge.sendMessage).not.toHaveBeenCalled()
      expect(store.messages).toHaveLength(0)
    })

    it('appends temp user message and calls bridge', async () => {
      vi.mocked(chatBridge.sendMessage).mockResolvedValue(undefined)

      const store = useChatStore()
      store.activeConversationId = 'conv-1'

      await store.sendMessage('hello')

      expect(store.messages).toHaveLength(1)
      expect(store.messages[0]).toMatchObject({
        role: 'user',
        content: 'hello',
        conversation_id: 'conv-1',
      })
      expect(typeof store.messages[0].id).toBe('string')
      expect(typeof store.messages[0].created_at).toBe('number')
      expect(chatBridge.sendMessage).toHaveBeenCalledWith('conv-1', 'hello', false)
    })

    it('sendMessage passes searchEnabled=true when search is on', async () => {
      vi.mocked(chatBridge.sendMessage).mockResolvedValue(undefined)

      const store = useChatStore()
      store.activeConversationId = 'conv-1'
      store.searchEnabled = true

      await store.sendMessage('搜索天气')

      expect(chatBridge.sendMessage).toHaveBeenCalledWith('conv-1', '搜索天气', true)
    })
  })

  describe('appendStreamChunk', () => {
    it('sets streamingId and accumulates content', () => {
      const store = useChatStore()
      store.appendStreamChunk('msg-1', 'Hello')
      expect(store.streamingId).toBe('msg-1')
      expect(store.streamingContent).toBe('Hello')

      store.appendStreamChunk('msg-1', ' World')
      expect(store.streamingContent).toBe('Hello World')
    })
  })

  describe('finishStream', () => {
    it('pushes assistant message and resets streaming state', async () => {
      const store = useChatStore()
      store.streamingId = 'msg-ast'
      store.streamingContent = 'Hello world'
      store.activeConversationId = 'conv-1'

      const before = Date.now()
      await store.finishStream()
      const after = Date.now()

      expect(store.messages).toHaveLength(1)
      expect(store.messages[0]).toMatchObject({
        id: 'msg-ast',
        conversation_id: 'conv-1',
        role: 'assistant',
        content: 'Hello world',
      })
      expect(store.messages[0].created_at).toBeGreaterThanOrEqual(before)
      expect(store.messages[0].created_at).toBeLessThanOrEqual(after)

      expect(store.streamingId).toBeNull()
      expect(store.streamingContent).toBe('')
    })

    it('does nothing when streamingId is null', async () => {
      const store = useChatStore()
      await store.finishStream()
      expect(store.messages).toHaveLength(0)
    })

    it('auto-title: renames conversation from 新对话 on first assistant reply', async () => {
      vi.mocked(conversationBridge.updateConversation).mockResolvedValue(undefined)

      const store = useChatStore()
      store.conversations = [createConv({ id: 'conv-1', title: '新对话' })]
      store.activeConversationId = 'conv-1'
      store.messages = [createMsg({ id: 'msg-user', role: 'user', content: '你好' })]
      store.streamingId = 'msg-ast'
      store.streamingContent = '你好！有什么可以帮助你的吗？'

      await store.finishStream()

      expect(conversationBridge.updateConversation).toHaveBeenCalledWith('conv-1', '你好！有什么可以帮助你的吗？')
      expect(store.conversations[0].title).toBe('你好！有什么可以帮助你的吗？')
    })

    it('auto-title: truncates long title to 30 chars', async () => {
      vi.mocked(conversationBridge.updateConversation).mockResolvedValue(undefined)

      const store = useChatStore()
      store.conversations = [createConv({ id: 'conv-1', title: '新对话' })]
      store.activeConversationId = 'conv-1'
      store.messages = [createMsg({ id: 'msg-user', role: 'user', content: '介绍一下自己' })]
      store.streamingId = 'msg-ast'
      store.streamingContent = '我是一个非常非常非常非常非常非常非常非常非常长的回复内容，应该被截断处理'

      await store.finishStream()

      const calledTitle = vi.mocked(conversationBridge.updateConversation).mock.calls[0][1] as string
      expect(calledTitle.length).toBeLessThanOrEqual(33) // 30 + '...'
    })

    it('auto-title: does not rename if title is not 新对话', async () => {
      vi.mocked(conversationBridge.updateConversation).mockResolvedValue(undefined)

      const store = useChatStore()
      store.conversations = [createConv({ id: 'conv-1', title: '自定义标题' })]
      store.activeConversationId = 'conv-1'
      store.messages = [createMsg({ id: 'msg-user', role: 'user', content: '你好' })]
      store.streamingId = 'msg-ast'
      store.streamingContent = '你好！'

      await store.finishStream()

      expect(conversationBridge.updateConversation).not.toHaveBeenCalled()
    })

    it('auto-title: strips markdown formatting', async () => {
      vi.mocked(conversationBridge.updateConversation).mockResolvedValue(undefined)

      const store = useChatStore()
      store.conversations = [createConv({ id: 'conv-1', title: '新对话' })]
      store.activeConversationId = 'conv-1'
      store.messages = [createMsg({ id: 'msg-user', role: 'user', content: '写个函数' })]
      store.streamingId = 'msg-ast'
      store.streamingContent = '# 你好世界\n\n这是一个 **加粗** 和 `代码` 的回复'

      await store.finishStream()

      expect(conversationBridge.updateConversation).toHaveBeenCalledWith('conv-1', '你好世界')
    })
  })

  describe('deleteMessage', () => {
    it('calls bridge and removes message from local state', async () => {
      vi.mocked(chatBridge.deleteMessage).mockResolvedValue(undefined)

      const store = useChatStore()
      store.messages = [
        createMsg({ id: 'msg-1', content: 'hello' }),
        createMsg({ id: 'msg-2', content: 'world' }),
      ]

      await store.deleteMessage('msg-1')

      expect(chatBridge.deleteMessage).toHaveBeenCalledWith('msg-1')
      expect(store.messages).toHaveLength(1)
      expect(store.messages[0].id).toBe('msg-2')
    })

    it('does nothing when message id does not exist', async () => {
      vi.mocked(chatBridge.deleteMessage).mockResolvedValue(undefined)

      const store = useChatStore()
      store.messages = [createMsg({ id: 'msg-1' })]

      await store.deleteMessage('non-existent')

      expect(chatBridge.deleteMessage).toHaveBeenCalledWith('non-existent')
      expect(store.messages).toHaveLength(1)
    })
  })

  describe('regenerateMessage', () => {
    it('returns early if no active conversation', async () => {
      const store = useChatStore()
      store.activeConversationId = null

      await store.regenerateMessage()

      expect(chatBridge.sendMessage).not.toHaveBeenCalled()
    })

    it('deletes last assistant message and calls regenerate bridge', async () => {
      vi.mocked(chatBridge.deleteMessage).mockResolvedValue(undefined)
      vi.mocked(chatBridge.regenerateMessage).mockResolvedValue(undefined)

      const store = useChatStore()
      store.activeConversationId = 'conv-1'
      store.messages = [
        createMsg({ id: 'user-1', role: 'user', content: 'hello' }),
        createMsg({ id: 'ast-1', role: 'assistant', content: 'hi there' }),
      ]

      await store.regenerateMessage()

      expect(chatBridge.deleteMessage).toHaveBeenCalledWith('ast-1')
      expect(store.messages).toHaveLength(1)
      expect(store.messages[0].id).toBe('user-1')
      expect(chatBridge.regenerateMessage).toHaveBeenCalledWith('conv-1')
      expect(chatBridge.sendMessage).not.toHaveBeenCalled()
    })

    it('does nothing if last message is not assistant', async () => {
      const store = useChatStore()
      store.activeConversationId = 'conv-1'
      store.messages = [
        createMsg({ id: 'user-1', role: 'user', content: 'hello' }),
      ]

      await store.regenerateMessage()

      expect(chatBridge.deleteMessage).not.toHaveBeenCalled()
      expect(chatBridge.sendMessage).not.toHaveBeenCalled()
    })
  })
})

// ---------------------------------------------------------------------------
// settingsStore
// ---------------------------------------------------------------------------
describe('settingsStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('has correct default values', () => {
    const store = useSettingsStore()
    expect(store.providers).toEqual([])
    expect(store.active_provider_id).toBe('')
    expect(store.temperature).toBe(0.7)
  })

  describe('loadSettings', () => {
    it('fetches remote settings and applies them to state', async () => {
      const remote: Settings = {
        providers: [createProvider()],
        active_provider_id: 'prov-1',
        temperature: 0.5,
        top_p: 0.9,
      }
      vi.mocked(settingsBridge.getSettings).mockResolvedValue(remote)

      const store = useSettingsStore()
      await store.loadSettings()

      expect(settingsBridge.getSettings).toHaveBeenCalledOnce()
      expect(store.providers).toHaveLength(1)
      expect(store.providers[0].id).toBe('prov-1')
      expect(store.active_provider_id).toBe('prov-1')
      expect(store.temperature).toBe(0.5)
    })
  })

  describe('addProvider', () => {
    it('prepends provider to list without changing active', async () => {
      vi.mocked(settingsBridge.updateSettings).mockResolvedValue(undefined)

      const store = useSettingsStore()
      store.providers = [createProvider({ id: 'old' })]
      store.active_provider_id = 'old'

      const p = await store.addProvider({ name: 'OpenAI', base_url: 'https://api.openai.com/v1' })

      expect(store.providers).toHaveLength(2)
      expect(store.providers[0].name).toBe('OpenAI')
      expect(store.active_provider_id).toBe('old')
      expect(settingsBridge.updateSettings).toHaveBeenCalled()
    })
  })

  describe('removeProvider', () => {
    it('removes provider from list', async () => {
      vi.mocked(settingsBridge.updateSettings).mockResolvedValue(undefined)

      const store = useSettingsStore()
      store.providers = [createProvider({ id: 'p1' }), createProvider({ id: 'p2' })]
      store.active_provider_id = 'p2'

      await store.removeProvider('p1')

      expect(store.providers).toHaveLength(1)
      expect(store.providers[0].id).toBe('p2')
    })

    it('refuses to remove active provider', async () => {
      vi.mocked(settingsBridge.updateSettings).mockResolvedValue(undefined)

      const store = useSettingsStore()
      store.providers = [createProvider({ id: 'p1' }), createProvider({ id: 'p2' })]
      store.active_provider_id = 'p1'

      await store.removeProvider('p1')

      expect(store.providers).toHaveLength(2)
      expect(settingsBridge.updateSettings).not.toHaveBeenCalled()
    })
  })

  describe('updateProvider', () => {
    it('merges partial update into provider', async () => {
      vi.mocked(settingsBridge.updateSettings).mockResolvedValue(undefined)

      const store = useSettingsStore()
      store.providers = [createProvider({ id: 'p1', name: 'Old' })]

      await store.updateProvider('p1', { name: 'New' })

      expect(store.providers[0].name).toBe('New')
      expect(settingsBridge.updateSettings).toHaveBeenCalled()
    })
  })

  describe('setActiveProvider', () => {
    it('changes active_provider_id', async () => {
      vi.mocked(settingsBridge.updateSettings).mockResolvedValue(undefined)

      const store = useSettingsStore()
      store.providers = [createProvider({ id: 'p1' }), createProvider({ id: 'p2' })]

      await store.setActiveProvider('p2')

      expect(store.active_provider_id).toBe('p2')
    })
  })

  describe('fetchModels', () => {
    it('calls bridge and updates provider models', async () => {
      vi.mocked(settingsBridge.fetchProviderModels).mockResolvedValue(['gpt-4o', 'gpt-4o-mini'])
      vi.mocked(settingsBridge.updateSettings).mockResolvedValue(undefined)

      const store = useSettingsStore()
      store.providers = [createProvider({ id: 'p1', models: [] })]

      await store.fetchModels('p1')

      expect(settingsBridge.fetchProviderModels).toHaveBeenCalledWith(store.providers[0])
      expect(store.providers[0].models).toEqual(['gpt-4o', 'gpt-4o-mini'])
    })
  })

  describe('addModel', () => {
    it('appends model to provider', () => {
      vi.mocked(settingsBridge.updateSettings).mockResolvedValue(undefined)

      const store = useSettingsStore()
      store.providers = [createProvider({ id: 'p1', models: ['gpt-4'] })]

      store.addModel('p1', 'gpt-4o')

      expect(store.providers[0].models).toEqual(['gpt-4', 'gpt-4o'])
    })

    it('does not add duplicate model', () => {
      vi.mocked(settingsBridge.updateSettings).mockResolvedValue(undefined)

      const store = useSettingsStore()
      store.providers = [createProvider({ id: 'p1', models: ['gpt-4'] })]

      store.addModel('p1', 'gpt-4')

      expect(store.providers[0].models).toEqual(['gpt-4'])
    })
  })

  describe('removeModel', () => {
    it('removes model from provider', () => {
      vi.mocked(settingsBridge.updateSettings).mockResolvedValue(undefined)

      const store = useSettingsStore()
      store.providers = [createProvider({ id: 'p1', models: ['gpt-4', 'gpt-4o'] })]

      store.removeModel('p1', 'gpt-4')

      expect(store.providers[0].models).toEqual(['gpt-4o'])
    })
  })

  describe('testConnection', () => {
    it('calls bridge with provider and returns result', async () => {
      vi.mocked(settingsBridge.testProviderConnection).mockResolvedValue({ ok: true })

      const store = useSettingsStore()
      store.providers = [createProvider({ id: 'p1' })]

      const result = await store.testConnection('p1')

      expect(settingsBridge.testProviderConnection).toHaveBeenCalledWith(store.providers[0])
      expect(result).toEqual({ ok: true })
    })

    it('returns error for non-existent provider', async () => {
      const store = useSettingsStore()

      const result = await store.testConnection('non-existent')

      expect(result.ok).toBe(false)
      expect(result.error).toBe('Provider 不存在')
    })
  })

  describe('getters', () => {
    it('activeProvider returns the active provider', () => {
      const store = useSettingsStore()
      store.providers = [createProvider({ id: 'p1' }), createProvider({ id: 'p2' })]
      store.active_provider_id = 'p2'

      expect(store.activeProvider?.id).toBe('p2')
    })

    it('activeProvider returns undefined when no active', () => {
      const store = useSettingsStore()
      store.providers = [createProvider({ id: 'p1' })]
      store.active_provider_id = ''

      expect(store.activeProvider).toBeUndefined()
    })

    it('enabledProviders filters disabled providers', () => {
      const store = useSettingsStore()
      store.providers = [
        createProvider({ id: 'p1', enabled: true }),
        createProvider({ id: 'p2', enabled: false }),
        createProvider({ id: 'p3', enabled: true }),
      ]

      expect(store.enabledProviders).toHaveLength(2)
      expect(store.enabledProviders.map(p => p.id)).toEqual(['p1', 'p3'])
    })
  })
})

// ---------------------------------------------------------------------------
// promptStore
// ---------------------------------------------------------------------------
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
