<script setup lang="ts">
import { ref, computed } from 'vue'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import { Send, Square, ChevronDown, Bot, Sparkles, Brain, Diamond, Server, Settings } from 'lucide-vue-next'
import { useSettingsStore } from '@/stores/settingsStore'
import { useChatStore } from '@/stores/chatStore'
import * as conversationBridge from '@/bridge/conversation'

const props = defineProps<{
  disabled?: boolean
  streaming?: boolean
}>()

const emit = defineEmits<{
  (e: 'send', content: string): void
  (e: 'stop-stream'): void
}>()

const settingsStore = useSettingsStore()
const chatStore = useChatStore()

const input = ref('')
const showModelMenu = ref(false)

const iconMap: Record<string, any> = { Bot, Sparkles, Brain, Diamond, Server, Settings }

function getIcon(icon?: string) {
  return iconMap[icon ?? ''] ?? Settings
}

const currentModel = computed(() => {
  const conv = chatStore.activeConversation
  if (!conv) return null
  const provider = settingsStore.providers.find(p => p.id === conv.provider_id)
  return { provider, model: conv.model }
})

async function selectModel(providerId: string, model: string) {
  const conv = chatStore.activeConversation
  if (!conv) return
  await conversationBridge.updateConversation(conv.id, conv.title)
  // Update conversation locally
  conv.provider_id = providerId
  conv.model = model
  // Persist via a dedicated update (we need to update the conversation's provider_id + model)
  // For now, we'll use the Rust update_conversation which only updates title
  // TODO: add update_conversation_model command or extend update_conversation
  showModelMenu.value = false
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault()
    handleSend()
  }
}

function handleSend() {
  const text = input.value.trim()
  if (!text || props.disabled) return
  emit('send', text)
  input.value = ''
}

// Close menu on outside click
function handleOutsideClick(e: MouseEvent) {
  const target = e.target as HTMLElement
  if (!target.closest('[data-model-menu]')) {
    showModelMenu.value = false
  }
}
</script>

<template>
  <div class="relative bg-background px-3 pt-1 pb-2">
    <!-- Model Switcher Dropdown -->
    <div v-if="showModelMenu" class="absolute bottom-full left-3 z-50 mb-1 w-72 rounded-lg border bg-popover p-1 shadow-md" data-model-menu>
      <div class="max-h-64 overflow-y-auto">
        <template v-for="provider in settingsStore.enabledProviders" :key="provider.id">
          <div class="flex items-center gap-2 px-2 py-1.5 text-xs font-medium text-muted-foreground">
            <component :is="getIcon(provider.icon)" class="size-3" />
            {{ provider.name }}
          </div>
          <div
            v-for="model in provider.models"
            :key="`${provider.id}-${model}`"
            :class="cn(
              'flex cursor-pointer items-center gap-2 rounded-sm px-6 py-1.5 text-sm transition-colors hover:bg-foreground/5',
              currentModel?.provider?.id === provider.id && currentModel?.model === model && 'bg-accent',
            )"
            @click="selectModel(provider.id, model)"
          >
            {{ model }}
          </div>
          <div v-if="provider.models.length === 0" class="px-6 py-1.5 text-xs text-muted-foreground italic">
            无模型
          </div>
        </template>
      </div>
    </div>

    <!-- Input Area -->
    <div :class="cn('flex items-end gap-2')">
      <Textarea
        v-model="input"
        :disabled="disabled"
        :rows="1"
        placeholder="输入消息..."
        :class="cn(
          'min-h-[40px] max-h-[120px] resize-none flex-1 text-sm leading-relaxed',
        )"
        @keydown="handleKeydown"
      />
      <Button
        v-if="streaming"
        variant="destructive"
        size="icon"
        class="h-10 w-10 shrink-0"
        @click="emit('stop-stream')"
      >
        <Square class="h-4 w-4" />
      </Button>
      <Button
        v-else
        size="icon"
        class="h-10 w-10 shrink-0"
        :disabled="disabled || !input.trim()"
        @click="handleSend"
      >
        <Send class="h-4 w-4" />
      </Button>
    </div>

    <!-- Model Switcher Button -->
    <div class="mt-1 flex items-center">
      <button
        class="flex items-center gap-1 rounded px-1.5 py-0.5 text-xs text-muted-foreground transition-colors hover:bg-foreground/5"
        @click="showModelMenu = !showModelMenu"
      >
        <component
          :is="getIcon(currentModel?.provider?.icon)"
          class="size-3"
        />
        <span class="max-w-32 truncate">
          {{ currentModel?.provider?.name ?? '未配置' }} / {{ currentModel?.model ?? '未知' }}
        </span>
        <ChevronDown class="size-3" />
      </button>
    </div>
  </div>
</template>
