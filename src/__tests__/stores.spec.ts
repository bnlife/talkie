import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import type { Conversation, Message, Settings } from '../types'

// Mock bridge modules before importing stores
vi.mock('../bridge/chat')
vi.mock('../bridge/conversation')
vi.mock('../bridge/settings')
vi.mock('../bridge/log')

import { useChatStore } from '../stores/chatStore'
import { useSettingsStore } from '../stores/settingsStore'
import * as chatBridge from '../bridge/chat'
import * as conversationBridge from '../bridge/conversation'
import * as settingsBridge from '../bridge/settings'

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------
function createConv(overrides: Partial<Conversation> = {}): Conversation {
  return {
    id: 'conv-1',
    title: 'Test',
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

      // Set the saved last_active id
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
      // Only conv-1 exists, but saved id points to deleted-conv
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
    it('calls bridge, prepends conversation, and sets active', async () => {
      const newConv = createConv({ id: 'new-id', title: 'New Chat' })
      vi.mocked(conversationBridge.createConversation).mockResolvedValue(newConv)

      const store = useChatStore()
      store.conversations = [createConv({ id: 'old-id' })]
      store.messages = [createMsg()]

      await store.createConversation()

      expect(conversationBridge.createConversation).toHaveBeenCalledOnce()
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

      // settingsStore should have the new id
      const settingsStore = useSettingsStore()
      expect(settingsStore.last_active_conversation_id).toBe('target-id')

      // bridge should have been called with full settings including the id
      expect(settingsBridge.updateSettings).toHaveBeenCalled()
      const callArg = vi.mocked(settingsBridge.updateSettings).mock.calls[0][0] as Settings
      expect(callArg.last_active_conversation_id).toBe('target-id')
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
      expect(chatBridge.sendMessage).toHaveBeenCalledWith('conv-1', 'hello')
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
    expect(store.base_url).toBe('https://api.deepseek.com/v1')
    expect(store.api_key).toBe('')
    expect(store.model).toBe('deepseek-chat')
    expect(store.temperature).toBe(0.7)
  })

  describe('loadSettings', () => {
    it('fetches remote settings and applies them to state', async () => {
      const remote = {
        base_url: 'https://custom.api.com',
        api_key: 'sk-custom',
        model: 'gpt-4',
        temperature: 0.5,
      }
      vi.mocked(settingsBridge.getSettings).mockResolvedValue(remote)

      const store = useSettingsStore()
      await store.loadSettings()

      expect(settingsBridge.getSettings).toHaveBeenCalledOnce()
      expect(store.base_url).toBe('https://custom.api.com')
      expect(store.api_key).toBe('sk-custom')
      expect(store.model).toBe('gpt-4')
      expect(store.temperature).toBe(0.5)
    })
  })

  describe('updateSettings', () => {
    it('partially updates state and persists via bridge', async () => {
      vi.mocked(settingsBridge.updateSettings).mockResolvedValue(undefined)

      const store = useSettingsStore()
      await store.updateSettings({ model: 'gpt-4', temperature: 0.2 })

      expect(store.model).toBe('gpt-4')
      expect(store.temperature).toBe(0.2)
      // unchanged fields
      expect(store.base_url).toBe('https://api.deepseek.com/v1')
      expect(store.api_key).toBe('')

      expect(settingsBridge.updateSettings).toHaveBeenCalledWith({
        model: 'gpt-4',
        temperature: 0.2,
      })
    })
  })

  describe('testConnection', () => {
    it('calls bridge with current settings and returns result', async () => {
      vi.mocked(settingsBridge.testConnection).mockResolvedValue({ ok: true })

      const store = useSettingsStore()
      const result = await store.testConnection()

      expect(settingsBridge.testConnection).toHaveBeenCalledWith({
        base_url: 'https://api.deepseek.com/v1',
        api_key: '',
        model: 'deepseek-chat',
        temperature: 0.7,
      })
      expect(result).toEqual({ ok: true })
    })
  })
})
