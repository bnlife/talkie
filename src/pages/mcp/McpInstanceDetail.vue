<script setup lang="ts">
import { computed } from 'vue'
import { useMcpStore } from '@/stores/mcpStore'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import type { McpInstance } from '@/types'

const props = defineProps<{
  testingId: string | null
  testResult: { id: string; ok: boolean; msg: string } | null
}>()

const emit = defineEmits<{
  toggle: [id: string, enabled: boolean]
  test: [id: string]
  uninstall: [id: string]
}>()

const mcpStore = useMcpStore()

function getInstanceStatus(inst: McpInstance): 'running' | 'starting' | 'stopped' {
  if (mcpStore.startingIds.has(inst.id)) return 'starting'
  if (inst.enabled) return 'running'
  return 'stopped'
}

function getInstanceStatusText(inst: McpInstance): string {
  const status = getInstanceStatus(inst)
  if (status === 'starting') return '启动中...'
  if (status === 'running') return '运行中'
  return '已暂停'
}
</script>

<template>
  <div v-if="mcpStore.activeInstance">
    <div class="mb-3">
      <h2 class="text-sm font-medium">{{ mcpStore.activeInstance.name }}</h2>
      <p class="text-xs text-muted-foreground mt-1">
        状态：
        <span :class="{
          'text-success': getInstanceStatus(mcpStore.activeInstance) === 'running',
          'text-warning': getInstanceStatus(mcpStore.activeInstance) === 'starting',
          'text-muted-foreground': getInstanceStatus(mcpStore.activeInstance) === 'stopped',
        }">
          {{ getInstanceStatusText(mcpStore.activeInstance) }}
        </span>
        <span v-if="mcpStore.errorMap[mcpStore.activeInstance.id]" class="text-error ml-2">
          ({{ mcpStore.errorMap[mcpStore.activeInstance.id] }})
        </span>
      </p>
    </div>
    <div class="space-y-3">
      <div>
        <div class="text-xs font-medium text-foreground mb-1">配置</div>
        <div class="rounded-md border p-2 text-xs font-mono text-muted-foreground">
          <div>传输方式: {{ mcpStore.activeInstance.transport }}</div>
          <div v-if="mcpStore.activeInstance.command">命令: {{ mcpStore.activeInstance.command }} {{ mcpStore.activeInstance.args?.join(' ') }}</div>
          <div v-if="mcpStore.activeInstance.url">URL: {{ mcpStore.activeInstance.url }}</div>
          <div v-if="mcpStore.activeInstance.env">
            环境变量:
            <div v-for="(v, k) in mcpStore.activeInstance.env" :key="k" class="ml-2">
              {{ k }}={{ v }}
            </div>
          </div>
        </div>
      </div>
      <div class="flex gap-2">
        <Button
          size="default"
          :variant="getInstanceStatus(mcpStore.activeInstance) === 'running' ? 'default' : 'secondary'"
          :disabled="getInstanceStatus(mcpStore.activeInstance) === 'starting'"
          @click="emit('toggle', mcpStore.activeInstance.id, mcpStore.activeInstance.enabled)"
        >
          {{ getInstanceStatus(mcpStore.activeInstance) === 'starting' ? '启动中...' :
             getInstanceStatus(mcpStore.activeInstance) === 'running' ? '暂停' : '启动' }}
        </Button>
        <Button
          size="default"
          variant="secondary"
          :disabled="testingId === mcpStore.activeInstance.id"
          @click="emit('test', mcpStore.activeInstance.id)"
        >
          {{ testingId === mcpStore.activeInstance.id ? '测试中...' : '测试连接' }}
        </Button>
        <Button
          size="default"
          variant="destructive"
          @click="emit('uninstall', mcpStore.activeInstance.id)"
        >
          移除
        </Button>
      </div>
      <div
        v-if="testResult && testResult.id === mcpStore.activeInstance.id"
        :class="cn(
          'mt-2 rounded-md border px-3 py-2 text-xs',
          testResult.ok ? 'border-success/30 bg-success/5 text-success' : 'border-error/30 bg-error/5 text-error',
        )"
      >
        {{ testResult.msg }}
      </div>
    </div>
  </div>
</template>
