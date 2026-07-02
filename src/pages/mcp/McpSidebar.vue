<script setup lang="ts">
import { computed } from 'vue'
import { useMcpStore } from '@/stores/mcpStore'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Plus, Search, Trash2, Settings } from 'lucide-vue-next'
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

function getInstanceStatus(inst: McpInstance): 'running' | 'starting' | 'stopped' {
  if (mcpStore.startingIds.has(inst.id)) return 'starting'
  if (inst.enabled) return 'running'
  return 'stopped'
}
</script>

<template>
  <div class="flex w-[220px] shrink-0 flex-col gap-1 border-r p-1.5 text-sm">
    <!-- 搜索 -->
    <div class="relative">
      <Search class="absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground" />
      <Input
        :value="searchQuery"
        placeholder="搜索 MCP 服务..."
        class="h-7 pl-8 text-sm"
        @input="emit('update:searchQuery', ($event.target as HTMLInputElement).value)"
      />
    </div>

    <!-- 添加自定义 -->
    <div
      class="flex cursor-pointer items-center gap-2 rounded-md border border-dashed px-2 py-1.5 transition-colors hover:bg-foreground/5"
      @click="emit('openCustomDialog')"
    >
      <Plus class="size-3.5" />
      <span>添加自定义服务</span>
    </div>

    <div class="my-1 border-t" />

    <!-- 市场分类 -->
    <div class="text-xs font-medium text-foreground px-1">市场</div>
    <div
      v-for="cat in mcpStore.categories"
      :key="cat.id"
      :class="cn(
        'flex cursor-pointer items-center gap-2 rounded-md px-2 py-1.5 transition-colors hover:bg-foreground/5',
        mcpStore.activeCategoryId === cat.id && 'bg-accent text-accent-foreground',
      )"
      @click="emit('selectCategory', cat.id)"
    >
      <span>{{ cat.icon }}</span>
      <span class="truncate text-sm text-muted-foreground">{{ cat.name }}</span>
    </div>

    <div class="my-1 border-t" />

    <!-- 已安装 -->
    <div class="text-xs font-medium text-foreground px-1">已安装</div>
    <div class="flex-1 overflow-y-auto">
      <div
        v-for="inst in mcpStore.instances"
        :key="inst.id"
        :class="cn(
          'group flex cursor-pointer items-center justify-between rounded-md px-2 py-1.5 transition-colors hover:bg-foreground/5',
          mcpStore.activeInstanceId === inst.id && 'bg-accent text-accent-foreground',
        )"
        @click="emit('selectInstance', inst.id)"
      >
        <div class="flex min-w-0 items-center gap-2">
          <span
            :class="cn(
              'size-1.5 shrink-0 rounded-full',
              getInstanceStatus(inst) === 'running' ? 'bg-green-500' :
              getInstanceStatus(inst) === 'starting' ? 'bg-yellow-500 animate-pulse' :
              'bg-muted-foreground/30',
            )"
          />
          <span class="truncate text-sm text-muted-foreground">{{ inst.name }}</span>
        </div>
        <div class="flex shrink-0 items-center gap-0.5 opacity-0 group-hover:opacity-100">
          <Button
            variant="ghost"
            size="icon-sm"
            class="size-5"
            @click.stop="emit('toggleInstance', inst.id, inst.enabled)"
          >
            <Settings class="size-3" />
          </Button>
          <Button
            variant="ghost"
            size="icon-sm"
            class="size-5"
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
