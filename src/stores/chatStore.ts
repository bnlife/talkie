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
  }),

  getters: {
    activeConversation(state): ConversationView | undefined {
      return state.conversations.find(c => c.id === state.activeConversationId)
    },
    searchEnabled(state): boolean {
      return state.conversations.find(c => c.id === state.activeConversationId)?.search_enabled ?? false
    },
  },

  actions: {
    async loadConversations(): Promise<void> {
      this.conversations = await conversationBridge.listConversations()
      const settingsStore = useSettingsStore()
      if (settingsStore.last_active_conversation_id) {
        const exists = this.conversations.some(
          c => c.id === settingsStore.last_active_conversation_id
        )
        if (exists) {
          await log('info', `前端::chatStore::loadConversations | 恢复最后对话 | id=${settingsStore.last_active_conversation_id}`)
          await this.switchConversation(settingsStore.last_active_conversation_id)
          return
        } else {
          await log('info', `前端::chatStore::loadConversations | 最后对话已不存在，跳过 | id=${settingsStore.last_active_conversation_id}`)
        }
      }
      // Fallback: activate the most recent conversation
      if (this.conversations.length > 0) {
        await log('info', `前端::chatStore::loadConversations | 自动激活最近对话 | id=${this.conversations[0].id}`)
        await this.switchConversation(this.conversations[0].id)
      }
    },

    async createConversation(): Promise<void> {
      const settingsStore = useSettingsStore()
      const providerId = settingsStore.active_provider_id
      await log('info', `前端::chatStore::createConversation | 新建对话 | provider_id=${providerId}`)
      const conv = await conversationBridge.createConversation(providerId)
      this.conversations.unshift(conv)
      this.activeConversationId = conv.id
      this.messages = []
    },

    async deleteConversation(id: string): Promise<void> {
      await log('info', `前端::chatStore::deleteConversation | 删除对话 | id=${id}`)
      await conversationBridge.deleteConversation(id)
      this.conversations = this.conversations.filter(c => c.id !== id)
      if (this.activeConversationId === id) {
        this.activeConversationId = null
        this.messages = []
      }
    },

    async renameConversation(id: string, title: string): Promise<void> {
      await log('info', `前端::chatStore::renameConversation | 重命名对话 | id=${id}`)
      await conversationBridge.updateConversation(id, { title })
      const conv = this.conversations.find(c => c.id === id)
      if (conv) conv.title = title
    },

    async pinConversation(id: string): Promise<void> {
      await log('info', `前端::chatStore::pinConversation | 置顶对话 | id=${id}`)
      await conversationBridge.pinConversation(id)
      const conv = this.conversations.find(c => c.id === id)
      if (conv) conv.pinned = true
    },

    async unpinConversation(id: string): Promise<void> {
      await log('info', `前端::chatStore::unpinConversation | 取消置顶 | id=${id}`)
      await conversationBridge.unpinConversation(id)
      const conv = this.conversations.find(c => c.id === id)
      if (conv) conv.pinned = false
    },

    async switchConversation(id: string): Promise<void> {
      await log('info', `前端::chatStore::switchConversation | 切换对话 | id=${id}`)
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
      await log('info', `前端::chatStore::sendMessage | 发送消息 | len=${content.length} search=${conv?.search_enabled ?? false}`)
      if (!this.activeConversationId || !conv) return
      const tempMsg: Message = {
        id: crypto.randomUUID(),
        conversation_id: this.activeConversationId,
        role: 'user',
        content,
        created_at: Date.now(),
      }
      this.messages.push(tempMsg)
      await chatBridge.sendMessage(this.activeConversationId, content, conv.search_enabled)
    },

    async toggleSearch(): Promise<void> {
      const conv = this.conversations.find(c => c.id === this.activeConversationId)
      if (!conv) return
      const newValue = !conv.search_enabled
      await log('info', `前端::chatStore::toggleSearch | 切换搜索 | id=${conv.id} enabled=${newValue}`)
      await conversationBridge.updateConversation(conv.id, { searchEnabled: newValue })
      conv.search_enabled = newValue
    },

    async selectPrompt(promptId: string | null): Promise<void> {
      const conv = this.conversations.find(c => c.id === this.activeConversationId)
      if (!conv) return
      await log('info', `前端::chatStore::selectPrompt | 选择提示词 | id=${conv.id} promptId=${promptId}`)
      await conversationBridge.updateConversation(conv.id, { promptId })
      conv.prompt_id = promptId
    },

    appendStreamChunk(messageId: string, delta: string): void {
      this.streamingId = messageId
      this.streamingContent += delta
    },

    async finishStream(tokenCount?: number): Promise<void> {
      await log('info', '前端::chatStore::finishStream | 流式完成')
      if (!this.streamingId) return
      const finalMsg: Message = {
        id: this.streamingId,
        conversation_id: this.activeConversationId || '',
        role: 'assistant',
        content: this.streamingContent,
        created_at: Date.now(),
        token_count: tokenCount ?? undefined,
      }
      this.messages.push(finalMsg)
      this.streamingId = null
      this.streamingContent = ''

      try {
        const conv = this.conversations.find(c => c.id === this.activeConversationId)
        await log('info', `前端::chatStore::finishStream | 检查自动标题 | conv=${!!conv} title=${conv?.title} assistantCount=${this.messages.filter(m => m.role === 'assistant').length}`)
        if (conv && conv.title === '新对话') {
          const assistantCount = this.messages.filter(m => m.role === 'assistant').length
          if (assistantCount === 1) {
            const autoTitle = this.extractTitle(finalMsg.content)
            await log('info', `前端::chatStore::finishStream | extractTitle | raw="${finalMsg.content.slice(0, 50)}" | result="${autoTitle}"`)
            if (autoTitle) {
              await log('info', `前端::chatStore::finishStream | 自动设置标题 | title=${autoTitle}`)
              await this.renameConversation(this.activeConversationId!, autoTitle)
            } else {
              await log('info', '前端::chatStore::finishStream | extractTitle 为空，跳过')
            }
          }
        }
      } catch (e) {
        await log('error', `前端::chatStore::finishStream | 自动标题失败 | ${e}`)
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
      await log('info', `前端::chatStore::deleteMessage | 删除消息 | id=${messageId}`)
      await chatBridge.deleteMessage(messageId)
      this.messages = this.messages.filter(m => m.id !== messageId)
    },

    async regenerateMessage(): Promise<void> {
      await log('info', '前端::chatStore::regenerateMessage | 重新生成')
      if (!this.activeConversationId) return
      const lastMsg = this.messages[this.messages.length - 1]
      if (!lastMsg || lastMsg.role !== 'assistant') return
      this.messages.pop()
      await chatBridge.deleteMessage(lastMsg.id)
      await chatBridge.regenerateMessage(this.activeConversationId)
    },
  },
})
