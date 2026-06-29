<script setup lang="ts">
import type { Message } from '../../types'

const props = defineProps<{
  message: Message
  isStreaming?: boolean
}>()
</script>

<template>
  <n-space
    :justify="props.message.role === 'user' ? 'end' : 'start'"
    style="width: 100%;"
  >
    <n-card
      style="max-width: 70%;"
      :content-style="{ padding: '6px 12px' }"
      :bordered="props.message.role === 'assistant'"
      :embedded="props.message.role === 'user'"
      size="small"
    >
      <n-tag
        :type="props.message.role === 'user' ? 'primary' : 'info'"
        size="tiny"
        :bordered="false"
        style="margin-bottom: 4px;"
      >
        {{ props.message.role === 'user' ? '用户' : '助手' }}
      </n-tag>
      <n-text :depth="props.message.role === 'user' ? 1 : 2">
        {{ props.message.content }}<template v-if="props.isStreaming"><n-text depth="3">|</n-text></template>
      </n-text>
    </n-card>
  </n-space>
</template>
