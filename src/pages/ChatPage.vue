<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { useChatStore } from '@/stores/chatStore'
import * as chatBridge from '@/bridge/chat'
import { cn } from '@/lib/utils'
import MessageList from '@/components/chat/MessageList.vue'
import ChatInput from '@/components/chat/ChatInput.vue'

const chatStore = useChatStore()

const isStreaming = computed(() => chatStore.streamingId !== null)
const isDisabled = computed(() => !chatStore.activeConversationId)

let unlisteners: (() => void)[] = []

onMounted(async () => {
  unlisteners.push(
    await listen<{ message_id: string; delta: string }>('chat:stream-chunk', (event) => {
      chatStore.appendStreamChunk(event.payload.message_id, event.payload.delta)
    })
  )
  unlisteners.push(
    await listen<void>('chat:stream-done', () => {
      chatStore.finishStream()
    })
  )
})

onUnmounted(() => {
  unlisteners.forEach((unlisten) => unlisten())
  unlisteners = []
})

async function handleSend(content: string) {
  await chatStore.sendMessage(content)
}

async function handleStopStream() {
  await chatBridge.stopStream()
}
</script>

<template>
  <div :class="cn('flex flex-col h-full bg-background')">
    <MessageList />
    <ChatInput
      :disabled="isDisabled"
      :streaming="isStreaming"
      @send="handleSend"
      @stop-stream="handleStopStream"
    />
  </div>
</template>
