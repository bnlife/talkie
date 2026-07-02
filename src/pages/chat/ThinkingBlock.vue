<script setup lang="ts">
import { ref, computed, onUnmounted } from 'vue'
import { cn } from '@/lib/utils'
import { renderMarkdown } from '@/lib/markdown'
import { ChevronDown, ChevronUp, Loader2, Check, Globe, Brain } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import type { SearchResult } from '@/types'

const props = defineProps<{
  thinking: string
  streaming?: boolean
  startTime?: number
  searchResults?: SearchResult[] | null
}>()

const expanded = ref(false)

const elapsed = computed(() => {
  if (!props.startTime) return 0
  const end = props.streaming ? Date.now() : (props.startTime + 1000)
  return Math.round((end - props.startTime) / 1000)
})

// Live timer for streaming
let timer: ReturnType<typeof setInterval> | null = null
if (props.streaming && props.startTime) {
  timer = setInterval(() => {
    // Force reactivity update
  }, 1000)
}
onUnmounted(() => { if (timer) clearInterval(timer) })

const thinkingHtml = computed(() => {
  if (!props.thinking) return ''
  return renderMarkdown(props.thinking)
})

const hasSearchResults = computed(() => (props.searchResults?.length ?? 0) > 0)
</script>

<template>
  <div class="mb-2">
    <!-- Collapsed header -->
    <Button
      variant="ghost"
      class="flex w-full justify-start gap-1.5 px-2 py-1 text-sm text-muted-foreground"
      @click="expanded = !expanded"
    >
      <template v-if="streaming">
        <Loader2 class="size-3 animate-spin" />
      </template>
      <template v-else>
        <Brain class="size-3" />
      </template>
      <template v-if="streaming">
        <span>思考中...</span>
      </template>
      <template v-else>
        <span>思考了 {{ elapsed }} 秒</span>
      </template>
      <ChevronDown v-if="!expanded" class="size-3 ml-auto" />
      <ChevronUp v-else class="size-3 ml-auto" />
    </Button>

    <!-- Expanded content -->
    <div v-if="expanded" class="mt-1 rounded-md border bg-muted/30 p-2.5">
      <!-- Search status -->
      <div v-if="hasSearchResults" class="mb-2 space-y-0.5">
        <div
          v-for="(result, idx) in searchResults"
          :key="idx"
          class="flex items-center gap-1.5 text-sm text-muted-foreground"
        >
          <Check class="size-3 shrink-0 text-success" />
          <Globe class="size-3 shrink-0" />
          <span class="truncate">{{ result.title || result.url }}</span>
        </div>
      </div>

      <!-- Thinking content -->
      <div
        v-if="thinking"
        class="markdown-body text-sm text-muted-foreground max-h-60 overflow-y-auto"
        v-html="thinkingHtml"
      />
      <div v-else-if="streaming" class="text-sm text-muted-foreground/60 italic">
        等待思考内容...
      </div>
    </div>
  </div>
</template>
