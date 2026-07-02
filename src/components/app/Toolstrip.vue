<script setup lang="ts">
import { Button } from '@/components/ui/button'
import { MessageSquare, BookOpen, Settings, Moon, Sun, FileText, Puzzle } from 'lucide-vue-next'
import { useSettingsStore } from '@/stores/settingsStore'

defineProps<{ activeView: string }>()
const emit = defineEmits<{ select: [view: string] }>()
const settingsStore = useSettingsStore()
</script>

<template>
  <div class="flex w-10 flex-col items-center gap-2 bg-muted py-2" data-tauri-drag-region>
    <Button
      :variant="activeView === 'chat' ? 'active' : 'ghost'"
      size="icon"
      @click="emit('select', 'chat')"
    >
      <MessageSquare class="size-4" />
    </Button>
    <Button
      :variant="activeView === 'knowledge' ? 'active' : 'ghost'"
      size="icon"
      @click="emit('select', 'knowledge')"
    >
      <BookOpen class="size-4" />
    </Button>
    <Button
      :variant="activeView === 'prompt' ? 'active' : 'ghost'"
      size="icon"
      @click="emit('select', 'prompt')"
    >
      <FileText class="size-4" />
    </Button>
    <Button
      :variant="activeView === 'mcp' ? 'active' : 'ghost'"
      size="icon"
      @click="emit('select', 'mcp')"
    >
      <Puzzle class="size-4" />
    </Button>
    <div class="mt-auto flex flex-col items-center gap-2">
      <Button
        variant="ghost"
        size="icon"
        @click="settingsStore.darkMode = !settingsStore.darkMode; settingsStore.saveSettings()"
      >
        <Moon v-if="!settingsStore.darkMode" class="size-4" />
        <Sun v-else class="size-4" />
      </Button>
      <Button
        :variant="activeView === 'settings' ? 'active' : 'ghost'"
        size="icon"
        @click="emit('select', 'settings')"
      >
        <Settings class="size-4" />
      </Button>
    </div>
  </div>
</template>
