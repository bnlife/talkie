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
  <n-space vertical :size="8">
    <n-input
      v-model:value="inputText"
      type="textarea"
      :autosize="{ minRows: 2, maxRows: 6 }"
      :disabled="disabled"
      placeholder="输入消息... (Enter 发送, Shift+Enter 换行)"
      @keydown="handleKeydown"
    />
    <n-space justify="end" :size="8">
      <n-button
        v-if="streaming"
        type="error"
        @click="emit('stop-stream')"
      >
        停止生成
      </n-button>
      <n-button
        v-else
        type="primary"
        :disabled="disabled || !inputText.trim()"
        @click="handleSend"
      >
        发送
      </n-button>
    </n-space>
  </n-space>
</template>
