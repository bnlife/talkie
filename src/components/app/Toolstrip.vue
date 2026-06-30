<script setup lang="ts">
import { Button } from '@/components/ui/button'
import { MessageSquare, BookOpen, Settings } from 'lucide-vue-next'

defineProps<{ activeView: string }>()
const emit = defineEmits<{ select: [view: string] }>()
</script>

<template>
  <div class="flex w-10 flex-col items-center gap-2 border-r bg-background py-2" data-tauri-drag-region>
    <Button
      variant="ghost"
      size="icon-sm"
      :class="activeView === 'chat' ? 'bg-accent text-accent-foreground' : ''"
      @click="emit('select', 'chat')"
    >
      <MessageSquare class="size-4" />
    </Button>
    <Button
      variant="ghost"
      size="icon-sm"
      :class="activeView === 'knowledge' ? 'bg-accent text-accent-foreground' : ''"
      @click="emit('select', 'knowledge')"
    >
      <BookOpen class="size-4" />
    </Button>
    <Button
      variant="ghost"
      size="icon-sm"
      :class="activeView === 'settings' ? 'bg-accent text-accent-foreground' : ''"
      @click="emit('select', 'settings')"
    >
      <Settings class="size-4" />
    </Button>
  </div>
</template>
