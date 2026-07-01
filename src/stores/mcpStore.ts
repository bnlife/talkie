import { defineStore } from 'pinia'
import { listen } from '@tauri-apps/api/event'
import { toast } from 'vue-sonner'
import type { McpCategory, McpServer, McpInstance } from '../types'
import * as mcpBridge from '../bridge/mcp'
import { log } from '../bridge/log'
import { EVENTS } from '../lib/events'

export const useMcpStore = defineStore('mcp', {
  state: () => ({
    categories: [] as McpCategory[],
    servers: [] as McpServer[],
    instances: [] as McpInstance[],
    activeCategoryId: null as string | null,
    activeInstanceId: null as string | null,
    startingIds: new Set<string>(),
    errorMap: {} as Record<string, string>,
  }),

  getters: {
    activeCategory(state): McpCategory | undefined {
      return state.categories.find(c => c.id === state.activeCategoryId)
    },
    activeInstance(state): McpInstance | undefined {
      return state.instances.find(i => i.id === state.activeInstanceId)
    },
    filteredServers(state): McpServer[] {
      if (!state.activeCategoryId) return state.servers
      return state.servers.filter(s => s.category_id === state.activeCategoryId)
    },
    installedServerIds(state): Set<string> {
      return new Set(state.instances.map(i => i.server_id))
    },
  },

  actions: {
    async loadData(): Promise<void> {
      await log('info', 'FE::mcpStore | load')
      const [cats, servers, instances] = await Promise.all([
        mcpBridge.listMcpCategories(),
        mcpBridge.listMcpServers(),
        mcpBridge.listMcpInstances(),
      ])
      this.categories = cats
      this.servers = servers
      this.instances = instances
      if (!this.activeCategoryId && cats.length > 0) {
        this.activeCategoryId = cats[0].id
      }
    },

    async listenEvents(): Promise<void> {
      await listen(EVENTS.MCP_STARTED, (event) => {
        const payload = event.payload as { id: string }
        log('info', `FE::mcpStore | started | id=${payload.id}`)
        this.startingIds.delete(payload.id)
        delete this.errorMap[payload.id]
        const inst = this.instances.find(i => i.id === payload.id)
        if (inst) {
          inst.enabled = true
          toast.success(`${inst.name} 已启动`)
        }
      })

      await listen(EVENTS.MCP_ERROR, (event) => {
        const payload = event.payload as { id: string; error: string }
        log('error', `FE::mcpStore | start fail | id=${payload.id} err=${payload.error}`)
        this.startingIds.delete(payload.id)
        this.errorMap[payload.id] = payload.error
        const inst = this.instances.find(i => i.id === payload.id)
        if (inst) {
          inst.enabled = false
          toast.error(`${inst.name} 启动失败: ${payload.error}`)
        }
      })
    },

    async installServer(server: McpServer, config: Record<string, string>): Promise<void> {
      await log('info', `FE::mcpStore | install | server=${server.id}`)
      const now = Math.floor(Date.now() / 1000)
      const instance: McpInstance = {
        id: crypto.randomUUID(),
        server_id: server.id,
        name: server.name,
        enabled: false,
        transport: server.transport as 'stdio' | 'sse' | 'http',
        command: 'cmd',
        args: ['/c', 'npx', '-y', server.identifier],
        env: Object.keys(config).length > 0 ? config : undefined,
        installed_at: now,
      }
      const created = await mcpBridge.addMcpInstance(instance)
      this.instances.unshift(created)
    },

    async uninstallInstance(id: string): Promise<void> {
      await log('info', `FE::mcpStore | uninstall | id=${id}`)
      await mcpBridge.removeMcpInstance(id)
      this.instances = this.instances.filter(i => i.id !== id)
      this.startingIds.delete(id)
      delete this.errorMap[id]
      if (this.activeInstanceId === id) this.activeInstanceId = null
    },

    async toggleInstance(id: string, enabled: boolean): Promise<void> {
      await log('info', `FE::mcpStore | toggle | id=${id} enabled=${enabled}`)

      if (enabled) {
        // Mark as starting — UI shows "启动中..."
        this.startingIds.add(id)
        delete this.errorMap[id]
      }

      // Update DB + trigger background spawn (returns immediately)
      await mcpBridge.toggleMcpInstance(id, enabled)

      if (!enabled) {
        // Stop is synchronous, update immediately
        const inst = this.instances.find(i => i.id === id)
        if (inst) inst.enabled = false
      }
      // enabled=true: don't update inst.enabled yet — wait for mcp:started event
    },

    async addInstance(instance: McpInstance): Promise<void> {
      await log('info', `FE::mcpStore | add custom | name=${instance.name}`)
      const created = await mcpBridge.addMcpInstance(instance)
      this.instances.unshift(created)
    },
  },
})
