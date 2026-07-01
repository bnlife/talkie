<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import type { Message, SearchResult } from '@/types'
import { cn } from '@/lib/utils'
import { renderMarkdown } from '@/lib/markdown'
import { openUrl } from '@/bridge/settings'
import { Button } from '@/components/ui/button'
import { Avatar, AvatarFallback } from '@/components/ui/avatar'
import { Copy, Trash2, RefreshCw, Check, Globe, ChevronDown, ChevronUp } from 'lucide-vue-next'

const props = defineProps<{
  message: Message
  streaming?: boolean
  isLast?: boolean
  modelName?: string
}>()

const emit = defineEmits<{
  copy: [content: string]
  delete: [messageId: string]
  regenerate: []
}>()

const isUser = computed(() => props.message.role === 'user')
const isCopied = ref(false)

const avatarInitial = computed(() => isUser.value ? '你' : 'AI')
const displayName = computed(() => isUser.value ? '你' : (props.modelName || 'AI'))
const formattedTime = computed(() => {
  const date = new Date(props.message.created_at)
  return date.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })
})

// Search results
const searchResults = computed<SearchResult[]>(() => {
  if (isUser.value) return []
  return props.message.search_results || []
})
const showSearchResults = computed(() => searchResults.value.length > 0 && !props.streaming)
const searchExpanded = ref(false)
const hasMoreResults = computed(() => searchResults.value.length > 5)
const visibleResults = computed(() => {
  if (searchExpanded.value || searchResults.value.length <= 5) return searchResults.value
  return searchResults.value.slice(0, 5)
})

// When search results first appear, expand briefly then auto-collapse
const hasAnimated = ref(false)
watch(showSearchResults, (show) => {
  if (show && !hasAnimated.value) {
    hasAnimated.value = true
    nextTick(() => {
      searchExpanded.value = true
      if (searchResults.value.length > 5) {
        setTimeout(() => { searchExpanded.value = false }, 3000)
      }
    })
  }
})

// Build URL → citation index map
const urlIndexMap = computed(() => {
  const map = new Map<string, number>()
  searchResults.value.forEach((r, i) => {
    if (r.url) map.set(r.url, i + 1)
  })
  return map
})

// Rendered HTML with inline citations injected
const renderedHtml = computed(() => {
  if (isUser.value) return ''
  let html = renderMarkdown(props.message.content)
  if (searchResults.value.length === 0) return html

  // Pass 1: Insert citation after markdown links [text](url) whose href matches a search result
  html = html.replace(
    /<a\s+[^>]*href="([^"]*)"[^>]*>(.*?)<\/a>/gi,
    (match, href) => {
      const idx = urlIndexMap.value.get(href)
      if (idx) {
        const url = searchResults.value[idx - 1]?.url ?? href
        return `${match}<sup class="search-citation" data-url="${url}">${idx}</sup>`
      }
      return match
    }
  )

  // Pass 2: Convert plain text [N] (not inside <code>/<pre>) into clickable citations
  html = html.replace(
    /(?<!<[^>]*?)\[(\d+)\](?![^<]*?<\/(?:code|pre)>)/g,
    (match, num) => {
      const idx = parseInt(num, 10)
      if (idx >= 1 && idx <= searchResults.value.length) {
        const url = searchResults.value[idx - 1]?.url ?? ''
        return `<sup class="search-citation" data-url="${url}">${idx}</sup>`
      }
      return match
    }
  )

  return html
})

function getDomain(url: string): string {
  try {
    return new URL(url).hostname.replace(/^www\./, '')
  } catch {
    return url
  }
}

function getFaviconUrl(url: string): string {
  try {
    const domain = new URL(url).hostname
    return `https://www.google.com/s2/favicons?domain=${domain}&sz=16`
  } catch {
    return ''
  }
}

async function handleCopy() {
  await navigator.clipboard.writeText(props.message.content)
  isCopied.value = true
  setTimeout(() => { isCopied.value = false }, 2000)
  emit('copy', props.message.content)
}

function onContentClick(e: MouseEvent) {
  const target = e.target as HTMLElement
  // Handle citation click
  if (target.classList.contains('search-citation')) {
    e.preventDefault()
    const url = target.getAttribute('data-url')
    if (url) openUrl(url)
    return
  }
  // Handle link click
  const link = target.closest('a')
  if (link) {
    e.preventDefault()
    const href = link.getAttribute('href')
    if (href) openUrl(href)
  }
}
</script>

<template>
  <div class="group relative flex w-full gap-3 pb-4">
    <!-- 头像 -->
    <Avatar
      :class="cn(
        'h-8 w-8 shrink-0 mt-1',
        isUser ? 'bg-primary text-primary-foreground' : 'bg-muted text-muted-foreground'
      )"
    >
      <AvatarFallback class="text-xs">{{ avatarInitial }}</AvatarFallback>
    </Avatar>

    <!-- 右侧内容 -->
    <div class="flex-1 min-w-0">
      <!-- 用户名/模型名 -->
      <div class="flex items-center gap-2 mb-1">
        <span class="text-sm font-medium text-foreground">{{ displayName }}</span>
        <span class="text-xs text-muted-foreground">{{ formattedTime }}</span>
      </div>

      <!-- 搜索来源（消息上方） -->
      <Transition name="sources">
        <div v-if="showSearchResults" class="mb-2">
          <div class="flex items-center gap-1.5 mb-1.5 text-xs text-muted-foreground">
            <Globe class="size-3" />
            <span>{{ searchResults.length }} 个来源</span>
          </div>
          <div class="flex flex-wrap gap-1.5">
            <button
              v-for="(result, idx) in visibleResults"
              :key="idx"
              class="inline-flex items-center gap-1.5 rounded-md border bg-muted/40 px-2 py-1 text-xs transition-colors hover:bg-muted/70 max-w-48"
              @click="openUrl(result.url)"
            >
              <img
                :src="getFaviconUrl(result.url)"
                class="size-3 shrink-0 rounded-sm"
                @error="($event.target as HTMLImageElement).style.display='none'"
              />
              <span class="truncate">{{ result.title || getDomain(result.url) }}</span>
            </button>
            <button
              v-if="hasMoreResults"
              class="inline-flex items-center rounded-md border border-dashed px-2 py-1 text-xs text-muted-foreground transition-colors hover:bg-muted/40"
              @click="searchExpanded = !searchExpanded"
            >
              <template v-if="searchExpanded">
                <ChevronUp class="size-3 mr-0.5" /> 收起
              </template>
              <template v-else>
                +{{ searchResults.length - 5 }}
              </template>
            </button>
          </div>
        </div>
      </Transition>

      <!-- 消息正文 -->
      <div class="relative">
        <div class="p-0">
          <p v-if="isUser" class="text-sm leading-relaxed whitespace-pre-wrap break-words">
            {{ message.content }}
          </p>
          <div
            v-else
            class="markdown-body text-sm"
            v-html="renderedHtml"
            @click="onContentClick"
          />
          <span
            v-if="streaming"
            class="inline-block h-4 ml-0.5 -mb-0.5 text-muted-foreground animate-pulse"
          >
            |
          </span>
        </div>

        <!-- 操作按钮 -->
        <div
          v-if="!streaming"
          class="absolute -bottom-7 left-0 flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity"
        >
          <Button variant="ghost" size="icon" class="h-6 w-6" @click="handleCopy">
            <Check v-if="isCopied" class="h-3 w-3 text-green-500" />
            <Copy v-else class="h-3 w-3" />
          </Button>
          <Button variant="ghost" size="icon" class="h-6 w-6" @click="emit('delete', message.id)">
            <Trash2 class="h-3 w-3" />
          </Button>
          <Button v-if="!isUser && isLast" variant="ghost" size="icon" class="h-6 w-6" @click="emit('regenerate')">
            <RefreshCw class="h-3 w-3" />
          </Button>
        </div>

        <!-- Token 消耗 -->
        <div
          v-if="!isUser && message.token_count && !streaming"
          class="mt-1 text-xs text-muted-foreground"
        >
          {{ message.token_count }} tokens
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.sources-enter-active {
  transition: all 0.3s ease-out;
}
.sources-leave-active {
  transition: all 0.2s ease-in;
}
.sources-enter-from {
  opacity: 0;
  transform: translateY(-6px);
}
.sources-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
</style>
