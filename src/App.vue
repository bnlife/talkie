<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useChatStore } from './stores/chatStore'
import { useSettingsStore } from './stores/settingsStore'
import Sidebar from './components/chat/Sidebar.vue'
import ChatPage from './pages/ChatPage.vue'
import SettingsPanel from './components/settings/SettingsPanel.vue'
import type { Settings } from './types'
import type { GlobalThemeOverrides } from 'naive-ui'
import { createDiscreteApi } from 'naive-ui'

const chatStore = useChatStore()
const { message } = createDiscreteApi(['message'])
const settingsStore = useSettingsStore()

const themeOverrides: GlobalThemeOverrides = {
  common: {
    primaryColor: '#6b7280',
    primaryColorHover: '#9ca3af',
    primaryColorPressed: '#4b5563',
    primaryColorSuppl: '#6b7280',
    baseColor: '#ffffff',
    bodyColor: '#f8f9fa',
    borderColor: '#e5e7eb',
    textColorBase: '#1f2937',
    textColor1: '#1f2937',
    textColor2: '#4b5563',
    textColor3: '#9ca3af',
    dividerColor: '#e5e7eb',
    hoverColor: '#f3f4f6',
    pressedColor: '#e5e7eb',
    inputColor: '#ffffff',
    inputColorDisabled: '#f9fafb',
    cardColor: '#ffffff',
    modalColor: '#ffffff',
    popoverColor: '#ffffff',
    tableColor: '#ffffff',
    actionColor: '#f9fafb',
    clearColor: '#9ca3af',
    iconColor: '#6b7280',
    iconColorHover: '#4b5563',
    iconColorPressed: '#374151',
  },
  Button: {
    colorTertiary: '#f3f4f6',
    colorHoverTertiary: '#e5e7eb',
    colorPressedTertiary: '#d1d5db',
    textColorTertiary: '#374151',
    borderTertiary: '1px solid #e5e7eb',
  },
  Card: {
    borderColor: '#e5e7eb',
    color: '#ffffff',
  },
  Menu: {
    itemColorHover: '#f3f4f6',
    itemColorActive: '#e5e7eb',
    itemTextColor: '#374151',
    itemTextColorHover: '#111827',
    itemTextColorActive: '#111827',
    arrowColor: '#6b7280',
  },
  Modal: {
    color: '#ffffff',
    borderColor: '#e5e7eb',
  },
  Divider: {
    color: '#e5e7eb',
  },
  Input: {
    color: '#ffffff',
    border: '1px solid #d1d5db',
    borderHover: '1px solid #9ca3af',
    borderFocus: '1px solid #6b7280',
    placeholderColor: '#9ca3af',
    textColor: '#1f2937',
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
    border: '1px solid #d1d5db',
    borderHover: '1px solid #9ca3af',
    borderFocus: '1px solid #6b7280',
  },
  Tag: {
    color: '#f3f4f6',
    textColor: '#374151',
    border: '1px solid #e5e7eb',
  },
  Tabs: {
    tabTextColor: '#6b7280',
    tabTextColorActive: '#111827',
    tabTextColorHover: '#374151',
    barColor: '#6b7280',
  },
  Empty: {
    textColor: '#9ca3af',
    iconColor: '#d1d5db',
  },
  Message: {
    color: '#ffffff',
    textColor: '#1f2937',
    border: '1px solid #e5e7eb',
  },
}

const showSettings = ref(false)

onMounted(() => {
  chatStore.loadConversations()
  settingsStore.loadSettings()
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

function handleRenameConversation(id: string, title: string) {
  // TODO: rename logic
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
    <n-layout position="absolute" style="height: 100vh; width: 100vw;">
      <n-layout has-sider>
        <n-layout-sider
          bordered
          show-trigger="bar"
          collapse-mode="width"
          :collapsed-width="48"
          :width="240"
          style="height: 100vh; overflow: hidden;"
        >
          <Sidebar
            :conversations="chatStore.conversations"
            :active-id="chatStore.activeConversationId"
            @select="handleSelectConversation"
            @create="handleCreateConversation"
            @close="handleDeleteConversation"
            @rename="handleRenameConversation"
            @open-settings="showSettings = true"
          />
        </n-layout-sider>
        <n-layout-content style="height: 100vh;" content-style="overflow: hidden;">
          <ChatPage />
        </n-layout-content>
      </n-layout>
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
