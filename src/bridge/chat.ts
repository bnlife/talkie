import { invoke } from '@tauri-apps/api/core'
import type { Message } from '../types'

export async function sendMessage(conversationId: string, content: string, searchEnabled?: boolean): Promise<void> {
  return invoke<void>('send_message', { conversationId, content, searchEnabled: searchEnabled ?? false })
}

export async function stopStream(): Promise<void> {
  return invoke<void>('stop_stream')
}

export async function getMessages(conversationId: string): Promise<Message[]> {
  return invoke<Message[]>('get_messages', { conversationId })
}

export async function deleteMessage(messageId: string): Promise<void> {
  return invoke<void>('delete_message', { messageId })
}

export async function regenerateMessage(conversationId: string): Promise<void> {
  return invoke<void>('regenerate_message', { conversationId })
}
