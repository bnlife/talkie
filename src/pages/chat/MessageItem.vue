<script setup lang="ts">
import { ref, computed } from 'vue'
import type { Message } from '@/types'
import { cn } from '@/lib/utils'
import { Card, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Copy, Trash2, RefreshCw, Check } from 'lucide-vue-next'

const props = defineProps<{
  message: Message
  streaming?: boolean
  isLast?: boolean
}>()

const emit = defineEmits<{
  copy: [content: string]
  delete: [messageId: string]
  regenerate: []
}>()

const isUser = computed(() => props.message.role === 'user')
const isCopied = ref(false)

async function handleCopy() {
  await navigator.clipboard.writeText(props.message.content)
  isCopied.value = true
  setTimeout(() => { isCopied.value = false }, 2000)
  emit('copy', props.message.content)
}

</script>

<template>
  <div
    :class="cn(
      'group relative flex w-full',
      isUser ? 'justify-end' : 'justify-start'
    )"
  >
    <div class="relative">
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

      <!-- 操作按钮 -->
      <div
        v-if="!streaming"
        :class="cn(
          'absolute -bottom-8 flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity',
          isUser ? 'right-0' : 'left-0'
        )"
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
</template>
