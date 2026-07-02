<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useChatStore } from '@/stores/chatStore'
import { useSettingsStore } from '@/stores/settingsStore'
import { useChatEvents } from './useChatEvents'
import type { AttachmentMeta } from '@/lib/attachment'
import Sidebar from './Sidebar.vue'
import MessageList from './MessageList.vue'
import ChatInput from './ChatInput.vue'
import ChatHeader from './ChatHeader.vue'

const chatStore = useChatStore()
const settingsStore = useSettingsStore()
const { startListening, handleStopStream } = useChatEvents()

const sidebarCollapsed = ref(false)
const searchQuery = ref('')

function toggleSidebar() { sidebarCollapsed.value = !sidebarCollapsed.value }

// Conversation handlers
function handleSelect(id: string) { chatStore.switchConversation(id) }
function handleCreate() { chatStore.createConversation() }
function handleClose(id: string) { chatStore.deleteConversation(id) }
async function handleRename(id: string, title: string) { await chatStore.renameConversation(id, title) }
function handlePin(id: string) { chatStore.pinConversation(id) }
function handleUnpin(id: string) { chatStore.unpinConversation(id) }
function handleSend(displayContent: string, fullContent: string, attachments?: AttachmentMeta[]) { chatStore.sendMessage(displayContent, fullContent, attachments) }

onMounted(async () => {
  await settingsStore.loadSettings()
  await chatStore.loadConversations()
  await startListening()
})
</script>

<template>
  <div class="flex h-full flex-col">
    <ChatHeader :sidebar-collapsed="sidebarCollapsed" @toggle-sidebar="toggleSidebar" />
    <div class="flex flex-1 overflow-hidden p-1">
      <div class="flex flex-1 overflow-hidden rounded-lg border">
        <aside
          v-show="!sidebarCollapsed"
          class="w-[220px] shrink-0 border-r bg-background overflow-hidden"
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
