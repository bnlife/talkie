<script setup lang="ts">
import { Button } from '@/components/ui/button'
import { MessageSquare, BookOpen, Settings, Moon, Sun } from 'lucide-vue-next'
import { useSettingsStore } from '@/stores/settingsStore'

defineProps<{ activeView: string }>()
const emit = defineEmits<{ select: [view: string] }>()
const settingsStore = useSettingsStore()
</script>

<template>
  <div class="flex w-10 flex-col items-center gap-2 bg-muted py-2" data-tauri-drag-region>
    <Button
      variant="ghost"
      size="icon-sm"
      :class="activeView === 'chat' ? 'bg-background text-foreground shadow-sm' : 'hover:bg-background'"
      @click="emit('select', 'chat')"
    >
      <MessageSquare class="size-4" />
    </Button>
    <Button
      variant="ghost"
      size="icon-sm"
      :class="activeView === 'knowledge' ? 'bg-background text-foreground shadow-sm' : 'hover:bg-background'"
      @click="emit('select', 'knowledge')"
    >
      <BookOpen class="size-4" />
    </Button>
    <div class="mt-auto flex flex-col items-center gap-2">
      <Button
        variant="ghost"
        size="icon-sm"
        class="hover:bg-background"
        @click="settingsStore.updateSettings({ darkMode: !settingsStore.darkMode })"
      >
        <Moon v-if="!settingsStore.darkMode" class="size-4" />
        <Sun v-else class="size-4" />
      </Button>
      <Button
        variant="ghost"
        size="icon-sm"
        :class="activeView === 'settings' ? 'bg-background text-foreground shadow-sm' : 'hover:bg-background'"
        @click="emit('select', 'settings')"
      >
        <Settings class="size-4" />
      </Button>
    </div>
  </div>
</template>
