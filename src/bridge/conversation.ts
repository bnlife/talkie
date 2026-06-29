import { invoke } from '@tauri-apps/api/core'
import type { Conversation } from '../types'

export async function listConversations(): Promise<Conversation[]> {
  return invoke<Conversation[]>('list_conversations')
}

export async function createConversation(title?: string): Promise<Conversation> {
  return invoke<Conversation>('create_conversation', { title })
}

export async function updateConversation(id: string, title: string): Promise<void> {
  return invoke<void>('update_conversation', { id, title })
}

export async function deleteConversation(id: string): Promise<void> {
  return invoke<void>('delete_conversation', { id })
}
