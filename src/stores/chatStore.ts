import { defineStore } from 'pinia'
import type { Conversation, Message } from '../types'
import * as chatBridge from '../bridge/chat'
import * as conversationBridge from '../bridge/conversation'
import { log } from '../bridge/log'

export const useChatStore = defineStore('chat', {
  state: () => ({
    conversations: [] as Conversation[],
    activeConversationId: null as string | null,
    messages: [] as Message[],
    streamingId: null as string | null,
    streamingContent: '',
  }),

  actions: {
    async loadConversations(): Promise<void> {
      this.conversations = await conversationBridge.listConversations()
    },

    async createConversation(): Promise<void> {
      await log('info', '前端::chatStore::createConversation | 新建对话')
      const conv = await conversationBridge.createConversation()
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
      await conversationBridge.updateConversation(id, title)
      const conv = this.conversations.find(c => c.id === id)
      if (conv) conv.title = title
    },

    async pinConversation(id: string): Promise<void> {
      await log('info', `前端::chatStore::pinConversation | 置顶对话 | id=${id}`)
      await conversationBridge.pinConversation(id)
      const conv = this.conversations.find(c => c.id === id)
      if (conv) {
        conv.pinned = true
      }
    },

    async unpinConversation(id: string): Promise<void> {
      await log('info', `前端::chatStore::unpinConversation | 取消置顶 | id=${id}`)
      await conversationBridge.unpinConversation(id)
      const conv = this.conversations.find(c => c.id === id)
      if (conv) {
        conv.pinned = false
      }
    },

    async switchConversation(id: string): Promise<void> {
      await log('info', `前端::chatStore::switchConversation | 切换对话 | id=${id}`)
      if (this.activeConversationId === id) return
      this.activeConversationId = id
      this.messages = await chatBridge.getMessages(id)
    },

    async sendMessage(content: string): Promise<void> {
      await log('info', `前端::chatStore::sendMessage | 发送消息 | len=${content.length}`)
      if (!this.activeConversationId) return
      const tempMsg: Message = {
        id: crypto.randomUUID(),
        conversation_id: this.activeConversationId,
        role: 'user',
        content,
        created_at: Date.now(),
      }
      this.messages.push(tempMsg)
      await chatBridge.sendMessage(this.activeConversationId, content)
    },

    appendStreamChunk(messageId: string, delta: string): void {
      this.streamingId = messageId
      this.streamingContent += delta
    },

    async finishStream(): Promise<void> {
      await log('info', '前端::chatStore::finishStream | 流式完成')
      if (!this.streamingId) return
      const finalMsg: Message = {
        id: this.streamingId,
        conversation_id: this.activeConversationId || '',
        role: 'assistant',
        content: this.streamingContent,
        created_at: Date.now(),
      }
      this.messages.push(finalMsg)
      this.streamingId = null
      this.streamingContent = ''
    },
  },
})
