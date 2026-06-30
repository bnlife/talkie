<script setup lang="ts">
import { computed, watch, nextTick, ref } from 'vue'
import type { Message } from '@/types'
import { useChatStore } from '@/stores/chatStore'
import { cn } from '@/lib/utils'
import { ScrollArea } from '@/components/ui/scroll-area'
import { MessageCircle } from 'lucide-vue-next'
import MessageItem from './MessageItem.vue'

const chatStore = useChatStore()
const scrollRef = ref<InstanceType<typeof ScrollArea> | null>(null)

const messages = computed(() => chatStore.messages)

const streamingMessage = computed<Message | null>(() => {
  if (!chatStore.streamingId) return null
  return {
    id: chatStore.streamingId,
    conversation_id: chatStore.activeConversationId || '',
    role: 'assistant',
    content: chatStore.streamingContent,
    created_at: Date.now(),
  }
})

const allMessages = computed(() => {
  const list = [...messages.value]
  if (streamingMessage.value) {
    list.push(streamingMessage.value)
  }
  return list
})

function scrollToBottom() {
  nextTick(() => {
    const el = scrollRef.value?.$el as HTMLElement | undefined
    if (el) {
      const viewport = el.querySelector<HTMLElement>('[data-radix-scroll-area-viewport]')
      if (viewport) {
        viewport.scrollTop = viewport.scrollHeight
      }
    }
  })
}

watch(
  () => allMessages.value.length,
  () => {
    scrollToBottom()
  }
)

watch(
  () => chatStore.streamingContent,
  () => {
    scrollToBottom()
  }
)
</script>

<template>
  <ScrollArea
    ref="scrollRef"
    :class="cn('mt-10 flex-1 w-full')"
  >
    <div class="flex flex-col gap-3 px-4 pb-4">
      <template v-if="allMessages.length">
        <MessageItem
          v-for="msg in allMessages"
          :key="msg.id"
          :message="msg"
          :streaming="msg.id === chatStore.streamingId"
        />
      </template>
      <div
        v-else
        class="flex flex-col items-center justify-center py-16 text-muted-foreground"
      >
        <MessageCircle class="w-10 h-10 mb-3" />
        <p class="text-sm">暂无消息</p>
      </div>
    </div>
  </ScrollArea>
</template>
