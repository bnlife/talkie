export interface SearchResult {
  title: string
  url: string
  snippet?: string
}

export interface Message {
  id: string
  conversation_id: string
  role: 'user' | 'assistant' | 'system'
  content: string
  created_at: number
  token_count?: number
  search_results?: SearchResult[]
  thinking_content?: string
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
  pinned: boolean
  created_at: number
  updated_at: number
}

export interface ConversationConfig {
  conversation_id: string
  provider_id: string
  model: string
  prompt_id: string | null
  search_enabled: boolean
}

export type ConversationView = Conversation & ConversationConfig

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

// ---------------------------------------------------------------------------
// MCP types
// ---------------------------------------------------------------------------

export interface McpCategory {
  id: string
  name: string
  icon: string
}

export interface McpEnvVar {
  name: string
  description: string
  required: boolean
  secret: boolean
  default?: string
  choices?: string[]
}

export interface McpArg {
  type: 'positional' | 'named'
  name?: string
  valueHint?: string
  description: string
  required: boolean
  default?: string
  choices?: string[]
  repeated?: boolean
}

export interface McpServer {
  id: string
  category_id: string
  name: string
  description: string
  publisher: string
  registry_type: string
  identifier: string
  transport: 'stdio' | 'sse' | 'http'
  env_vars?: McpEnvVar[]
  args?: McpArg[]
  github_stars?: number
}

export interface McpInstance {
  id: string
  server_id: string
  name: string
  enabled: boolean
  transport: 'stdio' | 'sse' | 'http'
  command?: string
  args?: string[]
  env?: Record<string, string>
  url?: string
  installed_at: number
}
