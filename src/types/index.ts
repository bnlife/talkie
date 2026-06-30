export interface Message {
  id: string
  conversation_id: string
  role: 'user' | 'assistant' | 'system'
  content: string
  created_at: number
  token_count?: number
}

export interface ModelProvider {
  id: string
  name: string
  icon?: string
  base_url: string
  api_key: string
  headers: Record<string, string>
  models: string[]
  enabled: boolean
}

export interface Conversation {
  id: string
  title: string
  provider_id: string
  model: string
  system_prompt: string
  created_at: number
  updated_at: number
  pinned: boolean
}

export interface Settings {
  providers: ModelProvider[]
  active_provider_id: string
  temperature: number
  top_p: number
  last_active_conversation_id?: string
  darkMode?: boolean
}

export interface Prompt {
  id: string
  name: string
  content: string
  is_default: boolean
  created_at: number
  updated_at: number
}
