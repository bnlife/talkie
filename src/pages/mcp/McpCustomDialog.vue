<script setup lang="ts">
import { ref, watch } from 'vue'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'

const props = defineProps<{
  visible: boolean
}>()

const emit = defineEmits<{
  confirm: [form: { name: string; command: string; args: string; envKey: string; envValue: string }]
  cancel: []
}>()

const customForm = ref({
  name: '',
  command: 'cmd',
  args: '/c npx -y @humansean/mcp-bocha',
  envKey: 'BOCHA_API_KEY',
  envValue: '',
})

watch(() => props.visible, (visible) => {
  if (visible) {
    customForm.value = {
      name: '',
      command: 'cmd',
      args: '/c npx -y @humansean/mcp-bocha',
      envKey: 'BOCHA_API_KEY',
      envValue: '',
    }
  }
})

function handleConfirm() {
  emit('confirm', customForm.value)
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="visible"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
      @click.self="emit('cancel')"
    >
      <div class="w-96 rounded-lg border bg-background p-4 shadow-lg">
        <h3 class="text-sm font-medium mb-3">添加自定义 MCP 服务</h3>

        <div class="space-y-2 mb-3">
          <div>
            <label class="text-xs text-muted-foreground">名称</label>
            <Input v-model="customForm.name" placeholder="我的搜索服务" class="h-7 text-sm mt-0.5" />
          </div>
          <div>
            <label class="text-xs text-muted-foreground">命令</label>
            <Input v-model="customForm.command" placeholder="npx" class="h-7 text-sm mt-0.5" />
          </div>
          <div>
            <label class="text-xs text-muted-foreground">参数（空格分隔）</label>
            <Input v-model="customForm.args" placeholder="-y @humansean/mcp-bocha" class="h-7 text-sm mt-0.5" />
          </div>
          <div>
            <label class="text-xs text-muted-foreground">环境变量 Key</label>
            <Input v-model="customForm.envKey" placeholder="BOCHA_API_KEY" class="h-7 text-sm mt-0.5" />
          </div>
          <div>
            <label class="text-xs text-muted-foreground">环境变量 Value</label>
            <Input v-model="customForm.envValue" type="password" placeholder="sk-xxx" class="h-7 text-sm mt-0.5" />
          </div>
        </div>

        <div class="flex justify-end gap-2">
          <Button size="sm" variant="outline" @click="emit('cancel')">取消</Button>
          <Button size="sm" @click="handleConfirm">确认添加</Button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
