<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { Minus, Maximize2, Minimize2, X, Settings, PanelLeftOpen, PanelLeftClose, Moon, Sun } from 'lucide-vue-next'
import { useChatStore } from '@/stores/chatStore'
import { useSettingsStore } from '@/stores/settingsStore'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from '@/components/ui/dialog'
import Sidebar from '@/components/chat/Sidebar.vue'
import ChatPage from '@/pages/ChatPage.vue'
import SettingsPanel from '@/components/settings/SettingsPanel.vue'
import type { Settings as SettingsType } from '@/types'

const chatStore = useChatStore()
const settingsStore = useSettingsStore()

const sidebarCollapsed = ref(false)
const isMaximized = ref(false)
const settingsVisible = ref(false)

const currentTitle = computed(() => {
  if (!chatStore.activeConversationId) return 'Talkie'
  const conv = chatStore.conversations.find(c => c.id === chatStore.activeConversationId)
  return conv?.title || 'Talkie'
})

// 侧栏事件处理
function handleSelect(id: string) {
  chatStore.switchConversation(id)
}

function handleCreate() {
  chatStore.createConversation()
}

function handleCloseConversation(id: string) {
  chatStore.deleteConversation(id)
}

function handleRename(id: string, title: string) {
  chatStore.renameConversation(id, title)
}

function handlePin(id: string) {
  chatStore.pinConversation(id)
}

function handleUnpin(id: string) {
  chatStore.unpinConversation(id)
}

// 设置事件处理
function handleSettingsUpdate(value: Partial<SettingsType>) {
  settingsStore.updateSettings(value)
}

async function handleTestConnection() {
  await settingsStore.testConnection()
}

// 窗口控制
async function toggleSidebar() {
  sidebarCollapsed.value = !sidebarCollapsed.value
}

async function handleMinimize() {
  await getCurrentWindow().minimize()
}

async function handleToggleMaximize() {
  await getCurrentWindow().toggleMaximize()
  isMaximized.value = await getCurrentWindow().isMaximized()
}

async function handleClose() {
  await getCurrentWindow().close()
}

onMounted(async () => {
  await settingsStore.loadSettings()
  await chatStore.loadConversations()
  isMaximized.value = await getCurrentWindow().isMaximized()
})

// 夜间模式
function toggleDarkMode() {
  settingsStore.updateSettings({ darkMode: !settingsStore.darkMode })
}

watch(
  () => settingsStore.darkMode,
  (darkMode) => {
    const el = document.documentElement
    if (darkMode) el.classList.add('dark')
    else el.classList.remove('dark')
  },
  { immediate: true },
)
</script>

<template>
  <div class="flex h-screen w-screen overflow-hidden bg-background text-foreground">
    <!-- 侧边栏 -->
    <aside
      v-show="!sidebarCollapsed"
      :class="cn(
        'flex flex-col border-r border-border bg-muted/30 transition-[width] duration-200',
        sidebarCollapsed ? 'w-0' : 'w-60'
      )"
    >
      <Sidebar
        :conversations="chatStore.conversations"
        :active-id="chatStore.activeConversationId"
        @select="handleSelect"
        @create="handleCreate"
        @close="handleCloseConversation"
        @rename="handleRename"
        @pin="handlePin"
        @unpin="handleUnpin"
        @toggle-collapse="toggleSidebar"
        @open-settings="settingsVisible = true"
      />
    </aside>

    <!-- 主内容区 -->
    <div class="flex flex-1 flex-col overflow-hidden">
      <!-- 顶栏 -->
      <header
        data-tauri-drag-region
        :class="cn(
          'flex h-9 shrink-0 items-center justify-between border-b border-border bg-background px-3',
          'select-none'
        )"
      >
        <div class="flex items-center gap-2">
          <!-- 侧栏切换按钮 -->
          <Button
            variant="ghost"
            size="icon"
            class="h-6 w-6"
            @click.stop="toggleSidebar"
          >
            <PanelLeftClose v-if="!sidebarCollapsed" class="h-3.5 w-3.5" />
            <PanelLeftOpen v-else class="h-3.5 w-3.5" />
            <span class="sr-only">切换侧栏</span>
          </Button>
          <!-- 当前对话标题 -->
          <span class="truncate text-sm font-medium text-muted-foreground">
            {{ currentTitle }}
          </span>
        </div>

        <div class="flex items-center gap-0.5">
          <!-- 夜间模式 -->
          <Button
            variant="ghost"
            size="icon"
            class="h-6 w-6"
            :title="settingsStore.darkMode ? '日间模式' : '夜间模式'"
            @click.stop="toggleDarkMode"
          >
            <Moon v-if="!settingsStore.darkMode" class="h-3.5 w-3.5" />
            <Sun v-else class="h-3.5 w-3.5" />
            <span class="sr-only">夜间模式</span>
          </Button>

          <!-- 设置按钮 -->
          <Button
            variant="ghost"
            size="icon"
            class="h-6 w-6"
            @click.stop="settingsVisible = true"
          >
            <Settings class="h-3.5 w-3.5" />
            <span class="sr-only">设置</span>
          </Button>

          <!-- 最小化 -->
          <Button
            variant="ghost"
            size="icon"
            class="h-6 w-6"
            @click.stop="handleMinimize"
          >
            <Minus class="h-3.5 w-3.5" />
            <span class="sr-only">最小化</span>
          </Button>

          <!-- 最大化/还原 -->
          <Button
            variant="ghost"
            size="icon"
            class="h-6 w-6"
            @click.stop="handleToggleMaximize"
          >
            <Maximize2 v-if="!isMaximized" class="h-3.5 w-3.5" />
            <Minimize2 v-else class="h-3.5 w-3.5" />
            <span class="sr-only">{{ isMaximized ? '还原' : '最大化' }}</span>
          </Button>

          <!-- 关闭 -->
          <Button
            variant="ghost"
            size="icon"
            class="h-6 w-6 hover:bg-destructive hover:text-destructive-foreground"
            @click.stop="handleClose"
          >
            <X class="h-3.5 w-3.5" />
            <span class="sr-only">关闭</span>
          </Button>
        </div>
      </header>

      <!-- 聊天主区域 -->
      <main class="flex-1 overflow-hidden">
        <ChatPage />
      </main>
    </div>

    <!-- 设置弹窗 -->
    <Dialog v-model:open="settingsVisible">
      <DialogContent class="max-w-lg">
        <DialogHeader>
          <DialogTitle>设置</DialogTitle>
          <DialogDescription>管理应用偏好与接口配置</DialogDescription>
        </DialogHeader>
        <SettingsPanel
          :settings="settingsStore.$state"
          @update="handleSettingsUpdate"
          @test-connection="handleTestConnection"
        />
      </DialogContent>
    </Dialog>
  </div>
</template>
