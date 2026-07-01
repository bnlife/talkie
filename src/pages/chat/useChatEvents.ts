import { onUnmounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { toast } from 'vue-sonner'
import { EVENTS } from '@/lib/events'
import { log } from '@/bridge/log'
import * as chatBridge from '@/bridge/chat'
import { useChatStore } from '@/stores/chatStore'
import type { SearchResult } from '@/types'

export function useChatEvents() {
  const chatStore = useChatStore()

  let cleanupFns: (() => void)[] = []

  async function startListening() {
    cleanupFns = [
      await listen(EVENTS.CHAT_STREAM_CHUNK, (event) => {
        const p = event.payload as { message_id: string; delta: string }
        chatStore.appendStreamChunk(p.message_id, p.delta)
      }),
      await listen(EVENTS.CHAT_STREAM_DONE, (event) => {
        const p = event.payload as { message_id: string; token_count?: number; search_results?: SearchResult[] }
        chatStore.finishStream(p.token_count, p.search_results)
      }),
      await listen(EVENTS.CHAT_ERROR, (event) => {
        const { message } = event.payload as { message: string }
        log('error', `FE::ChatView | error event | ${message}`)
        chatStore.waitingForResponse = false
        toast.error(message)
      }),
    ]
  }

  function stopListening() {
    cleanupFns.forEach(fn => fn())
    cleanupFns = []
  }

  async function handleStopStream() {
    await log('info', 'FE::ChatView | user stop stream')
    await chatBridge.stopStream()
    await chatStore.finishStream()
  }

  onUnmounted(stopListening)

  return { startListening, handleStopStream }
}
