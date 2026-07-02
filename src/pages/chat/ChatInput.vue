<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import { Send, Square, Paperclip, X } from 'lucide-vue-next'
import { useChatStore } from '@/stores/chatStore'
import { useAttachment } from '@/composables/useAttachment'
import SearchSelect from '@/components/chat/SearchSelect.vue'
import ModelSelect from '@/components/chat/ModelSelect.vue'
import PromptSelect from '@/components/chat/PromptSelect.vue'

const props = defineProps<{
  disabled?: boolean
  streaming?: boolean
}>()

const emit = defineEmits<{
  (e: 'send', displayContent: string, fullContent: string, attachments?: import('@/lib/attachment').AttachmentMeta[]): void
  (e: 'stop-stream'): void
}>()

const chatStore = useChatStore()

const {
  attachments, fileInputRef, justAdded,
  addFiles, removeAttachment, triggerFileInput, handleFileChange,
  handleDragOver, handleDrop, formatSize, canSend: hasAttachments,
  buildContent, clearAttachments,
} = useAttachment()

defineExpose({ addFiles })

const input = ref(chatStore.activeConversationId ? chatStore.getDraft(chatStore.activeConversationId) : '')

watch(() => chatStore.activeConversationId, (newId) => {
  if (newId) {
    input.value = chatStore.getDraft(newId)
  } else {
    input.value = ''
  }
  clearAttachments()
})

watch(input, (val) => {
  if (chatStore.activeConversationId) {
    chatStore.setDraft(chatStore.activeConversationId, val)
  }
})

const canSend = computed(() => {
  if (props.disabled || props.streaming) return false
  return input.value.trim().length > 0 || hasAttachments.value
})

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault()
    handleSend()
  }
}

async function handleSend() {
  if (!canSend.value) return

  const text = input.value.trim()
  const { displayContent, fullContent, metas } = await buildContent(text)

  emit('send', displayContent, fullContent, metas)
  input.value = ''
}
</script>

<template>
  <div class="relative bg-background px-3 pt-1 pb-2">
    <!-- Attachment List -->
    <div v-if="attachments.length > 0" class="mb-1.5 flex flex-wrap gap-1.5">
      <div
        v-for="(att, idx) in attachments"
        :key="`${att.name}-${att.size}`"
        :class="cn(
          'flex items-center gap-1 rounded-md bg-muted px-2 py-0.5 text-xs text-muted-foreground transition-all',
          justAdded.has(`${att.name}-${att.size}`) && 'ring-1 ring-foreground/20 bg-accent',
        )"
      >
        <span class="max-w-[140px] truncate">{{ att.name }}</span>
        <span class="text-2xs opacity-60">{{ formatSize(att.size) }}</span>
        <Button variant="ghost" size="icon" class="ml-0.5" @click="removeAttachment(idx)">
          <X class="size-3" />
        </Button>
      </div>
    </div>

    <!-- Input Area -->
    <div
      class="relative"
      @dragover="handleDragOver"
      @drop="handleDrop"
    >
      <input
        ref="fileInputRef"
        type="file"
        multiple
        class="hidden"
        @change="handleFileChange"
      />
      <Textarea
        v-model="input"
        :disabled="disabled"
        :rows="1"
        placeholder="输入消息..."
        :class="cn(
          'min-h-[80px] max-h-[240px] resize-none w-full text-sm leading-relaxed bg-muted/50 border-border/50 focus-visible:ring-1 focus-visible:ring-border pr-20',
        )"
        @keydown="handleKeydown"
      />
      <div class="absolute right-2 bottom-2 flex items-center gap-1">
        <Button
          variant="ghost"
          size="icon"
          class="hover:text-foreground"
          :disabled="disabled"
          @click="triggerFileInput"
        >
          <Paperclip class="h-4 w-4" />
        </Button>
        <Button
          v-if="streaming"
          variant="destructive"
          size="icon"
          @click="emit('stop-stream')"
        >
          <Square class="h-4 w-4" />
        </Button>
        <Button
          v-else
          size="icon"
          :disabled="!canSend"
          @click="handleSend"
        >
          <Send class="h-4 w-4" />
        </Button>
      </div>
    </div>

    <!-- Search + Prompt Switcher + Model Switcher -->
    <div class="mt-1.5 flex items-center gap-1.5">
      <SearchSelect />
      <PromptSelect />
      <ModelSelect />
    </div>
  </div>
</template>