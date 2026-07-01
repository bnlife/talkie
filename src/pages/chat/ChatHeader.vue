<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useChatStore } from '@/stores/chatStore'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { Button } from '@/components/ui/button'
import {
  PanelLeftOpen, PanelLeftClose,
  Minus, Maximize2, Minimize2, X,
} from 'lucide-vue-next'

const props = defineProps<{
  sidebarCollapsed: boolean
}>()

const emit = defineEmits<{
  'toggle-sidebar': []
}>()

const chatStore = useChatStore()
const appWindow = getCurrentWindow()
const isMaximized = ref(false)

onMounted(async () => {
  isMaximized.value = await appWindow.isMaximized()
})

async function minimizeWindow() { await appWindow.minimize() }
async function toggleMaximize() {
  if (isMaximized.value) { await appWindow.unmaximize() } else { await appWindow.maximize() }
  isMaximized.value = await appWindow.isMaximized()
}
async function closeWindow() { await appWindow.close() }
</script>

<template>
  <header
    data-tauri-drag-region
    class="flex h-9 shrink-0 items-center justify-between bg-muted px-3 select-none"
  >
    <div class="flex items-center gap-2">
      <Button variant="ghost" size="icon" class="h-6 w-6" @click.stop="emit('toggle-sidebar')">
        <PanelLeftClose v-if="!sidebarCollapsed" class="h-3.5 w-3.5" />
        <PanelLeftOpen v-else class="h-3.5 w-3.5" />
      </Button>
      <span class="text-sm font-medium text-muted-foreground truncate">
        {{ chatStore.activeConversation?.title || '对话' }}
      </span>
    </div>
    <div class="flex items-center gap-0.5">
      <Button variant="ghost" size="icon" class="h-6 w-6 hover:bg-background" @click="minimizeWindow">
        <Minus class="h-3.5 w-3.5" />
      </Button>
      <Button variant="ghost" size="icon" class="h-6 w-6 hover:bg-background" @click="toggleMaximize">
        <Maximize2 v-if="!isMaximized" class="h-3.5 w-3.5" />
        <Minimize2 v-else class="h-3.5 w-3.5" />
      </Button>
      <Button variant="ghost" size="icon" class="h-6 w-6 hover:bg-destructive hover:text-destructive-foreground" @click="closeWindow">
        <X class="h-3.5 w-3.5" />
      </Button>
    </div>
  </header>
</template>
