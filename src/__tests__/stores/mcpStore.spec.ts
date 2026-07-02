import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useMcpStore } from '../../stores/mcpStore'
import * as mcpBridge from '../../bridge/mcp'
import { createMcpCategory, createMcpServer, createMcpInstance } from './helpers'

vi.mock('../../bridge/mcp')
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(vi.fn()),
}))
vi.mock('vue-sonner', () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
  },
}))

describe('mcpStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  describe('loadData', () => {
    it('loads categories, servers, and instances', async () => {
      const categories = [createMcpCategory()]
      const servers = [createMcpServer()]
      const instances = [createMcpInstance()]

      vi.mocked(mcpBridge.listMcpCategories).mockResolvedValue(categories)
      vi.mocked(mcpBridge.listMcpServers).mockResolvedValue(servers)
      vi.mocked(mcpBridge.listMcpInstances).mockResolvedValue(instances)

      const store = useMcpStore()
      await store.loadData()

      expect(mcpBridge.listMcpCategories).toHaveBeenCalledOnce()
      expect(mcpBridge.listMcpServers).toHaveBeenCalledOnce()
      expect(mcpBridge.listMcpInstances).toHaveBeenCalledOnce()
      expect(store.categories).toEqual(categories)
      expect(store.servers).toEqual(servers)
      expect(store.instances).toEqual(instances)
    })

    it('sets first category as active when none is set', async () => {
      const categories = [createMcpCategory({ id: 'cat-1' }), createMcpCategory({ id: 'cat-2', name: '其他' })]

      vi.mocked(mcpBridge.listMcpCategories).mockResolvedValue(categories)
      vi.mocked(mcpBridge.listMcpServers).mockResolvedValue([])
      vi.mocked(mcpBridge.listMcpInstances).mockResolvedValue([])

      const store = useMcpStore()
      await store.loadData()

      expect(store.activeCategoryId).toBe('cat-1')
    })

    it('does not change activeCategoryId if already set', async () => {
      const categories = [createMcpCategory({ id: 'cat-1' }), createMcpCategory({ id: 'cat-2', name: '其他' })]

      vi.mocked(mcpBridge.listMcpCategories).mockResolvedValue(categories)
      vi.mocked(mcpBridge.listMcpServers).mockResolvedValue([])
      vi.mocked(mcpBridge.listMcpInstances).mockResolvedValue([])

      const store = useMcpStore()
      store.activeCategoryId = 'cat-2'
      await store.loadData()

      expect(store.activeCategoryId).toBe('cat-2')
    })
  })

  describe('installServer', () => {
    it('creates instance and prepends to list', async () => {
      const server = createMcpServer({ id: 'server-1', name: 'GitHub', identifier: '@modelcontextprotocol/server-github' })
      const created = createMcpInstance({ id: 'new-instance', server_id: 'server-1' })

      vi.mocked(mcpBridge.addMcpInstance).mockResolvedValue(created)

      const store = useMcpStore()
      await store.installServer(server, {})

      expect(mcpBridge.addMcpInstance).toHaveBeenCalledOnce()
      expect(store.instances).toHaveLength(1)
      expect(store.instances[0].id).toBe('new-instance')
    })

    it('passes config as env', async () => {
      const server = createMcpServer({ id: 'server-1', name: 'GitHub', identifier: '@modelcontextprotocol/server-github' })
      const config = { GITHUB_TOKEN: 'test-token' }

      vi.mocked(mcpBridge.addMcpInstance).mockResolvedValue(createMcpInstance())

      const store = useMcpStore()
      await store.installServer(server, config)

      const calledInstance = vi.mocked(mcpBridge.addMcpInstance).mock.calls[0][0]
      expect(calledInstance.env).toEqual(config)
    })
  })

  describe('uninstallInstance', () => {
    it('removes instance from list', async () => {
      vi.mocked(mcpBridge.removeMcpInstance).mockResolvedValue(undefined)

      const store = useMcpStore()
      store.instances = [createMcpInstance({ id: 'inst-1' }), createMcpInstance({ id: 'inst-2' })]
      store.activeInstanceId = 'inst-1'

      await store.uninstallInstance('inst-1')

      expect(mcpBridge.removeMcpInstance).toHaveBeenCalledWith('inst-1')
      expect(store.instances).toHaveLength(1)
      expect(store.instances[0].id).toBe('inst-2')
    })

    it('clears activeInstanceId when uninstalling active instance', async () => {
      vi.mocked(mcpBridge.removeMcpInstance).mockResolvedValue(undefined)

      const store = useMcpStore()
      store.instances = [createMcpInstance({ id: 'inst-1' })]
      store.activeInstanceId = 'inst-1'

      await store.uninstallInstance('inst-1')

      expect(store.activeInstanceId).toBeNull()
    })

    it('does not clear activeInstanceId when uninstalling inactive instance', async () => {
      vi.mocked(mcpBridge.removeMcpInstance).mockResolvedValue(undefined)

      const store = useMcpStore()
      store.instances = [createMcpInstance({ id: 'inst-1' }), createMcpInstance({ id: 'inst-2' })]
      store.activeInstanceId = 'inst-2'

      await store.uninstallInstance('inst-1')

      expect(store.activeInstanceId).toBe('inst-2')
    })
  })

  describe('toggleInstance', () => {
    it('calls bridge with id and enabled', async () => {
      vi.mocked(mcpBridge.toggleMcpInstance).mockResolvedValue(undefined)

      const store = useMcpStore()
      store.instances = [createMcpInstance({ id: 'inst-1', enabled: false })]

      await store.toggleInstance('inst-1', true)

      expect(mcpBridge.toggleMcpInstance).toHaveBeenCalledWith('inst-1', true)
    })

    it('adds to startingIds when enabling', async () => {
      vi.mocked(mcpBridge.toggleMcpInstance).mockResolvedValue(undefined)

      const store = useMcpStore()
      store.instances = [createMcpInstance({ id: 'inst-1', enabled: false })]

      await store.toggleInstance('inst-1', true)

      expect(store.startingIds.has('inst-1')).toBe(true)
    })

    it('updates enabled immediately when disabling', async () => {
      vi.mocked(mcpBridge.toggleMcpInstance).mockResolvedValue(undefined)

      const store = useMcpStore()
      store.instances = [createMcpInstance({ id: 'inst-1', enabled: true })]

      await store.toggleInstance('inst-1', false)

      expect(store.instances[0].enabled).toBe(false)
    })
  })

  describe('addInstance', () => {
    it('creates instance and prepends to list', async () => {
      const instance = createMcpInstance({ id: 'custom-1', name: 'Custom MCP' })
      vi.mocked(mcpBridge.addMcpInstance).mockResolvedValue(instance)

      const store = useMcpStore()
      await store.addInstance(instance)

      expect(mcpBridge.addMcpInstance).toHaveBeenCalledWith(instance)
      expect(store.instances).toHaveLength(1)
      expect(store.instances[0].id).toBe('custom-1')
    })
  })

  describe('testInstance', () => {
    it('calls bridge and returns result', async () => {
      vi.mocked(mcpBridge.testMcpConnection).mockResolvedValue('OK')

      const store = useMcpStore()
      const result = await store.testInstance('inst-1')

      expect(mcpBridge.testMcpConnection).toHaveBeenCalledWith('inst-1')
      expect(result).toBe('OK')
    })
  })

  describe('getters', () => {
    it('activeCategory returns the active category', () => {
      const store = useMcpStore()
      store.categories = [createMcpCategory({ id: 'cat-1' }), createMcpCategory({ id: 'cat-2', name: '其他' })]
      store.activeCategoryId = 'cat-2'

      expect(store.activeCategory?.id).toBe('cat-2')
    })

    it('activeCategory returns undefined when no active', () => {
      const store = useMcpStore()
      store.categories = [createMcpCategory({ id: 'cat-1' })]
      store.activeCategoryId = null

      expect(store.activeCategory).toBeUndefined()
    })

    it('activeInstance returns the active instance', () => {
      const store = useMcpStore()
      store.instances = [createMcpInstance({ id: 'inst-1' }), createMcpInstance({ id: 'inst-2' })]
      store.activeInstanceId = 'inst-2'

      expect(store.activeInstance?.id).toBe('inst-2')
    })

    it('filteredServers filters by active category', () => {
      const store = useMcpStore()
      store.servers = [
        createMcpServer({ id: 's1', category_id: 'cat-1' }),
        createMcpServer({ id: 's2', category_id: 'cat-2' }),
        createMcpServer({ id: 's3', category_id: 'cat-1' }),
      ]
      store.activeCategoryId = 'cat-1'

      expect(store.filteredServers).toHaveLength(2)
      expect(store.filteredServers.map(s => s.id)).toEqual(['s1', 's3'])
    })

    it('filteredServers returns all when no active category', () => {
      const store = useMcpStore()
      store.servers = [
        createMcpServer({ id: 's1', category_id: 'cat-1' }),
        createMcpServer({ id: 's2', category_id: 'cat-2' }),
      ]
      store.activeCategoryId = null

      expect(store.filteredServers).toHaveLength(2)
    })

    it('installedServerIds returns set of installed server ids', () => {
      const store = useMcpStore()
      store.instances = [
        createMcpInstance({ id: 'inst-1', server_id: 'server-1' }),
        createMcpInstance({ id: 'inst-2', server_id: 'server-2' }),
        createMcpInstance({ id: 'inst-3', server_id: 'server-1' }),
      ]

      expect(store.installedServerIds.size).toBe(2)
      expect(store.installedServerIds.has('server-1')).toBe(true)
      expect(store.installedServerIds.has('server-2')).toBe(true)
      expect(store.installedServerIds.has('server-3')).toBe(false)
    })
  })
})
