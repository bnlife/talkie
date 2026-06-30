<script setup lang="ts">
import { ref } from 'vue'
import { useSettingsStore } from '@/stores/settingsStore'
import { watch } from 'vue'
import 'vue-sonner/style.css'
import { Toaster } from 'vue-sonner'
import Toolstrip from '@/components/app/Toolstrip.vue'
import ChatView from '@/pages/chat/ChatView.vue'
import SettingsView from '@/pages/settings/SettingsView.vue'
import KnowledgeView from '@/pages/knowledge/KnowledgeView.vue'
import PromptView from '@/pages/prompt/PromptView.vue'

const settingsStore = useSettingsStore()
const activeView = ref('chat')

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
  </div>
</template>
