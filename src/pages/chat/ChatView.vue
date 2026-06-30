<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useChatStore } from '@/stores/chatStore'
import { useSettingsStore } from '@/stores/settingsStore'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { listen } from '@tauri-apps/api/event'
import * as chatBridge from '@/bridge/chat'
import { log } from '@/bridge/log'
import { Button } from '@/components/ui/button'

import { PanelLeftOpen, PanelLeftClose, Minus, Maximize2, Minimize2, X } from 'lucide-vue-next'
import Sidebar from './Sidebar.vue'
import MessageList from './MessageList.vue'
import ChatInput from './ChatInput.vue'

const chatStore = useChatStore()
const settingsStore = useSettingsStore()
const appWindow = getCurrentWindow()
const sidebarCollapsed = ref(false)
const searchQuery = ref('')
const isMaximized = ref(false)

function toggleSidebar() { sidebarCollapsed.value = !sidebarCollapsed.value }

async function minimizeWindow() { await appWindow.minimize() }
async function toggleMaximize() {
  if (isMaximized.value) { await appWindow.unmaximize() } else { await appWindow.maximize() }
  isMaximized.value = await appWindow.isMaximized()
}
async function closeWindow() { await appWindow.close() }

// Conversation handlers
function handleSelect(id: string) { chatStore.switchConversation(id) }
function handleCreate() { chatStore.createConversation() }
function handleClose(id: string) { chatStore.deleteConversation(id) }
async function handleRename(id: string, title: string) { await chatStore.renameConversation(id, title) }
function handlePin(id: string) { chatStore.pinConversation(id) }
function handleUnpin(id: string) { chatStore.unpinConversation(id) }
function handleSend(content: string) { chatStore.sendMessage(content) }
async function handleStopStream() {
  await log('info', '前端::ChatView | 用户停止流式输出')
  await chatBridge.stopStream()
  await chatStore.finishStream()
}

// Tauri event listeners
let cleanupFns: (() => void)[] = []
onMounted(async () => {
  await chatStore.loadConversations()
  await settingsStore.loadSettings()
  isMaximized.value = await appWindow.isMaximized()
  cleanupFns = [
    await listen('chat:stream-chunk', (event) => {
      const p = event.payload as { message_id: string; delta: string }
      chatStore.appendStreamChunk(p.message_id, p.delta)
    }),
    await listen('chat:stream-done', () => chatStore.finishStream()),
  ]
})
onUnmounted(() => { cleanupFns.forEach(fn => fn()) })
</script>

<template>
  <div class="flex h-full flex-col">
    <header
      data-tauri-drag-region
      class="flex h-9 shrink-0 items-center justify-between bg-muted px-3 select-none"
    >
      <div class="flex items-center gap-2">
        <Button variant="ghost" size="icon" class="h-6 w-6" @click.stop="toggleSidebar">
          <PanelLeftClose v-if="!sidebarCollapsed" class="h-3.5 w-3.5" />
          <PanelLeftOpen v-else class="h-3.5 w-3.5" />
        </Button>
        <span class="text-sm font-medium text-muted-foreground truncate">{{ chatStore.activeConversation?.title || '对话' }}</span>
      </div>
      <div class="flex items-center gap-0.5">
        <Button variant="ghost" size="icon" class="h-6 w-6 hover:bg-background" @click="minimizeWindow"><Minus class="h-3.5 w-3.5" /></Button>
        <Button variant="ghost" size="icon" class="h-6 w-6 hover:bg-background" @click="toggleMaximize">
          <Maximize2 v-if="!isMaximized" class="h-3.5 w-3.5" />
          <Minimize2 v-else class="h-3.5 w-3.5" />
        </Button>
        <Button variant="ghost" size="icon" class="h-6 w-6 hover:bg-destructive hover:text-destructive-foreground" @click="closeWindow"><X class="h-3.5 w-3.5" /></Button>
      </div>
    </header>
    <div class="flex flex-1 overflow-hidden p-1">
      <div class="flex flex-1 overflow-hidden rounded-lg border">
        <aside
          v-show="!sidebarCollapsed"
          class="w-60 shrink-0 border-r bg-background overflow-hidden"
        >
          <Sidebar
            :conversations="chatStore.conversations"
            :active-id="chatStore.activeConversationId"
            v-model:search-query="searchQuery"
            @select="handleSelect"
            @create="handleCreate"
            @close="handleClose"
            @rename="handleRename"
            @pin="handlePin"
            @unpin="handleUnpin"
          />
        </aside>
        <main class="relative flex flex-1 flex-col overflow-hidden bg-background">
          <MessageList
            :messages="chatStore.messages"
            :streaming-id="chatStore.streamingId"
            :streaming-content="chatStore.streamingContent"
          />
          <ChatInput
            :disabled="!chatStore.activeConversationId"
            :streaming="!!chatStore.streamingId"
            @send="handleSend"
            @stop-stream="handleStopStream"
          />
        </main>
      </div>
    </div>
  </div>
</template>
