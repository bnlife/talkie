import { invoke } from '@tauri-apps/api/core'
import type { ConversationView } from '../types'

export async function listConversations(): Promise<ConversationView[]> {
  return invoke<ConversationView[]>('list_conversations')
}

export async function createConversation(providerId: string, title?: string): Promise<ConversationView> {
  return invoke<ConversationView>('create_conversation', { providerId, title })
}

export interface ConversationUpdates {
  title?: string
  providerId?: string
  model?: string
  promptId?: string | null
  searchEnabled?: boolean
}

export async function updateConversation(id: string, updates: ConversationUpdates): Promise<void> {
  return invoke<void>('update_conversation', { id, ...updates })
}

export async function deleteConversation(id: string): Promise<void> {
  return invoke<void>('delete_conversation', { id })
}

export async function pinConversation(id: string): Promise<void> {
  return invoke<void>('pin_conversation', { id })
}

export async function unpinConversation(id: string): Promise<void> {
  return invoke<void>('unpin_conversation', { id })
}
