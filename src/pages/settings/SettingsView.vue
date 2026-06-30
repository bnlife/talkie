<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useSettingsStore } from '@/stores/settingsStore'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Moon, Sun, Minus, Maximize2, Minimize2, X } from 'lucide-vue-next'
import SettingsPanel from './SettingsPanel.vue'

const settingsStore = useSettingsStore()
const appWindow = getCurrentWindow()
const isMaximized = ref(false)

onMounted(async () => { isMaximized.value = await appWindow.isMaximized() })

async function minimizeWindow() { await appWindow.minimize() }
async function toggleMaximize() {
  if (isMaximized.value) { await appWindow.unmaximize() } else { await appWindow.maximize() }
  isMaximized.value = await appWindow.isMaximized()
}
async function closeWindow() { await appWindow.close() }

async function handleUpdate(partial: any) {
  await settingsStore.updateSettings(partial)
}
async function handleTestConnection() {
  const r = await settingsStore.testConnection()
  if (r.ok) alert('连接成功')
  else alert(r.error || '连接失败')
}
</script>

<template>
  <div class="flex h-full flex-col">
    <header
      data-tauri-drag-region
      class="flex h-9 shrink-0 items-center justify-between border-b bg-background px-3 select-none"
    >
      <span class="text-sm font-medium text-muted-foreground">设置</span>
      <div class="flex items-center gap-0.5">
        <Button variant="ghost" size="icon" class="h-6 w-6" @click.stop="settingsStore.updateSettings({ darkMode: !settingsStore.darkMode })">
          <Moon v-if="!settingsStore.darkMode" class="h-3.5 w-3.5" />
          <Sun v-else class="h-3.5 w-3.5" />
        </Button>
        <Button variant="ghost" size="icon" class="h-6 w-6" @click.stop="minimizeWindow"><Minus class="h-3.5 w-3.5" /></Button>
        <Button variant="ghost" size="icon" class="h-6 w-6" @click.stop="toggleMaximize">
          <Maximize2 v-if="!isMaximized" class="h-3.5 w-3.5" />
          <Minimize2 v-else class="h-3.5 w-3.5" />
        </Button>
        <Button variant="ghost" size="icon" class="h-6 w-6 hover:bg-destructive hover:text-destructive-foreground" @click.stop="closeWindow"><X class="h-3.5 w-3.5" /></Button>
      </div>
    </header>
    <div class="flex-1 overflow-y-auto p-4">
      <SettingsPanel :settings="settingsStore.$state" @update="handleUpdate" @test-connection="handleTestConnection" />
    </div>
  </div>
</template>
