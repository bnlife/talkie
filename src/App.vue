<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useSettingsStore } from '@/stores/settingsStore'
import { usePromptStore } from '@/stores/promptStore'
import { useMcpStore } from '@/stores/mcpStore'
import { watch } from 'vue'
import 'vue-sonner/style.css'
import { Toaster } from 'vue-sonner'
import Toolstrip from '@/components/app/Toolstrip.vue'
import ChatView from '@/pages/chat/ChatView.vue'
import SettingsView from '@/pages/settings/SettingsView.vue'
import KnowledgeView from '@/pages/knowledge/KnowledgeView.vue'
import PromptView from '@/pages/prompt/PromptView.vue'
import McpView from '@/pages/mcp/McpView.vue'

const settingsStore = useSettingsStore()
const promptStore = usePromptStore()
const mcpStore = useMcpStore()
const activeView = ref('chat')

onMounted(async () => {
  mcpStore.listenEvents()
  await Promise.all([
    mcpStore.loadData(),
    promptStore.loadPrompts(),
  ])
  // Set default prompt after prompts are loaded
  const def = promptStore.defaultPrompt
  if (def) {
    promptStore.selectPrompt(def.id)
  }
})

watch(
  () => settingsStore.darkMode,
  (darkMode) => {
    const el = document.documentElement
    if (darkMode) el.classList.add('dark')
    else el.classList.remove('dark')
  },
  { immediate: true }
)
</script>

<template>
  <Toaster />
  <div class="flex h-screen w-screen overflow-hidden bg-muted text-foreground">
    <Toolstrip :active-view="activeView" @select="activeView = $event" />
    <ChatView v-if="activeView === 'chat'" class="flex flex-1 overflow-hidden" />
    <SettingsView v-else-if="activeView === 'settings'" class="flex flex-1 flex-col overflow-hidden" />
    <KnowledgeView v-else-if="activeView === 'knowledge'" class="flex flex-1 flex-col overflow-hidden" />
    <PromptView v-else-if="activeView === 'prompt'" class="flex flex-1 flex-col overflow-hidden" />
    <McpView v-else-if="activeView === 'mcp'" class="flex flex-1 flex-col overflow-hidden" />
  </div>
</template>
