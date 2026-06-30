<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useChatStore } from './stores/chatStore'
import { useSettingsStore } from './stores/settingsStore'
import Sidebar from './components/chat/Sidebar.vue'
import ChatPage from './pages/ChatPage.vue'
import SettingsPanel from './components/settings/SettingsPanel.vue'
import type { Settings } from './types'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Toaster, toast } from 'vue-sonner'
import { PanelRightOpenIcon } from 'lucide-vue-next'

const chatStore = useChatStore()
const settingsStore = useSettingsStore()
const appWindow = getCurrentWindow()
const isMaximized = ref(false)
const sidebarCollapsed = ref(false)

const activeConversationTitle = computed(() => {
  if (!chatStore.activeConversationId) return ''
  const conv = chatStore.conversations.find(c => c.id === chatStore.activeConversationId)
  return conv?.title ?? ''
})

async function minimizeWindow() { await appWindow.minimize() }
async function toggleMaximize() {
  if (isMaximized.value) { await appWindow.unmaximize() } else { await appWindow.maximize() }
  isMaximized.value = await appWindow.isMaximized()
}
async function closeWindow() { await appWindow.close() }
function toggleSider() { sidebarCollapsed.value = !sidebarCollapsed.value }

const showSettings = ref(false)

onMounted(async () => {
  await settingsStore.loadSettings()
  await chatStore.loadConversations()
  isMaximized.value = await appWindow.isMaximized()
})

function handleSelectConversation(id: string) {
  chatStore.switchConversation(id)
}

function handleCreateConversation() {
  chatStore.createConversation()
}

function handleDeleteConversation(id: string) {
  chatStore.deleteConversation(id)
}

async function handleRenameConversation(id: string, title: string) {
  await chatStore.renameConversation(id, title)
}

function handlePinConversation(id: string) {
  chatStore.pinConversation(id)
}

function handleUnpinConversation(id: string) {
  chatStore.unpinConversation(id)
}

async function handleUpdateSettings(partial: Partial<Settings>) {
  await settingsStore.updateSettings(partial)
  toast.success('设置已保存')
  showSettings.value = false
}

async function handleTestConnection() {
  const result = await settingsStore.testConnection()
  if (result.ok) {
    toast.success('连接成功')
  } else {
    toast.error(result.error || '连接失败')
  }
}
</script>

<template>
  <div class="h-screen flex flex-col overflow-hidden">
    <!-- 右侧顶栏 32px -->
    <div class="flex flex-shrink-0 h-8 bg-page border-b border-border">
      <!-- 对应侧栏宽度的留空 -->
      <div class="w-60 flex-shrink-0"></div>
      <!-- 右侧：标签 + 拖拽区 + 控件 -->
      <div class="flex-1 flex items-center min-w-0" data-tauri-drag-region>
        <Button
          v-if="sidebarCollapsed"
          variant="ghost"
          size="sm"
          @click.stop="toggleSider"
          class="ml-tight"
        >
          <PanelRightOpenIcon class="size-4" />
        </Button>
        <Badge
          v-if="activeConversationTitle"
          variant="secondary"
          class="ml-3 flex-shrink-0 text-small"
        >
          {{ activeConversationTitle }}
        </Badge>
        <div class="flex-1 min-w-0"></div>
        <div class="flex-shrink-0 flex gap-0.5 pr-2" @click.stop @mousedown.stop>
          <button class="titlebar-btn" @click="minimizeWindow">─</button>
          <button class="titlebar-btn" @click="toggleMaximize">{{ isMaximized ? '❐' : '☐' }}</button>
          <button class="titlebar-btn titlebar-btn-close" @click="closeWindow">✕</button>
        </div>
      </div>
    </div>

    <!-- 主布局：侧栏 + 内容 -->
    <div class="flex flex-1 min-h-0">
      <!-- 侧栏 -->
      <aside
        :class="[
          'h-full overflow-hidden bg-page border-r border-border transition-all duration-200',
          sidebarCollapsed ? 'w-0' : 'w-60'
        ]"
      >
        <Sidebar
          :conversations="chatStore.conversations"
          :active-id="chatStore.activeConversationId"
          @select="handleSelectConversation"
          @create="handleCreateConversation"
          @close="handleDeleteConversation"
          @rename="handleRenameConversation"
          @pin="handlePinConversation"
          @unpin="handleUnpinConversation"
          @toggle-collapse="toggleSider"
          @open-settings="showSettings = true"
        />
      </aside>

      <!-- 主内容区 -->
      <main class="flex-1 flex flex-col overflow-hidden bg-surface">
        <ChatPage />
      </main>
    </div>
  </div>

  <!-- 设置弹窗 -->
  <Dialog v-model:open="showSettings">
    <DialogContent class="sm:max-w-[480px]">
      <DialogHeader>
        <DialogTitle>设置</DialogTitle>
      </DialogHeader>
      <SettingsPanel
        :settings="settingsStore.$state"
        @update="handleUpdateSettings"
        @test-connection="handleTestConnection"
      />
    </DialogContent>
  </Dialog>

  <!-- Sonner Toast -->
  <Toaster richColors position="top-right" />
</template>

<style scoped>
.titlebar-btn {
  width: 36px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  color: var(--color-sub);
  cursor: pointer;
  border-radius: 4px;
  transition: background-color 0.15s ease;
}
.titlebar-btn:hover { background-color: #e5e7eb; }
.titlebar-btn:active { background-color: #d1d5db; }
.titlebar-btn-close:hover { background-color: #ef4444; color: #fff; }
.titlebar-btn-close:active { background-color: #dc2626; }
:global(body) { font-size: 12px; }
</style>
