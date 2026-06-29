import { defineStore } from 'pinia'
import type { Conversation, Message } from '../types'
import * as chatBridge from '../bridge/chat'
import * as conversationBridge from '../bridge/conversation'

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
      const conv = await conversationBridge.createConversation()
      this.conversations.unshift(conv)
      this.activeConversationId = conv.id
      this.messages = []
    },

    async deleteConversation(id: string): Promise<void> {
      await conversationBridge.deleteConversation(id)
      this.conversations = this.conversations.filter(c => c.id !== id)
      if (this.activeConversationId === id) {
        this.activeConversationId = null
        this.messages = []
      }
    },

    async switchConversation(id: string): Promise<void> {
      if (this.activeConversationId === id) return
      this.activeConversationId = id
      this.messages = await chatBridge.getMessages(id)
    },

    async sendMessage(content: string): Promise<void> {
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

    finishStream(): void {
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
