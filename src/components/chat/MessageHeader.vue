<script setup lang="ts">
import { computed } from 'vue'
import { Avatar, AvatarFallback } from '@/components/ui/avatar'
import { cn } from '@/lib/utils'

const props = defineProps<{
  isUser: boolean
  modelName?: string
  createdAt: number
}>()

const avatarInitial = computed(() => props.isUser ? '你' : 'AI')
const displayName = computed(() => props.isUser ? '你' : (props.modelName || 'AI'))
const formattedTime = computed(() => {
  const date = new Date(props.createdAt)
  return date.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })
})
</script>

<template>
  <Avatar
    shape="square"
    :class="cn(
      'h-8 w-8 shrink-0 mt-1',
      isUser ? 'bg-primary text-primary-foreground' : 'bg-muted text-muted-foreground'
    )"
  >
    <AvatarFallback class="text-xs">{{ avatarInitial }}</AvatarFallback>
  </Avatar>
</template>