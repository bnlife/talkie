<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Minus, Maximize2, Minimize2, X, BookOpen } from 'lucide-vue-next'

const appWindow = getCurrentWindow()
const isMaximized = ref(false)

onMounted(async () => { isMaximized.value = await appWindow.isMaximized() })

async function minimizeWindow() { await appWindow.minimize() }
async function toggleMaximize() {
  if (isMaximized.value) { await appWindow.unmaximize() } else { await appWindow.maximize() }
  isMaximized.value = await appWindow.isMaximized()
}
async function closeWindow() { await appWindow.close() }
</script>

<template>
  <div class="flex h-full flex-col">
    <header
      data-tauri-drag-region
      class="flex h-9 shrink-0 items-center justify-between bg-muted px-3 select-none"
    >
      <span class="text-sm font-medium text-muted-foreground">知识库</span>
      <div class="flex items-center gap-0.5">
        <Button variant="ghost" size="icon" class="h-6 w-6 hover:bg-background" @click="minimizeWindow"><Minus class="h-3.5 w-3.5" /></Button>
        <Button variant="ghost" size="icon" class="h-6 w-6 hover:bg-background" @click="toggleMaximize">
          <Maximize2 v-if="!isMaximized" class="h-3.5 w-3.5" />
          <Minimize2 v-else class="h-3.5 w-3.5" />
        </Button>
        <Button variant="ghost" size="icon" class="h-6 w-6 hover:bg-destructive hover:text-destructive-foreground" @click="closeWindow"><X class="h-3.5 w-3.5" /></Button>
      </div>
    </header>
    <div class="flex flex-1 overflow-hidden p-1">
      <div class="flex flex-1 items-center justify-center rounded-lg border bg-background text-muted-foreground">
        <div class="flex flex-col items-center gap-2">
          <BookOpen class="size-10" />
          <p class="text-sm">知识库功能即将推出</p>
        </div>
      </div>
    </div>
  </div>
</template>
