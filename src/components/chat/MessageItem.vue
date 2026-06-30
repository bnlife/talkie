<script setup lang="ts">
import type { Message } from '../../types'
import { Card, CardHeader, CardContent } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'

const props = defineProps<{
  message: Message
  isStreaming?: boolean
}>()
</script>

<template>
  <div
    :class="[
      'flex w-full mb-1',
      props.message.role === 'user' ? 'justify-end' : 'justify-start'
    ]"
  >
    <Card
      :class="[
        
        props.message.role === 'user' ? 'bg-page' : 'bg-surface border-border'
      ]"
    >
      <CardHeader class="p-normal pb-0">
        <Badge :variant="props.message.role === 'user' ? 'default' : 'secondary'">
          {{ props.message.role === 'user' ? '用户' : '助手' }}
        </Badge>
      </CardHeader>
      <CardContent class="p-normal pt-tight">
        <span class="text-main">
          {{ props.message.content }}<template v-if="props.isStreaming"><span class="text-hint">|</span></template>
        </span>
      </CardContent>
    </Card>
  </div>
</template>

