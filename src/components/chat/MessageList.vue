<script setup lang="ts">
import type { Message } from '../../types'
import MessageItem from './MessageItem.vue'

const props = defineProps<{
  messages: Message[]
  streamingId: string | null
  streamingContent?: string
}>()

const emit = defineEmits<{
  'scroll-to-bottom': []
}>()
</script>

<template>
  <div style="height: 100%; overflow-y: auto;">
    <n-empty
      v-if="props.messages.length === 0 && !props.streamingId"
      description="暂无消息"
      style="margin-top: 80px;"
    />
    <div v-for="msg in props.messages" :key="msg.id" style="padding: 4px 0;">
      <MessageItem :message="msg" />
    </div>
    <div v-if="props.streamingId" style="padding: 4px 0;">
      <MessageItem
        :message="{
          id: props.streamingId,
          conversation_id: '',
          role: 'assistant',
          content: props.streamingContent ?? '',
          created_at: Date.now()
        }"
        :is-streaming="true"
      />
    </div>
  </div>
</template>
