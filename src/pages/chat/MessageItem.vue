<script setup lang="ts">
import { ref, computed } from 'vue'
import type { Message, SearchResult } from '@/types'
import { cn } from '@/lib/utils'
import { renderMarkdown } from '@/lib/markdown'
import { openUrl } from '@/bridge/settings'
import { Button } from '@/components/ui/button'
import { Avatar, AvatarFallback } from '@/components/ui/avatar'
import { Copy, Trash2, RefreshCw, Check, Globe, ChevronDown, ChevronUp } from 'lucide-vue-next'
import ThinkingBlock from './ThinkingBlock.vue'

const props = defineProps<{
  message: Message
  streaming?: boolean
  streamingThinking?: string
  streamingThinkingStart?: number
  streamingSearchResults?: SearchResult[] | null
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

// Thinking content: streaming or persisted
const thinkingContent = computed(() => {
  if (props.streamingThinking) return props.streamingThinking
  return props.message.thinking_content || ''
})
const showThinking = computed(() => thinkingContent.value.length > 0)

// Search results: streaming or persisted
const searchResults = computed<SearchResult[]>(() => {
  if (isUser.value) return []
  if (props.streamingSearchResults && props.streamingSearchResults.length > 0) return props.streamingSearchResults
  return props.message.search_results || []
})
const showSearchResults = computed(() => searchResults.value.length > 0)
const searchExpanded = ref(false)
const hasMoreResults = computed(() => searchResults.value.length > 3)
const visibleResults = computed(() => {
  if (searchExpanded.value || searchResults.value.length <= 3) return searchResults.value
  return searchResults.value.slice(0, 3)
})

// Build URL → citation index map
const urlIndexMap = computed(() => {
  const map = new Map<string, number>()
  searchResults.value.forEach((r, i) => {
    if (r.url) map.set(r.url, i + 1)
  })
  return map
})

// Rendered HTML with inline citations
const renderedHtml = computed(() => {
  if (isUser.value) return ''
  let html = renderMarkdown(props.message.content)
  if (searchResults.value.length === 0) return html

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
  try { return new URL(url).hostname.replace(/^www\./, '') } catch { return url }
}

function getFaviconUrl(url: string): string {
  try { return `https://www.google.com/s2/favicons?domain=${new URL(url).hostname}&sz=16` } catch { return '' }
}

async function handleCopy() {
  await navigator.clipboard.writeText(props.message.content)
  isCopied.value = true
  setTimeout(() => { isCopied.value = false }, 2000)
  emit('copy', props.message.content)
}

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
        <span class="text-base font-semibold text-foreground">{{ displayName }}</span>
        <span class="text-xs text-muted-foreground">{{ formattedTime }}</span>
      </div>

      <!-- Thinking 折叠块 -->
      <ThinkingBlock
        v-if="showThinking || (streaming && !message.content)"
        :thinking="thinkingContent"
        :streaming="streaming && !message.content"
        :start-time="streamingThinkingStart"
        :search-results="streamingSearchResults"
      />

      <!-- 消息正文 -->
      <div class="relative">
        <div class="p-0">
          <p v-if="isUser" class="text-base leading-relaxed whitespace-pre-wrap break-words">
            {{ message.content }}
          </p>
          <div
            v-else
            class="markdown-body text-base"
            v-html="renderedHtml"
            @click="onContentClick"
          />
          <span
            v-if="streaming && message.content"
            class="inline-block h-4 ml-0.5 -mb-0.5 text-muted-foreground animate-pulse"
          >
            |
          </span>
        </div>

        <!-- 搜索来源底栏 -->
        <div v-if="showSearchResults" class="mt-2">
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
          </div>
          <div class="mt-1 flex items-center gap-2">
            <span class="text-[11px] text-muted-foreground">
              <Globe class="size-3 inline -mt-0.5" />
              {{ searchResults.length }} 个来源
            </span>
            <button
              v-if="hasMoreResults"
              class="text-[11px] text-muted-foreground hover:text-foreground transition-colors"
              @click="searchExpanded = !searchExpanded"
            >
              <template v-if="searchExpanded">
                <ChevronUp class="size-3 inline -mt-0.5" /> 收起
              </template>
              <template v-else>
                展开更多
              </template>
            </button>
          </div>
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
          class="mt-1 text-xs text-muted-foreground opacity-0 group-hover:opacity-100 transition-opacity"
        >
          {{ message.token_count }} tokens
        </div>
      </div>
    </div>
  </div>
</template>
