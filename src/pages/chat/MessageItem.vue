<script setup lang="ts">
import { computed } from 'vue'
import type { Message } from '@/types'
import { cn } from '@/lib/utils'
import { Card, CardContent } from '@/components/ui/card'

const props = defineProps<{
  message: Message
  streaming?: boolean
}>()

const isUser = computed(() => props.message.role === 'user')

</script>

<template>
  <div
    :class="cn(
      'flex w-full',
      isUser ? 'justify-end' : 'justify-start'
    )"
  >
    <Card
      :class="cn(
        'max-w-[75%] shadow-xs',
        isUser
          ? 'bg-primary text-primary-foreground border-primary'
          : 'bg-muted text-foreground'
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
  </div>
</template>
