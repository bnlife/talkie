<script setup lang="ts">
import { computed } from 'vue'
import type { Message } from '@/types'
import { cn } from '@/lib/utils'
import { Card, CardContent } from '@/components/ui/card'
import { Avatar, AvatarFallback } from '@/components/ui/avatar'

const props = defineProps<{
  message: Message
  streaming?: boolean
}>()

const isUser = computed(() => props.message.role === 'user')
const isAssistant = computed(() => props.message.role === 'assistant')

const avatarInitial = computed(() => {
  switch (props.message.role) {
    case 'user':
      return '你'
    case 'assistant':
      return '助'
    default:
      return '系'
  }
})

const avatarColorClass = computed(() => {
  switch (props.message.role) {
    case 'user':
      return 'bg-primary text-primary-foreground'
    case 'assistant':
      return 'bg-accent text-accent-foreground'
    default:
      return 'bg-muted text-muted-foreground'
  }
})

</script>

<template>
  <div
    :class="cn(
      'flex w-full items-end gap-2',
      isUser ? 'justify-end' : 'justify-start'
    )"
  >
    <!-- Assistant avatar (left) -->
    <Avatar
      v-if="isAssistant"
      size="sm"
      shape="square"
      :class="cn('h-8 w-8 shrink-0', avatarColorClass)"
    >
      <AvatarFallback>{{ avatarInitial }}</AvatarFallback>
    </Avatar>

    <Card
      :class="cn(
        'max-w-[75%] shadow-xs',
        isUser
          ? 'bg-primary text-primary-foreground border-primary'
          : 'bg-muted'
      )"
    >
      <CardContent class="p-3">
        <p class="text-sm leading-relaxed whitespace-pre-wrap break-words">
          {{ message.content }}
        </p>
        <span
          v-if="streaming"
          class="inline-block h-4 ml-0.5 -mb-0.5 text-muted-foreground animate-pulse"
        >
          |
        </span>
      </CardContent>
    </Card>

    <!-- User avatar (right) -->
    <Avatar
      v-if="isUser"
      size="sm"
      shape="square"
      :class="cn('h-8 w-8 shrink-0', avatarColorClass)"
    >
      <AvatarFallback>{{ avatarInitial }}</AvatarFallback>
    </Avatar>
  </div>
</template>
