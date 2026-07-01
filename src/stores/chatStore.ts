import { defineStore } from 'pinia'
import type { ConversationView, Message } from '../types'
import * as chatBridge from '../bridge/chat'
import * as conversationBridge from '../bridge/conversation'
import { log } from '../bridge/log'
import { useSettingsStore } from './settingsStore'

export const useChatStore = defineStore('chat', {
  state: () => ({
    conversations: [] as ConversationView[],
    activeConversationId: null as string | null,
    messages: [] as Message[],
    streamingId: null as string | null,
    streamingContent: '',
    streamingThinking: '',
    streamingThinkingStart: 0,
    streamingSearchResults: null as import('@/types').SearchResult[] | null,
    waitingForResponse: false,
    drafts: {} as Record<string, string>,
  }),

  getters: {
    activeConversation(state): ConversationView | undefined {
      return state.conversations.find(c => c.id === state.activeConversationId)
    },
    searchEnabled(state): boolean {
      return state.conversations.find(c => c.id === state.activeConversationId)?.search_enabled ?? false
    },
    searchEngine(state): string {
      return state.conversations.find(c => c.id === state.activeConversationId)?.search_engine ?? ''
    },
  },

  actions: {
    getDraft(conversationId: string): string {
      return this.drafts[conversationId] ?? ''
    },

    setDraft(conversationId: string, text: string): void {
      this.drafts[conversationId] = text
    },

    clearDraft(conversationId: string): void {
      delete this.drafts[conversationId]
    },
    async loadConversations(): Promise<void> {
      this.conversations = await conversationBridge.listConversations()
      const settingsStore = useSettingsStore()
      if (settingsStore.last_active_conversation_id) {
        const exists = this.conversations.some(
          c => c.id === settingsStore.last_active_conversation_id
        )
        if (exists) {
          await log('info', `FE::chatStore | restore last | id=${settingsStore.last_active_conversation_id}`)
          await this.switchConversation(settingsStore.last_active_conversation_id)
          return
        } else {
          await log('info', `FE::chatStore | last conv gone | id=${settingsStore.last_active_conversation_id}`)
        }
      }
      // Fallback: activate the most recent conversation
      if (this.conversations.length > 0) {
        await log('info', `FE::chatStore | auto activate recent | id=${this.conversations[0].id}`)
        await this.switchConversation(this.conversations[0].id)
      }
    },

    async createConversation(): Promise<void> {
      const settingsStore = useSettingsStore()
      const providerId = settingsStore.active_provider_id
      await log('info', `FE::chatStore | create | provider=${providerId}`)
      const conv = await conversationBridge.createConversation(providerId)
      this.conversations.unshift(conv)
      this.activeConversationId = conv.id
      this.messages = []
    },

    async deleteConversation(id: string): Promise<void> {
      await log('info', `FE::chatStore | delete | id=${id}`)
      await conversationBridge.deleteConversation(id)
      this.conversations = this.conversations.filter(c => c.id !== id)
      if (this.activeConversationId === id) {
        this.activeConversationId = null
        this.messages = []
      }
    },

    async renameConversation(id: string, title: string): Promise<void> {
      await log('info', `FE::chatStore | rename | id=${id}`)
      await conversationBridge.updateConversation(id, { title })
      const conv = this.conversations.find(c => c.id === id)
      if (conv) conv.title = title
    },

    async pinConversation(id: string): Promise<void> {
      await log('info', `FE::chatStore | pin | id=${id}`)
      await conversationBridge.pinConversation(id)
      const conv = this.conversations.find(c => c.id === id)
      if (conv) conv.pinned = true
    },

    async unpinConversation(id: string): Promise<void> {
      await log('info', `FE::chatStore | unpin | id=${id}`)
      await conversationBridge.unpinConversation(id)
      const conv = this.conversations.find(c => c.id === id)
      if (conv) conv.pinned = false
    },

    async switchConversation(id: string): Promise<void> {
      await log('info', `FE::chatStore | switch | id=${id}`)
      if (this.activeConversationId === id) return
      this.activeConversationId = id
      this.messages = await chatBridge.getMessages(id)
      const settingsStore = useSettingsStore()
      if (settingsStore.last_active_conversation_id !== id) {
        settingsStore.last_active_conversation_id = id
        await settingsStore.saveSettings()
      }
    },

    async sendMessage(content: string): Promise<void> {
      const conv = this.conversations.find(c => c.id === this.activeConversationId)
      await log('info', `FE::chatStore | send | len=${content.length} search=${conv?.search_enabled ?? false} engine=${conv?.search_engine ?? ''}`)
      if (!this.activeConversationId || !conv) return
      this.clearDraft(this.activeConversationId)
      const tempMsg: Message = {
        id: crypto.randomUUID(),
        conversation_id: this.activeConversationId,
        role: 'user',
        content,
        created_at: Date.now(),
      }
      this.messages.push(tempMsg)
      this.waitingForResponse = true
      await chatBridge.sendMessage(this.activeConversationId, content, conv.search_enabled, conv.search_engine || undefined)
    },

    async selectSearchEngine(engine: string): Promise<void> {
      const conv = this.conversations.find(c => c.id === this.activeConversationId)
      if (!conv) return
      // If clicking the same engine, toggle off; otherwise switch engine and enable
      const isSameEngine = conv.search_engine === engine && conv.search_enabled
      const newEnabled = !isSameEngine
      const newEngine = isSameEngine ? '' : engine
      await log('info', `FE::chatStore | select search engine | id=${conv.id} engine=${newEngine} enabled=${newEnabled}`)
      await conversationBridge.updateConversation(conv.id, { searchEnabled: newEnabled, searchEngine: newEngine })
      conv.search_enabled = newEnabled
      conv.search_engine = newEngine
    },

    async selectPrompt(promptId: string | null): Promise<void> {
      const conv = this.conversations.find(c => c.id === this.activeConversationId)
      if (!conv) return
      await log('info', `FE::chatStore | select prompt | id=${conv.id} promptId=${promptId}`)
      await conversationBridge.updateConversation(conv.id, { promptId: promptId ?? '' })
      conv.prompt_id = promptId
    },

    async switchModel(providerId: string, model: string): Promise<void> {
      const conv = this.conversations.find(c => c.id === this.activeConversationId)
      if (!conv) return
      await log('info', `FE::chatStore | switch model | id=${conv.id} provider=${providerId} model=${model}`)
      await conversationBridge.updateConversation(conv.id, { providerId, model })
      conv.provider_id = providerId
      conv.model = model
    },

    appendStreamChunk(messageId: string, delta: string): void {
      this.waitingForResponse = false
      this.streamingId = messageId
      this.streamingContent += delta
    },

    appendThinkingChunk(messageId: string, delta: string): void {
      this.waitingForResponse = false
      if (!this.streamingThinkingStart) this.streamingThinkingStart = Date.now()
      this.streamingId = messageId
      this.streamingThinking += delta
    },

    setStreamingSearchResults(results: import('@/types').SearchResult[]): void {
      this.streamingSearchResults = results
    },

    async finishStream(tokenCount?: number, searchResults?: import('@/types').SearchResult[]): Promise<void> {
      await log('info', 'FE::chatStore | stream done')
      this.waitingForResponse = false
      if (!this.streamingId) return
      const finalMsg: Message = {
        id: this.streamingId,
        conversation_id: this.activeConversationId || '',
        role: 'assistant',
        content: this.streamingContent,
        created_at: Date.now(),
        token_count: tokenCount ?? undefined,
        search_results: searchResults && searchResults.length > 0 ? searchResults : undefined,
        thinking_content: this.streamingThinking || undefined,
      }
      this.messages.push(finalMsg)
      this.streamingId = null
      this.streamingContent = ''
      this.streamingThinking = ''
      this.streamingThinkingStart = 0
      this.streamingSearchResults = null

      try {
        const conv = this.conversations.find(c => c.id === this.activeConversationId)
        await log('info', `FE::chatStore | auto title check | conv=${!!conv} title=${conv?.title} asst=${this.messages.filter(m => m.role === 'assistant').length}`)
        if (conv && conv.title === '新对话') {
          const assistantCount = this.messages.filter(m => m.role === 'assistant').length
          if (assistantCount === 1) {
            const autoTitle = this.extractTitle(finalMsg.content)
            await log('info', `FE::chatStore | extractTitle | raw="${finalMsg.content.slice(0, 50)}" → "${autoTitle}"`)
            if (autoTitle) {
              await log('info', `FE::chatStore | set title | title=${autoTitle}`)
              await this.renameConversation(this.activeConversationId!, autoTitle)
            } else {
              await log('info', 'FE::chatStore | extractTitle empty, skip')
            }
          }
        }
      } catch (e) {
        await log('error', `FE::chatStore | auto title fail | ${e}`)
      }
    },

    extractTitle(content: string): string {
      const lines = content.split('\n').filter(l => l.trim())
      if (!lines.length) return ''
      let title = lines[0].trim()
      title = title.replace(/^#+\s*/, '')
      title = title.replace(/[*_`~]+/g, '')
      title = title.replace(/^\s*[-*+]\s*/, '')
      title = title.replace(/^\s*\d+\.\s*/, '')
      title = title.replace(/^>\s*/, '')
      if (title.length > 30) title = title.slice(0, 30) + '...'
      return title
    },

    async deleteMessage(messageId: string): Promise<void> {
      await log('info', `FE::chatStore | del msg | id=${messageId}`)
      await chatBridge.deleteMessage(messageId)
      this.messages = this.messages.filter(m => m.id !== messageId)
    },

    async regenerateMessage(): Promise<void> {
      await log('info', 'FE::chatStore | regen')
      if (!this.activeConversationId) return
      const lastMsg = this.messages[this.messages.length - 1]
      if (!lastMsg || lastMsg.role !== 'assistant') return
      this.messages.pop()
      await chatBridge.deleteMessage(lastMsg.id)
      await chatBridge.regenerateMessage(this.activeConversationId)
    },
  },
})
