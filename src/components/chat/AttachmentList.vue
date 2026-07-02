<script setup lang="ts">
import { Paperclip, Download } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'

defineProps<{
  attachments: Array<{ name: string; content?: string }>
}>()

function downloadAttachment(att: { name: string; content?: string }) {
  if (!att.content) return
  const blob = new Blob([att.content], { type: 'text/plain;charset=utf-8' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = att.name
  a.click()
  URL.revokeObjectURL(url)
}
</script>

<template>
  <div v-if="attachments && attachments.length > 0" class="mb-1.5 flex flex-wrap gap-1.5">
    <span
      v-for="att in attachments"
      :key="att.name"
      class="inline-flex items-center gap-1 rounded-md bg-muted px-2 py-0.5 text-xs text-muted-foreground"
    >
      <Paperclip class="size-3 shrink-0" />
      <span class="max-w-[140px] truncate">{{ att.name }}</span>
      <Button variant="ghost" size="icon" class="ml-0.5" @click="downloadAttachment(att)">
        <Download class="size-3" />
      </Button>
    </span>
  </div>
</template>