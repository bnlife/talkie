<script setup lang="ts">
import { computed } from 'vue'
import type { Message } from '@/types'
import { cn } from '@/lib/utils'
import { Card, CardContent, CardHeader } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'

const props = defineProps<{
  message: Message
  streaming?: boolean
}>()

const isUser = computed(() => props.message.role === 'user')

const roleLabel = computed(() => {
  switch (props.message.role) {
    case 'user':
      return '用户'
    case 'assistant':
      return '助手'
    default:
      return '系统'
  }
})

const badgeVariant = computed(() => (isUser.value ? 'default' : 'secondary'))
</script>

<template>
  <div :class="cn('flex w-full', isUser ? 'justify-end' : 'justify-start')">
    <Card
      :class="cn(
        'max-w-[75%] shadow-xs',
        isUser
          ? 'bg-primary text-primary-foreground border-primary'
          : 'bg-muted'
      )"
    >
      <CardHeader class="p-3 pb-1">
        <Badge
          :variant="badgeVariant"
          :class="cn('w-fit text-xs', isUser && 'bg-primary-foreground text-primary')"
        >
          {{ roleLabel }}
        </Badge>
      </CardHeader>
      <CardContent class="p-3 pt-0">
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
  </div>
</template>
