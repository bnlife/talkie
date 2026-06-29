<script setup lang="ts">
import { ref } from 'vue'

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
  <div style="display: flex; gap: 8px; align-items: flex-end;">
    <n-input
      v-model:value="inputText"
      type="textarea"
      :autosize="{ minRows: 2, maxRows: 6 }"
      :disabled="disabled"
      placeholder="输入消息... (Enter 发送)"
      @keydown="handleKeydown"
      style="flex: 1;"
    />
    <n-button
      v-if="streaming"
      type="error"
      @click="emit('stop-stream')"
      style="flex-shrink: 0;"
    >
      停止
    </n-button>
    <n-button
      v-else
      type="primary"
      :disabled="disabled || !inputText.trim()"
      @click="handleSend"
      style="flex-shrink: 0;"
    >
      发送
    </n-button>
  </div>
</template>
