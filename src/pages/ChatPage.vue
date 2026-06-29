<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { useChatStore } from '../stores/chatStore'
import * as chatBridge from '../bridge/chat'
import MessageList from '../components/chat/MessageList.vue'
import ChatInput from '../components/chat/ChatInput.vue'

const chatStore = useChatStore()

const cleanupFns: (() => void)[] = []

onMounted(() => {
  ;(async () => {
    const unlistenChunk = await listen('chat:stream-chunk', (event) => {
      const payload = event.payload as { message_id: string; delta: string }
      chatStore.appendStreamChunk(payload.message_id, payload.delta)
    })
    const unlistenDone = await listen('chat:stream-done', () => {
      chatStore.finishStream()
    })
    cleanupFns.push(unlistenChunk, unlistenDone)
  })()
})

onUnmounted(() => {
  cleanupFns.forEach(fn => fn())
})

const currentConversation = computed(() =>
  chatStore.conversations.find(c => c.id === chatStore.activeConversationId)
)

function handleSend(content: string) {
  chatStore.sendMessage(content)
}

function handleStopStream() {
  chatBridge.stopStream()
}
</script>

<template>
  <div style="height: 100%; display: flex; flex-direction: column; padding: 16px; box-sizing: border-box;">
    <n-h2 style="margin: 0 0 8px 0;">
      {{ currentConversation?.title ?? '选择或创建一个对话' }}
    </n-h2>
    <n-divider style="margin: 0 0 12px 0;" />
    <div style="flex: 1; overflow: hidden; min-height: 0;">
      <MessageList
        :messages="chatStore.messages"
        :streaming-id="chatStore.streamingId"
        :streaming-content="chatStore.streamingContent"
      />
    </div>
    <div style="margin-top: 12px; flex-shrink: 0;">
      <ChatInput
        :disabled="!chatStore.activeConversationId"
        :streaming="!!chatStore.streamingId"
        @send="handleSend"
        @stop-stream="handleStopStream"
      />
    </div>
  </div>
</template>
