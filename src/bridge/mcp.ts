import { invoke } from '@tauri-apps/api/core'
import type { McpCategory, McpServer, McpInstance } from '../types'

// ---------------------------------------------------------------------------
// Market (read-only, from built-in registry)
// ---------------------------------------------------------------------------

export async function listMcpCategories(): Promise<McpCategory[]> {
  return invoke<McpCategory[]>('list_mcp_categories')
}

export async function listMcpServers(categoryId?: string): Promise<McpServer[]> {
  return invoke<McpServer[]>('list_mcp_servers', { categoryId })
}

// ---------------------------------------------------------------------------
// Instances (user's installed services)
// ---------------------------------------------------------------------------

export async function listMcpInstances(): Promise<McpInstance[]> {
  return invoke<McpInstance[]>('list_mcp_instances')
}

export async function addMcpInstance(instance: Omit<McpInstance, 'id' | 'installed_at'>): Promise<McpInstance> {
  return invoke<McpInstance>('add_mcp_instance', { instance })
}

export async function removeMcpInstance(id: string): Promise<void> {
  return invoke<void>('remove_mcp_instance', { id })
}

export async function toggleMcpInstance(id: string, enabled: boolean): Promise<void> {
  return invoke<void>('toggle_mcp_instance', { id, enabled })
}

// ---------------------------------------------------------------------------
// MCP runtime
// ---------------------------------------------------------------------------

export async function startMcpInstance(id: string): Promise<void> {
  return invoke<void>('start_mcp_instance', { id })
}

export async function stopMcpInstance(id: string): Promise<void> {
  return invoke<void>('stop_mcp_instance', { id })
}

export async function callMcpTool(instanceId: string, toolName: string, args: Record<string, unknown>): Promise<unknown> {
  return invoke<unknown>('call_mcp_tool', { instanceId, toolName, args })
}

export async function testMcpConnection(id: string): Promise<string> {
  return invoke<string>('test_mcp_connection', { id })
}
