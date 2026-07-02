<script setup lang="ts">
import { computed } from 'vue'
import type { Message, SearchResult } from '@/types'
import ThinkingBlock from './ThinkingBlock.vue'
import MessageHeader from '@/components/chat/MessageHeader.vue'
import AttachmentList from '@/components/chat/AttachmentList.vue'
import MessageContent from '@/components/chat/MessageContent.vue'
import SearchSources from '@/components/chat/SearchSources.vue'
import MessageActions from '@/components/chat/MessageActions.vue'

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
</script>

<template>
  <div class="group relative flex w-full gap-3 pb-4">
    <!-- 头像 -->
    <MessageHeader
      :is-user="isUser"
      :model-name="modelName"
      :created-at="message.created_at"
    />

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
        <!-- 附件列表 -->
        <AttachmentList
          v-if="isUser"
          :attachments="message.attachments || []"
        />

        <!-- 消息内容 -->
        <MessageContent
          :content="message.content"
          :is-user="isUser"
          :streaming="streaming"
          :search-results="searchResults"
        />

        <!-- 搜索来源 -->
        <SearchSources
          v-if="!isUser"
          :search-results="searchResults"
        />

        <!-- 操作按钮 -->
        <MessageActions
          v-if="!streaming"
          :content="message.content"
          :message-id="message.id"
          :is-user="isUser"
          :is-last="isLast"
          :token-count="message.token_count"
          @copy="(content) => emit('copy', content)"
          @delete="(messageId) => emit('delete', messageId)"
          @regenerate="emit('regenerate')"
        />
      </div>
    </div>
  </div>
</template>