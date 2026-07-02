import type {
  ConversationView,
  Message,
  Settings,
  Prompt,
  ModelProvider,
  McpCategory,
  McpServer,
  McpInstance,
} from '../../types'

export function createConv(overrides: Partial<ConversationView> = {}): ConversationView {
  return {
    id: 'conv-1',
    title: 'Test',
    conversation_id: 'conv-1',
    provider_id: 'prov-1',
    model: 'deepseek-chat',
    prompt_id: null,
    search_enabled: false,
    search_engine: '',
    created_at: 0,
    updated_at: 0,
    pinned: false,
    ...overrides,
  }
}

export function createMsg(overrides: Partial<Message> = {}): Message {
  return {
    id: 'msg-1',
    conversation_id: 'conv-1',
    role: 'user',
    content: 'hello',
    created_at: 100,
    ...overrides,
  }
}

export function createPrompt(overrides: Partial<Prompt> = {}): Prompt {
  return {
    id: 'prompt-1',
    name: '翻译助手',
    content: '你是一个翻译助手',
    is_default: false,
    created_at: 1000,
    updated_at: 1000,
    ...overrides,
  }
}

export function createProvider(overrides: Partial<ModelProvider> = {}): ModelProvider {
  return {
    id: 'prov-1',
    name: 'DeepSeek',
    icon: 'Sparkles',
    base_url: 'https://api.deepseek.com/v1',
    api_key: 'sk-test',
    headers: {},
    models: ['deepseek-chat'],
    enabled: true,
    ...overrides,
  }
}

export function createMcpCategory(overrides: Partial<McpCategory> = {}): McpCategory {
  return {
    id: 'cat-1',
    name: '开发工具',
    icon: 'Code',
    ...overrides,
  }
}

export function createMcpServer(overrides: Partial<McpServer> = {}): McpServer {
  return {
    id: 'server-1',
    category_id: 'cat-1',
    name: 'GitHub',
    description: 'GitHub API 集成',
    publisher: 'GitHub',
    registry_type: 'npm',
    identifier: '@modelcontextprotocol/server-github',
    transport: 'stdio',
    ...overrides,
  }
}

export function createMcpInstance(overrides: Partial<McpInstance> = {}): McpInstance {
  return {
    id: 'instance-1',
    server_id: 'server-1',
    name: 'GitHub',
    enabled: false,
    transport: 'stdio',
    command: 'cmd',
    args: ['/c', 'npx', '-y', '@modelcontextprotocol/server-github'],
    installed_at: Math.floor(Date.now() / 1000),
    ...overrides,
  }
}
