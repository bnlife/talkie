import { invoke } from '@tauri-apps/api/core'
import type { Message, MessagesPage, AttachmentMeta } from '../types'

export async function sendMessage(conversationId: string, content: string, attachments?: AttachmentMeta[], searchEnabled?: boolean, searchEngine?: string): Promise<void> {
  return invoke<void>('send_message', { conversationId, content, attachments: attachments ?? null, searchEnabled: searchEnabled ?? false, searchEngine: searchEngine ?? null })
}

export async function stopStream(): Promise<void> {
  return invoke<void>('stop_stream')
}

export async function getMessages(conversationId: string, offset?: number, limit?: number): Promise<MessagesPage> {
  return invoke<MessagesPage>('get_messages', { conversationId, offset: offset ?? null, limit: limit ?? null })
}

export async function deleteMessage(messageId: string): Promise<void> {
  return invoke<void>('delete_message', { messageId })
}

export async function regenerateMessage(conversationId: string): Promise<void> {
  return invoke<void>('regenerate_message', { conversationId })
}
