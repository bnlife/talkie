<script setup lang="ts">
import { computed } from 'vue'
import { useMcpStore } from '@/stores/mcpStore'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Plus, Search, Trash2, Settings, FolderOpen, Database, Code, Cloud, MessageSquare, Zap } from 'lucide-vue-next'
import type { McpInstance } from '@/types'

const props = defineProps<{
  searchQuery: string
}>()

const emit = defineEmits<{
  'update:searchQuery': [value: string]
  selectCategory: [id: string]
  selectInstance: [id: string]
  openCustomDialog: []
  toggleInstance: [id: string, enabled: boolean]
  uninstallInstance: [id: string]
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

function getInstanceStatus(inst: McpInstance): 'running' | 'starting' | 'stopped' {
  if (mcpStore.startingIds.has(inst.id)) return 'starting'
  if (inst.enabled) return 'running'
  return 'stopped'
}
</script>

<template>
  <div class="sidebar-container h-full w-[220px] shrink-0 border-r">
    <!-- 搜索 -->
    <div class="relative">
      <Search class="absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground" />
      <Input
        :value="searchQuery"
        placeholder="搜索 MCP 服务..."
        size="sidebar"
        class="sidebar-search"
        @input="emit('update:searchQuery', ($event.target as HTMLInputElement).value)"
      />
    </div>

    <!-- 添加自定义 -->
    <div
      class="sidebar-action"
      @click="emit('openCustomDialog')"
    >
      <Plus class="size-3.5" />
      <span>添加自定义服务</span>
    </div>

    <div class="my-1 border-t" />

    <!-- 市场分类 -->
    <div class="sidebar-section-title">市场</div>
    <div>
      <div
        v-for="cat in mcpStore.categories"
        :key="cat.id"
        :class="cn(
          'sidebar-item',
          mcpStore.activeCategoryId === cat.id && 'bg-accent text-accent-foreground',
        )"
        @click="emit('selectCategory', cat.id)"
      >
        <div class="sidebar-item-content">
          <component :is="getCategoryIcon(cat.id)" class="size-3.5 shrink-0 text-muted-foreground" />
          <span class="truncate text-sm text-muted-foreground">{{ cat.name }}</span>
        </div>
      </div>
    </div>

    <div class="my-1 border-t" />

    <!-- 已安装 -->
    <div class="sidebar-section-title">已安装</div>
    <div class="flex-1 overflow-y-auto" style="scrollbar-gutter: stable">
      <div
        v-for="inst in mcpStore.instances"
        :key="inst.id"
        :class="cn(
          'group sidebar-item',
          mcpStore.activeInstanceId === inst.id && 'bg-accent text-accent-foreground',
        )"
        @click="emit('selectInstance', inst.id)"
      >
        <div class="sidebar-item-content">
          <span
            :class="cn(
              'size-1.5 shrink-0 rounded-full',
              getInstanceStatus(inst) === 'running' ? 'bg-success' :
              getInstanceStatus(inst) === 'starting' ? 'bg-warning animate-pulse' :
              'bg-muted-foreground/30',
            )"
          />
          <span class="truncate text-sm text-muted-foreground">{{ inst.name }}</span>
        </div>
        <div class="sidebar-item-actions opacity-0 group-hover:opacity-100">
          <Button
            variant="ghost"
            size="icon"
            @click.stop="emit('toggleInstance', inst.id, inst.enabled)"
          >
            <Settings class="size-3" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            @click.stop="emit('uninstallInstance', inst.id)"
          >
            <Trash2 class="size-3" />
          </Button>
        </div>
      </div>

      <div v-if="mcpStore.instances.length === 0" class="py-4 text-center text-xs text-muted-foreground">
        暂无已安装服务
      </div>
    </div>
  </div>
</template>
