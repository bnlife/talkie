<script setup lang="ts">
import { computed, watch, nextTick, ref } from 'vue'
import type { Message } from '@/types'
import { useChatStore } from '@/stores/chatStore'
import { cn } from '@/lib/utils'
import { MessageCircle } from 'lucide-vue-next'
import { Avatar, AvatarFallback } from '@/components/ui/avatar'
import MessageItem from './MessageItem.vue'

const chatStore = useChatStore()
const scrollRef = ref<HTMLDivElement | null>(null)
const userScrolled = ref(false)

const messages = computed(() => chatStore.messages)
const modelName = computed(() => chatStore.activeConversation?.model || '')

const showWaiting = computed(() => chatStore.waitingForResponse && !chatStore.streamingId)

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

const isLastMessage = (msg: Message) => {
  const last = allMessages.value[allMessages.value.length - 1]
  return last && last.id === msg.id
}

function isNearBottom(): boolean {
  const el = scrollRef.value
  if (!el) return true
  return el.scrollHeight - el.scrollTop - el.clientHeight < 100
}

function scrollToBottom() {
  nextTick(() => {
    const el = scrollRef.value
    if (el) {
      el.scrollTop = el.scrollHeight
    }
  })
}

function onScroll() {
  userScrolled.value = !isNearBottom()
}

watch(
  () => allMessages.value.length,
  () => {
    if (!userScrolled.value) {
      scrollToBottom()
    }
  }
)

watch(
  () => chatStore.streamingContent,
  () => {
    if (!userScrolled.value) {
      scrollToBottom()
    }
  }
)

watch(
  () => chatStore.waitingForResponse,
  (waiting) => {
    if (waiting) scrollToBottom()
  }
)

async function handleCopy(content: string) {
  await navigator.clipboard.writeText(content)
}

async function handleDelete(messageId: string) {
  await chatStore.deleteMessage(messageId)
}

async function handleRegenerate() {
  await chatStore.regenerateMessage()
}
</script>

<template>
  <div
    ref="scrollRef"
    :class="cn('flex-1 w-full overflow-y-auto')"
    @scroll="onScroll"
  >
    <div class="flex flex-col gap-3 px-4 py-2">
      <template v-if="allMessages.length">
        <MessageItem
          v-for="msg in allMessages"
          :key="msg.id"
          :message="msg"
          :streaming="msg.id === chatStore.streamingId"
          :is-last="isLastMessage(msg)"
          :model-name="modelName"
          @copy="handleCopy"
          @delete="handleDelete"
          @regenerate="handleRegenerate"
        />
        <!-- Waiting for API response -->
        <div v-if="showWaiting" class="flex w-full gap-3 pb-4">
          <Avatar class="h-8 w-8 shrink-0 mt-1 bg-muted text-muted-foreground">
            <AvatarFallback class="text-xs">AI</AvatarFallback>
          </Avatar>
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2 mb-1">
              <span class="text-sm font-medium text-foreground">{{ modelName || 'AI' }}</span>
            </div>
            <div class="flex items-center gap-1 py-1">
              <span class="size-1.5 rounded-full bg-muted-foreground/40 animate-bounce [animation-delay:0ms]" />
              <span class="size-1.5 rounded-full bg-muted-foreground/40 animate-bounce [animation-delay:150ms]" />
              <span class="size-1.5 rounded-full bg-muted-foreground/40 animate-bounce [animation-delay:300ms]" />
            </div>
          </div>
        </div>
      </template>
      <div
        v-else
        class="flex flex-col items-center justify-center py-16 text-muted-foreground"
      >
        <MessageCircle class="w-10 h-10 mb-3" />
        <p class="text-sm">暂无消息</p>
      </div>
    </div>
  </div>
</template>
