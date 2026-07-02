<script setup lang="ts">
import { ref, watch } from 'vue'
import type { ModelProvider } from '@/types'
import { useSettingsStore } from '@/stores/settingsStore'
import { toast } from 'vue-sonner'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Check, X as XIcon } from 'lucide-vue-next'

const props = defineProps<{
  visible: boolean
  provider: ModelProvider
}>()

const settingsStore = useSettingsStore()

const emit = defineEmits<{
  confirm: [modelName: string]
  cancel: []
}>()

const modelName = ref('')
const isChecking = ref(false)
const checkResult = ref<{ ok: boolean; error?: string } | null>(null)

watch(() => props.visible, (visible) => {
  if (visible) {
    modelName.value = ''
    checkResult.value = null
  }
})

watch(modelName, () => {
  checkResult.value = null
})

async function handleCheck() {
  if (!modelName.value.trim()) {
    toast.warning('请输入模型名称')
    return
  }
  isChecking.value = true
  checkResult.value = null

  try {
    const result = await settingsStore.verifyModel(props.provider.id, modelName.value.trim())
    checkResult.value = result
  } catch {
    checkResult.value = { ok: false, error: '检测失败' }
  } finally {
    isChecking.value = false
  }
}

function handleConfirm() {
  if (!modelName.value.trim()) {
    toast.warning('请输入模型名称')
    return
  }
  emit('confirm', modelName.value.trim())
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="visible"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
    >
      <div class="w-96 rounded-lg border bg-background p-4 shadow-lg">
        <h3 class="text-sm font-medium mb-3">添加模型</h3>

        <div class="space-y-2 mb-3">
          <div>
            <div class="flex items-center border rounded-md h-8">
              <div class="relative flex-1">
                <Input
                  v-model="modelName"
                  placeholder="模型名称，例如: gpt-4o"
                  size="inline"
                  class="pr-10"
                  @keyup.enter="handleCheck"
                />
                <div class="absolute right-1 top-1/2 flex -translate-y-1/2 items-center gap-0.5">
                  <Check v-if="checkResult?.ok" class="size-3.5 text-success" />
                  <XIcon v-else-if="checkResult && !checkResult.ok" class="size-3.5 text-error" />
                </div>
              </div>
              <div class="border-l h-7" />
              <Button
                size="default"
                :variant="checkResult?.ok ? 'default' : checkResult && !checkResult.ok ? 'destructive' : 'ghost'"
                class="h-full rounded-none px-3"
                :disabled="isChecking || !modelName.trim()"
                @click="handleCheck"
              >
                {{ isChecking ? '检测中...' : '检测' }}
              </Button>
            </div>
            <div class="min-h-[2rem] mt-1">
              <div v-if="checkResult" class="text-xs line-clamp-2 break-all" :class="checkResult.ok ? 'text-success' : 'text-error'">
                {{ checkResult.ok ? '模型可用' : checkResult.error }}
              </div>
            </div>
          </div>
        </div>

        <div class="flex justify-end gap-2">
          <Button size="default" variant="secondary" @click="emit('cancel')">退出</Button>
          <Button size="default" :disabled="!modelName.trim() || (checkResult && !checkResult.ok)" @click="handleConfirm">确认</Button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
