<script setup lang="ts">
import { ref } from 'vue'
import { Button } from '@/components/ui/button'
import { Copy, Trash2, RefreshCw, Check } from 'lucide-vue-next'

const props = defineProps<{
  content: string
  messageId: string
  isUser: boolean
  isLast?: boolean
  tokenCount?: number
}>()

const emit = defineEmits<{
  copy: [content: string]
  delete: [messageId: string]
  regenerate: []
}>()

const isCopied = ref(false)

async function handleCopy() {
  await navigator.clipboard.writeText(props.content)
  isCopied.value = true
  setTimeout(() => { isCopied.value = false }, 2000)
  emit('copy', props.content)
}
</script>

<template>
  <div>
    <!-- 操作按钮 -->
    <div
      class="absolute -bottom-7 left-0 flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity"
    >
      <Button variant="ghost" size="icon" @click="handleCopy">
        <Check v-if="isCopied" class="h-3 w-3 text-success" />
        <Copy v-else class="h-3 w-3" />
      </Button>
      <Button variant="ghost" size="icon" @click="emit('delete', messageId)">
        <Trash2 class="h-3 w-3" />
      </Button>
      <Button v-if="!isUser && isLast" variant="ghost" size="icon" @click="emit('regenerate')">
        <RefreshCw class="h-3 w-3" />
      </Button>
    </div>

    <!-- Token 消耗 -->
    <div
      v-if="!isUser && tokenCount"
      class="mt-1 text-xs text-muted-foreground opacity-0 group-hover:opacity-100 transition-opacity"
    >
      {{ tokenCount }} tokens
    </div>
  </div>
</template>