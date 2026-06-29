<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useChatStore } from './stores/chatStore'
import { useSettingsStore } from './stores/settingsStore'
import Sidebar from './components/chat/Sidebar.vue'
import ChatPage from './pages/ChatPage.vue'
import SettingsPanel from './components/settings/SettingsPanel.vue'
import type { Settings } from './types'
import type { GlobalThemeOverrides } from 'naive-ui'
import { createDiscreteApi } from 'naive-ui'
import { MenuOutline } from '@vicons/ionicons5'
import { getCurrentWindow } from '@tauri-apps/api/window'

const chatStore = useChatStore()
const { message } = createDiscreteApi(['message'])
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

const themeOverrides: GlobalThemeOverrides = {
  common: {
    fontSizeSmall: '11px',
    fontSizeMedium: '12px',
    fontSize: '12px',
    fontSizeLarge: '14px',
    primaryColor: '#0d0d0d',
    primaryColorHover: '#5d5d5d',
    primaryColorPressed: '#0d0d0d',
    primaryColorSuppl: '#0d0d0d',
    baseColor: '#ffffff',
    bodyColor: '#f9f9f9',
    borderColor: '#ececec',
    textColorBase: '#0d0d0d',
    textColor1: '#0d0d0d',
    textColor2: '#5d5d5d',
    textColor3: '#8f8f8f',
    dividerColor: '#ececec',
    hoverColor: '#ececec',
    pressedColor: '#ececec',
    inputColor: '#ffffff',
    inputColorDisabled: '#f9f9f9',
    cardColor: '#ffffff',
    modalColor: '#ffffff',
    popoverColor: '#ffffff',
    tableColor: '#ffffff',
    actionColor: '#f9f9f9',
    clearColor: '#8f8f8f',
    iconColor: '#5d5d5d',
    iconColorHover: '#0d0d0d',
    iconColorPressed: '#0d0d0d',
  },
  Button: {
    // secondary prop (used by "新建对话" button)
    colorSecondary: '#ffffff',
    colorSecondaryHover: '#ececec',
    colorSecondaryPressed: '#d9d9e3',
    textColor: '#0d0d0d',
    heightSmall: '28px',
    paddingSmall: '0 12px',
  },
  Card: {
    borderColor: '#ececec',
    color: '#ffffff',
    titleFontSizeSmall: '12px',
    titleFontSizeMedium: '12px',
    titleFontSizeLarge: '12px',
    titleFontSizeHuge: '12px',
  },
  Menu: {
    itemColorHover: '#ececec',
    itemColorActive: '#ececec',
    itemTextColor: '#5d5d5d',
    itemTextColorHover: '#0d0d0d',
    itemTextColorActive: '#0d0d0d',
    arrowColor: '#5d5d5d',
  },
  Modal: {
    color: '#ffffff',
    borderColor: '#ececec',
    // Modal preset="card" 时标题字号由 Card.titleFontSizeMedium 控制
  },
  Form: {
    feedbackFontSizeSmall: '12px',
    feedbackFontSizeMedium: '12px',
    feedbackFontSizeLarge: '12px',
    labelFontSizeLeftSmall: '12px',
    labelFontSizeLeftMedium: '12px',
    labelFontSizeLeftLarge: '12px',
    labelFontSizeTopSmall: '12px',
    labelFontSizeTopMedium: '12px',
    labelFontSizeTopLarge: '12px',
  },
  Divider: {
    color: '#ececec',
  },
  Input: {
    color: '#ffffff',
    border: '1px solid #ececec',
    borderHover: '1px solid #c8c8c8',
    borderFocus: '1px solid #0d0d0d',
    placeholderColor: '#8f8f8f',
    textColor: '#0d0d0d',
    heightSmall: '28px',
    paddingSmall: '0 10px',
  },
  Slider: {
    fillColor: '#6b7280',
    fillColorHover: '#9ca3af',
    railColor: '#e5e7eb',
    handleColor: '#ffffff',
    handleBorderColor: '#d1d5db',
  },
  Select: {
    menuColor: '#ffffff',
    border: '1px solid #ececec',
    borderHover: '1px solid #c8c8c8',
    borderFocus: '1px solid #0d0d0d',
    heightSmall: '28px',
  },
  Tag: {
    color: '#ececec',
    textColor: '#0d0d0d',
    border: '1px solid #ececec',
    fontSizeMedium: '12px',
    paddingTiny: '0 6px',
    heightTiny: '20px',
  },
  Tabs: {
    tabTextColor: '#5d5d5d',
    tabTextColorActive: '#0d0d0d',
    tabTextColorHover: '#0d0d0d',
    barColor: '#0d0d0d',
  },
  Empty: {
    textColor: '#8f8f8f',
    iconColor: '#c8c8c8',
  },
  Message: {
    color: '#ffffff',
    textColor: '#0d0d0d',
    border: '1px solid #ececec',
  },
  Typography: {
    headerFontSize1: '12px',
    headerFontSize2: '12px',
    headerFontSize3: '12px',
    headerFontSize4: '12px',
    headerFontSize5: '12px',
    headerFontSize6: '12px',
  },
}

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
  message.success('设置已保存')
  showSettings.value = false
}

async function handleTestConnection() {
  const result = await settingsStore.testConnection()
  if (result.ok) {
    message.success('连接成功')
  } else {
    message.error(result.error || '连接失败')
  }
}
</script>

<template>
  <n-config-provider :theme-overrides="themeOverrides">
    <n-layout has-sider style="height: 100vh;">
      <n-layout-sider
        :bordered="!sidebarCollapsed"
        collapse-mode="width"
        :collapsed-width="0"
        :width="240"
        :collapsed="sidebarCollapsed"
        style="height: 100%; overflow: hidden; background: #f9f9f9;"
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
      </n-layout-sider>
      <n-layout-content content-style="overflow: hidden; background: #ffffff;">
        <div style="height: 100%; display: flex; flex-direction: column;">
          <!-- 右侧顶栏 32px -->
          <div style="height: 32px; flex-shrink: 0; display: flex; align-items: center;" data-tauri-drag-region>
            <n-button
              v-if="sidebarCollapsed"
              text
              @click.stop="toggleSider"
              style="margin-left: 4px; width: 28px;"
            >
              <template #icon><n-icon :component="MenuOutline" /></template>
            </n-button>
            <n-tag
              v-if="activeConversationTitle"
              :bordered="false"
              style="flex-shrink: 0; margin-left: 12px;"
            >
              {{ activeConversationTitle }}
            </n-tag>
            <div style="flex: 1; min-width: 0;"></div>
            <div style="flex-shrink: 0; display: flex; gap: 2px; padding-right: 8px;" @click.stop @mousedown.stop>
              <button class="titlebar-btn" @click="minimizeWindow">─</button>
              <button class="titlebar-btn" @click="toggleMaximize">{{ isMaximized ? '❐' : '☐' }}</button>
              <button class="titlebar-btn titlebar-btn-close" @click="closeWindow">✕</button>
            </div>
          </div>
          <ChatPage />
        </div>
      </n-layout-content>
    </n-layout>

    <n-modal v-model:show="showSettings" title="设置" preset="card" style="width: 480px;">
      <SettingsPanel
        :settings="settingsStore.$state"
        @update="handleUpdateSettings"
        @test-connection="handleTestConnection"
      />
    </n-modal>
  </n-config-provider>
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
  color: #4b5563;
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
