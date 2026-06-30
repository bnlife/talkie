<script setup lang="ts">
import type { Message } from '../../types'
import MessageItem from './MessageItem.vue'
import { InboxIcon } from 'lucide-vue-next'

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
  <div class="h-full overflow-y-auto">
    <div
      v-if="props.messages.length === 0 && !props.streamingId"
      class="flex flex-col items-center justify-center mt-12 text-hint gap-normal"
    >
      <InboxIcon class="size-8 text-hint" />
      <span class="text-small">暂无消息</span>
    </div>
    <div v-for="msg in props.messages" :key="msg.id" class="py-1">
      <MessageItem :message="msg" />
    </div>
    <div v-if="props.streamingId" class="py-1">
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


