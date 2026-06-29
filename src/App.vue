<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useChatStore } from './stores/chatStore'
import { useSettingsStore } from './stores/settingsStore'
import Sidebar from './components/chat/Sidebar.vue'
import ChatPage from './pages/ChatPage.vue'
import SettingsPanel from './components/settings/SettingsPanel.vue'
import type { Settings } from './types'

const chatStore = useChatStore()
const settingsStore = useSettingsStore()

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

function handleUpdateSettings(partial: Partial<Settings>) {
  settingsStore.updateSettings(partial)
}

function handleTestConnection() {
  settingsStore.testConnection()
}
</script>

<template>
  <n-layout position="absolute" style="height: 100vh; width: 100vw;">
    <n-layout has-sider>
      <n-layout-sider
        bordered
        show-trigger="bar"
        collapse-mode="width"
        :collapsed-width="48"
        :width="240"
        :native-scrollbar="false"
      >
        <Sidebar
          :conversations="chatStore.conversations"
          :active-id="chatStore.activeConversationId"
          @select="handleSelectConversation"
          @create="handleCreateConversation"
          @close="handleDeleteConversation"
          @rename="handleRenameConversation"
        />
      </n-layout-sider>
      <n-layout-content style="height: 100vh;">
        <ChatPage />
      </n-layout-content>
    </n-layout>
    <n-button
      secondary
      circle
      style="position: fixed; bottom: 24px; right: 24px; z-index: 1000;"
      @click="showSettings = true"
    >
      设置
    </n-button>
  </n-layout>

  <n-modal v-model:show="showSettings" title="设置" preset="card" style="width: 480px;">
    <SettingsPanel
      :settings="settingsStore.$state"
      @update="handleUpdateSettings"
      @test-connection="handleTestConnection"
    />
  </n-modal>
</template>
