export interface Message {
  id: string
  conversation_id: string
  role: 'user' | 'assistant' | 'system'
  content: string
  created_at: number
  token_count?: number
}

export interface Conversation {
  id: string
  title: string
  model: string
  system_prompt: string
  created_at: number
  updated_at: number
  pinned: boolean
}

export interface Settings {
  base_url: string
  api_key: string
  model: string
  temperature: number
}
