import { invoke } from '@tauri-apps/api/core'
import type { Conversation } from '../types'

export async function listConversations(): Promise<Conversation[]> {
  return invoke<Conversation[]>('list_conversations')
}

export async function createConversation(providerId: string, title?: string): Promise<Conversation> {
  return invoke<Conversation>('create_conversation', { providerId, title })
}

export async function updateConversation(id: string, title?: string, providerId?: string, model?: string, searchEnabled?: boolean): Promise<void> {
  return invoke<void>('update_conversation', { id, title, providerId, model, searchEnabled })
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
