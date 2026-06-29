import { describe, it, expect, vi, beforeEach } from 'vitest'
import { invoke } from '@tauri-apps/api/core'

import { sendMessage, stopStream, getMessages } from '../bridge/chat'
import {
  listConversations,
  createConversation,
  updateConversation,
  deleteConversation,
} from '../bridge/conversation'
import { getSettings, updateSettings, testConnection } from '../bridge/settings'

const mockedInvoke = vi.mocked(invoke)

beforeEach(() => {
  mockedInvoke.mockReset()
})

// ---------------------------------------------------------------------------
// chat bridge
// ---------------------------------------------------------------------------
describe('chat bridge', () => {
  it('sendMessage calls invoke with correct arguments', async () => {
    mockedInvoke.mockResolvedValue(undefined)
    await sendMessage('conv-1', 'hello')
    expect(mockedInvoke).toHaveBeenCalledWith('send_message', {
      conversationId: 'conv-1',
      content: 'hello',
    })
  })

  it('stopStream calls invoke with no arguments', async () => {
    mockedInvoke.mockResolvedValue(undefined)
    await stopStream()
    expect(mockedInvoke).toHaveBeenCalledWith('stop_stream')
  })

  it('getMessages returns messages from invoke', async () => {
    const mockMessages = [
      { id: '1', conversation_id: 'c1', role: 'user', content: 'hi', created_at: 100 },
    ]
    mockedInvoke.mockResolvedValue(mockMessages)
    const result = await getMessages('conv-1')
    expect(mockedInvoke).toHaveBeenCalledWith('get_messages', {
      conversationId: 'conv-1',
    })
    expect(result).toEqual(mockMessages)
  })
})

// ---------------------------------------------------------------------------
// conversation bridge
// ---------------------------------------------------------------------------
describe('conversation bridge', () => {
  const mockConv = {
    id: 'c1',
    title: 'Test',
    model: 'deepseek-chat',
    system_prompt: '',
    created_at: 0,
    updated_at: 0,
  }

  it('listConversations returns conversations from invoke', async () => {
    mockedInvoke.mockResolvedValue([mockConv])
    const result = await listConversations()
    expect(mockedInvoke).toHaveBeenCalledWith('list_conversations')
    expect(result).toEqual([mockConv])
  })

  it('createConversation calls invoke with optional title', async () => {
    mockedInvoke.mockResolvedValue(mockConv)
    const result = await createConversation('New Chat')
    expect(mockedInvoke).toHaveBeenCalledWith('create_conversation', {
      title: 'New Chat',
    })
    expect(result).toEqual(mockConv)
  })

  it('createConversation works without title', async () => {
    mockedInvoke.mockResolvedValue(mockConv)
    const result = await createConversation()
    expect(mockedInvoke).toHaveBeenCalledWith('create_conversation', {
      title: undefined,
    })
    expect(result).toEqual(mockConv)
  })

  it('updateConversation calls invoke with correct arguments', async () => {
    mockedInvoke.mockResolvedValue(undefined)
    await updateConversation('c1', 'Renamed')
    expect(mockedInvoke).toHaveBeenCalledWith('update_conversation', {
      id: 'c1',
      title: 'Renamed',
    })
  })

  it('deleteConversation calls invoke with correct id', async () => {
    mockedInvoke.mockResolvedValue(undefined)
    await deleteConversation('c1')
    expect(mockedInvoke).toHaveBeenCalledWith('delete_conversation', { id: 'c1' })
  })
})

// ---------------------------------------------------------------------------
// settings bridge
// ---------------------------------------------------------------------------
describe('settings bridge', () => {
  const mockSettings = {
    base_url: 'https://api.deepseek.com/v1',
    api_key: 'sk-xxx',
    model: 'deepseek-chat',
    temperature: 0.7,
  }

  it('getSettings returns settings from invoke', async () => {
    mockedInvoke.mockResolvedValue(mockSettings)
    const result = await getSettings()
    expect(mockedInvoke).toHaveBeenCalledWith('get_settings')
    expect(result).toEqual(mockSettings)
  })

  it('updateSettings calls invoke with correct arguments', async () => {
    mockedInvoke.mockResolvedValue(undefined)
    await updateSettings({ model: 'gpt-4' })
    expect(mockedInvoke).toHaveBeenCalledWith('update_settings', {
      settings: { model: 'gpt-4' },
    })
  })

  it('testConnection returns result from invoke', async () => {
    mockedInvoke.mockResolvedValue({ ok: true })
    const result = await testConnection(mockSettings)
    expect(mockedInvoke).toHaveBeenCalledWith('test_connection', {
      settings: mockSettings,
    })
    expect(result).toEqual({ ok: true })
  })

  it('testConnection propagates error from invoke', async () => {
    mockedInvoke.mockResolvedValue({ ok: false, error: 'connection refused' })
    const result = await testConnection(mockSettings)
    expect(result).toEqual({ ok: false, error: 'connection refused' })
  })
})
