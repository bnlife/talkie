<script setup lang="ts">
import { computed, toRef } from 'vue'
import type { SearchResult } from '@/types'
import { openUrl } from '@/bridge/settings'
import { useMessageRender } from '@/composables/useMessageRender'

const props = defineProps<{
  content: string
  isUser: boolean
  streaming?: boolean
  searchResults: SearchResult[]
}>()

const contentRef = toRef(props, 'content')
const isUserRef = toRef(props, 'isUser')
const searchResultsRef = toRef(props, 'searchResults')

const { renderedHtml } = useMessageRender(contentRef, searchResultsRef, isUserRef)

function onContentClick(e: MouseEvent) {
  const target = e.target as HTMLElement
  if (target.classList.contains('search-citation')) {
    e.preventDefault()
    const url = target.getAttribute('data-url')
    if (url) openUrl(url)
    return
  }
  const link = target.closest('a')
  if (link) {
    e.preventDefault()
    const href = link.getAttribute('href')
    if (href) openUrl(href)
  }
}
</script>

<template>
  <div class="relative">
    <div class="p-0">
      <p v-if="isUser" class="text-base leading-relaxed whitespace-pre-wrap break-words">
        {{ content }}
      </p>
      <div
        v-else
        class="markdown-body text-base"
        v-html="renderedHtml"
        @click="onContentClick"
      />
      <span
        v-if="streaming && content"
        class="inline-block h-4 ml-0.5 -mb-0.5 text-muted-foreground animate-pulse"
      >
        |
      </span>
    </div>
  </div>
</template>