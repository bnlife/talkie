<script setup lang="ts">
import { computed } from 'vue'
import { useMcpStore } from '@/stores/mcpStore'
import { Button } from '@/components/ui/button'
import { Search, FolderOpen, Database, Code, Cloud, MessageSquare, Zap, Star, Plus, Check } from 'lucide-vue-next'
import type { McpServer } from '@/types'

const props = defineProps<{
  searchQuery: string
  installedServerIds: Set<string>
}>()

const emit = defineEmits<{
  install: [server: McpServer]
}>()

const mcpStore = useMcpStore()

const categoryIconMap: Record<string, any> = {
  filesystem: FolderOpen,
  database: Database,
  search: Search,
  devtools: Code,
  cloud: Cloud,
  comms: MessageSquare,
  productivity: Zap,
}

function getCategoryIcon(id: string) {
  return categoryIconMap[id] || FolderOpen
}

const filteredServers = computed(() => {
  const q = props.searchQuery.toLowerCase()
  if (!q) return mcpStore.filteredServers
  return mcpStore.servers.filter(s =>
    s.name.toLowerCase().includes(q) || s.description.toLowerCase().includes(q)
  )
})
</script>

<template>
  <div class="mb-3 flex items-center justify-between">
    <div class="flex items-center gap-2">
      <component :is="getCategoryIcon(mcpStore.activeCategoryId || '')" class="size-4 text-muted-foreground" />
      <h2 class="text-sm font-medium">{{ mcpStore.activeCategory?.name }}</h2>
    </div>
    <span class="text-xs text-muted-foreground">{{ filteredServers.length }} 个服务</span>
  </div>
  <div class="grid grid-cols-2 gap-3">
    <div
      v-for="server in filteredServers"
      :key="server.id"
      class="rounded-lg border p-3"
    >
      <div class="mb-1 flex items-center justify-between">
        <span class="text-sm font-medium">{{ server.name }}</span>
        <span class="text-xs text-muted-foreground">{{ server.publisher }}</span>
      </div>
      <p class="mb-2 text-xs text-muted-foreground line-clamp-2">{{ server.description }}</p>
      <div class="flex items-center justify-between">
        <span v-if="server.github_stars" class="inline-flex items-center gap-1 text-xs text-muted-foreground"><Star class="size-3" /> {{ (server.github_stars / 1000).toFixed(0) }}k</span>
        <span v-else />
        <Button
          v-if="!installedServerIds.has(server.id)"
          size="sm"
          variant="secondary"
          class="h-7 text-xs"
          @click="emit('install', server)"
        >
          <Plus class="size-3" />
          添加
        </Button>
        <Button
          v-else
          size="sm"
          variant="secondary"
          class="h-7 text-xs"
          disabled
        >
          <Check class="size-3" />
          已添加
        </Button>
      </div>
    </div>
  </div>
  <div v-if="filteredServers.length === 0" class="flex flex-col items-center py-16 text-muted-foreground">
    <Search class="mb-2 size-8" />
    <p class="text-sm">{{ searchQuery ? '无匹配结果' : '该分类暂无服务' }}</p>
  </div>
</template>
