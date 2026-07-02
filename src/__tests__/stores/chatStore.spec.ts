import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useChatStore } from '../../stores/chatStore'
import { useSettingsStore } from '../../stores/settingsStore'
import * as chatBridge from '../../bridge/chat'
import * as conversationBridge from '../../bridge/conversation'
import * as settingsBridge from '../../bridge/settings'
import { createConv, createMsg, createProvider } from './helpers'

vi.mock('../../bridge/chat')
vi.mock('../../bridge/conversation')
vi.mock('../../bridge/settings')

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

    it('falls back to first conversation when last_active_conversation_id is not set', async () => {
      const convs = [createConv({ id: 'c1' })]
      vi.mocked(conversationBridge.listConversations).mockResolvedValue(convs)
      vi.mocked(chatBridge.getMessages).mockResolvedValue([])

      const store = useChatStore()
      await store.loadConversations()

      expect(store.activeConversationId).toBe('c1')
    })

    it('falls back to first conversation when saved conversation no longer exists', async () => {
      vi.mocked(conversationBridge.listConversations).mockResolvedValue([
        createConv({ id: 'conv-1' }),
      ])
      vi.mocked(chatBridge.getMessages).mockResolvedValue([])

      const settingsStore = useSettingsStore()
      settingsStore.last_active_conversation_id = 'deleted-conv'

      const store = useChatStore()
      await store.loadConversations()

      expect(store.activeConversationId).toBe('conv-1')
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
      store.conversations = [createConv({ id: 'conv-1' })]
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
      expect(chatBridge.sendMessage).toHaveBeenCalledWith('conv-1', 'hello', undefined, false, undefined)
    })

    it('sendMessage passes searchEnabled=true when search is on', async () => {
      vi.mocked(chatBridge.sendMessage).mockResolvedValue(undefined)

      const store = useChatStore()
      store.conversations = [createConv({ id: 'conv-1', search_enabled: true, search_engine: 'bocha-search' })]
      store.activeConversationId = 'conv-1'

      await store.sendMessage('搜索天气')

      expect(chatBridge.sendMessage).toHaveBeenCalledWith('conv-1', '搜索天气', undefined, true, 'bocha-search')
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

      expect(conversationBridge.updateConversation).toHaveBeenCalledWith('conv-1', { title: '你好！有什么可以帮助你的吗？' })
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

      const calledTitle = vi.mocked(conversationBridge.updateConversation).mock.calls[0][1].title as string
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

      expect(conversationBridge.updateConversation).toHaveBeenCalledWith('conv-1', { title: '你好世界' })
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

  describe('switchModel', () => {
    it('updates conversation model via bridge and local state', async () => {
      vi.mocked(conversationBridge.updateConversation).mockResolvedValue(undefined)

      const store = useChatStore()
      store.conversations = [createConv({ id: 'conv-1', provider_id: 'prov-1', model: 'gpt-4' })]
      store.activeConversationId = 'conv-1'

      await store.switchModel('prov-2', 'deepseek-chat')

      expect(conversationBridge.updateConversation).toHaveBeenCalledWith('conv-1', { providerId: 'prov-2', model: 'deepseek-chat' })
      expect(store.conversations[0].provider_id).toBe('prov-2')
      expect(store.conversations[0].model).toBe('deepseek-chat')
    })

    it('does nothing when no active conversation', async () => {
      const store = useChatStore()
      store.activeConversationId = null

      await store.switchModel('prov-1', 'gpt-4')

      expect(conversationBridge.updateConversation).not.toHaveBeenCalled()
    })
  })
})
