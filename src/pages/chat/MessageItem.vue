<script setup lang="ts">
import { ref, computed } from 'vue'
import type { Message } from '@/types'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Avatar, AvatarFallback } from '@/components/ui/avatar'
import { Copy, Trash2, RefreshCw, Check } from 'lucide-vue-next'

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

async function handleCopy() {
  await navigator.clipboard.writeText(props.message.content)
  isCopied.value = true
  setTimeout(() => { isCopied.value = false }, 2000)
  emit('copy', props.message.content)
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
        <div
          :class="cn(
            'rounded-lg p-3 shadow-xs',
            isUser
              ? 'bg-primary text-primary-foreground'
              : 'bg-muted text-foreground'
          )"
        >
          <p class="text-sm leading-relaxed whitespace-pre-wrap break-words">
            {{ message.content }}
          </p>
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
      </div>
    </div>
  </div>
</template>
