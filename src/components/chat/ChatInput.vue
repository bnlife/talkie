<script setup lang="ts">
import { ref } from 'vue'
import { Textarea } from '@/components/ui/textarea'
import { Button } from '@/components/ui/button'
import { SendIcon, SquareIcon } from 'lucide-vue-next'

const props = defineProps<{
  disabled: boolean
  streaming: boolean
}>()

const emit = defineEmits<{
  send: [content: string]
  'stop-stream': []
}>()

const inputText = ref('')

function handleSend() {
  if (inputText.value.trim() && !props.disabled && !props.streaming) {
    emit('send', inputText.value.trim())
    inputText.value = ''
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault()
    handleSend()
  }
}
</script>

<template>
  <div class="flex gap-normal items-end">
    <Textarea
      v-model="inputText"
      :disabled="disabled"
      placeholder="输入消息... (Enter 发送)"
      class="flex-1 min-h-14"
      :rows="2"
      @keydown="handleKeydown"
    />
    <Button v-if="streaming" variant="destructive" size="icon" @click="emit('stop-stream')">
      <SquareIcon class="size-4" />
    </Button>
    <Button v-else :disabled="disabled || !inputText.trim()" @click="handleSend">
      <SendIcon />
      发送
    </Button>
  </div>
</template>
