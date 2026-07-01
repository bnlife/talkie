<script setup lang="ts">
import { ref, computed } from 'vue'
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
const renderedHtml = computed(() => isUser.value ? '' : renderMarkdown(props.message.content))

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
const showSearchResults = computed(() => searchResults.value.length > 0)
const searchExpanded = ref(false)
const visibleResults = computed(() => {
  if (searchExpanded.value || searchResults.value.length <= 3) return searchResults.value
  return searchResults.value.slice(0, 3)
})
const hasMoreResults = computed(() => searchResults.value.length > 3)

function getDomain(url: string): string {
  try {
    return new URL(url).hostname.replace(/^www\./, '')
  } catch {
    return url
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
  if (target.tagName === 'A') {
    e.preventDefault()
    const href = target.getAttribute('href')
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
          <Button
            variant="ghost"
            size="icon"
            class="h-6 w-6"
            @click="handleCopy"
          >
            <Check v-if="isCopied" class="h-3 w-3 text-green-500" />
            <Copy v-else class="h-3 w-3" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            class="h-6 w-6"
            @click="emit('delete', message.id)"
          >
            <Trash2 class="h-3 w-3" />
          </Button>
          <Button
            v-if="!isUser && isLast"
            variant="ghost"
            size="icon"
            class="h-6 w-6"
            @click="emit('regenerate')"
          >
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

        <!-- 搜索来源卡片 -->
        <div
          v-if="showSearchResults && !streaming"
          class="mt-2 rounded-lg border bg-muted/30 p-2"
        >
          <div class="flex items-center gap-1.5 mb-1.5 text-xs font-medium text-foreground">
            <Globe class="size-3" />
            <span>搜索了 {{ searchResults.length }} 个来源</span>
          </div>
          <div class="space-y-1">
            <div
              v-for="(result, idx) in visibleResults"
              :key="idx"
              class="group/item flex items-start gap-2 rounded px-1.5 py-1 transition-colors hover:bg-foreground/5 cursor-pointer"
              @click="openUrl(result.url)"
            >
              <Globe class="size-3 mt-0.5 shrink-0 text-muted-foreground" />
              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-2">
                  <span class="text-xs font-medium text-foreground truncate">{{ result.title }}</span>
                  <span class="text-[10px] text-muted-foreground shrink-0">{{ getDomain(result.url) }}</span>
                </div>
                <p v-if="result.snippet" class="text-[11px] text-muted-foreground line-clamp-2 mt-0.5">
                  {{ result.snippet }}
                </p>
              </div>
            </div>
          </div>
          <button
            v-if="hasMoreResults"
            class="mt-1 flex items-center gap-1 text-[11px] text-muted-foreground hover:text-foreground transition-colors"
            @click="searchExpanded = !searchExpanded"
          >
            <template v-if="searchExpanded">
              <ChevronUp class="size-3" />
              收起
            </template>
            <template v-else>
              <ChevronDown class="size-3" />
              展开更多 ({{ searchResults.length - 3 }})
            </template>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
