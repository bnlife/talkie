<script setup lang="ts">
import { ref } from 'vue'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import { Send, Square } from 'lucide-vue-next'

const props = defineProps<{
  disabled?: boolean
  streaming?: boolean
}>()

const emit = defineEmits<{
  (e: 'send', content: string): void
  (e: 'stop-stream'): void
}>()

const input = ref('')

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault()
    handleSend()
  }
}

function handleSend() {
  const text = input.value.trim()
  if (!text || props.disabled) return
  emit('send', text)
  input.value = ''
}
</script>

<template>
  <div
    :class="cn(
      'flex items-end gap-2 px-3 pt-1 pb-2 bg-background',
    )"
  >
    <Textarea
      v-model="input"
      :disabled="disabled"
      :rows="1"
      placeholder="输入消息..."
      :class="cn(
        'min-h-[40px] max-h-[120px] resize-none flex-1 text-sm leading-relaxed',
      )"
      @keydown="handleKeydown"
    />
    <Button
      v-if="streaming"
      variant="destructive"
      size="icon"
      class="shrink-0 h-10 w-10"
      @click="emit('stop-stream')"
    >
      <Square class="w-4 h-4" />
    </Button>
    <Button
      v-else
      size="icon"
      class="shrink-0 h-10 w-10"
      :disabled="disabled || !input.trim()"
      @click="handleSend"
    >
      <Send class="w-4 h-4" />
    </Button>
  </div>
</template>
