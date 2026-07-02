<script setup lang="ts">
import { ref, watch } from 'vue'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import type { McpServer } from '@/types'

const props = defineProps<{
  server: McpServer | null
}>()

const emit = defineEmits<{
  confirm: [config: Record<string, string>]
  cancel: []
}>()

const installConfig = ref<Record<string, string>>({})

watch(() => props.server, (server) => {
  if (server) {
    const config: Record<string, string> = {}
    if (server.env_vars) {
      for (const v of server.env_vars) {
        if (v.default) config[v.name] = v.default
      }
    }
    if (server.args) {
      for (const a of server.args) {
        if (a.default) config[a.valueHint || a.name || ''] = a.default
      }
    }
    installConfig.value = config
  }
}, { immediate: true })

function handleConfirm() {
  emit('confirm', installConfig.value)
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="server"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
      @click.self="emit('cancel')"
    >
      <div class="w-96 rounded-lg border bg-background p-4 shadow-lg">
        <h3 class="text-sm font-medium mb-3">添加: {{ server.name }}</h3>

        <!-- 需要用户填写的参数 -->
        <div v-if="server.env_vars && server.env_vars.length > 0" class="space-y-2 mb-3">
          <div v-for="v in server.env_vars" :key="v.name">
            <label class="text-xs text-muted-foreground">
              {{ v.name }}
              <span v-if="v.required" class="text-error">*</span>
              <span v-if="v.description" class="ml-1 text-xs text-muted-foreground">({{ v.description }})</span>
            </label>
            <Input
              v-model="installConfig[v.name]"
              :type="v.secret ? 'password' : 'text'"
              :placeholder="v.default || ''"
              class="h-7 text-sm mt-0.5"
            />
          </div>
        </div>

        <div v-if="server.args && server.args.length > 0" class="space-y-2 mb-3">
          <div v-for="a in server.args" :key="a.valueHint || a.name || ''">
            <label class="text-xs text-muted-foreground">
              {{ a.description }}
              <span v-if="a.required" class="text-error">*</span>
            </label>
            <Input
              v-model="installConfig[a.valueHint || a.name || '']"
              :placeholder="a.default || ''"
              class="h-7 text-sm mt-0.5"
            />
          </div>
        </div>

        <div v-if="(!server.env_vars || server.env_vars.length === 0) && (!server.args || server.args.length === 0)" class="mb-3 text-xs text-muted-foreground">
          此服务无需额外配置，点击确认即可添加。
        </div>

        <div class="flex justify-end gap-2">
          <Button size="default" variant="secondary" @click="emit('cancel')">取消</Button>
          <Button size="default" @click="handleConfirm">确认添加</Button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
