<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
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

function handleSend(content: string) {
  chatStore.sendMessage(content)
}

function handleStopStream() {
  chatBridge.stopStream()
}
</script>

<template>
  <div style="height: 100%; display: flex; flex-direction: column; padding: 0 12px 12px; box-sizing: border-box;">
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
