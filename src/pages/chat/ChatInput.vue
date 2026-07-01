<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import { Send, Square, ChevronDown, Bot, Sparkles, Brain, Diamond, Server, Settings, Globe, FileText } from 'lucide-vue-next'
import { useSettingsStore } from '@/stores/settingsStore'
import { usePromptStore } from '@/stores/promptStore'
import { useChatStore } from '@/stores/chatStore'

const props = defineProps<{
  disabled?: boolean
  streaming?: boolean
}>()

const emit = defineEmits<{
  (e: 'send', content: string): void
  (e: 'stop-stream'): void
}>()

const settingsStore = useSettingsStore()
const promptStore = usePromptStore()
const chatStore = useChatStore()

const input = ref(chatStore.activeConversationId ? chatStore.getDraft(chatStore.activeConversationId) : '')
const showModelMenu = ref(false)
const showPromptMenu = ref(false)

// Restore draft when conversation changes
watch(() => chatStore.activeConversationId, (newId) => {
  if (newId) {
    input.value = chatStore.getDraft(newId)
  } else {
    input.value = ''
  }
})

// Save draft when input changes
watch(input, (val) => {
  if (chatStore.activeConversationId) {
    chatStore.setDraft(chatStore.activeConversationId, val)
  }
})

// Search toggle — per-conversation, persisted to DB
const searchEnabled = computed(() => chatStore.searchEnabled)
async function toggleSearch() {
  await chatStore.toggleSearch()
}

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

const currentPrompt = computed(() => {
  const conv = chatStore.activeConversation
  if (!conv) return null
  if (!conv.prompt_id || conv.prompt_id === '') return { name: '无', id: null }
  if (conv.prompt_id === 'default') return { name: '默认', id: 'default' }
  const prompt = promptStore.prompts.find(p => p.id === conv.prompt_id)
  return prompt ? { name: prompt.name, id: prompt.id } : { name: '无', id: null }
})

async function selectModel(providerId: string, model: string) {
  await chatStore.switchModel(providerId, model)
  showModelMenu.value = false
}

async function selectPrompt(promptId: string | null) {
  await chatStore.selectPrompt(promptId)
  showPromptMenu.value = false
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault()
    handleSend()
  }
}

function handleSend() {
  const text = input.value.trim()
  if (!text || props.disabled || props.streaming) return
  emit('send', text)
  input.value = ''
}

function handleOutsideClick(e: MouseEvent) {
  const target = e.target as HTMLElement
  if (!target.closest('[data-model-menu]') && !target.closest('[data-prompt-menu]')) {
    showModelMenu.value = false
    showPromptMenu.value = false
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

    <!-- Prompt Switcher Dropdown -->
    <div v-if="showPromptMenu" class="absolute bottom-full left-3 z-50 mb-1 w-64 rounded-lg border bg-popover p-1 shadow-md" data-prompt-menu>
      <div class="max-h-64 overflow-y-auto">
        <div
          :class="cn(
            'flex cursor-pointer items-center rounded-sm px-2 py-1.5 text-sm transition-colors hover:bg-foreground/5',
            currentPrompt?.id === null && 'bg-accent',
          )"
          @click="selectPrompt(null)"
        >
          无
        </div>
        <div
          :class="cn(
            'flex cursor-pointer items-center rounded-sm px-2 py-1.5 text-sm transition-colors hover:bg-foreground/5',
            currentPrompt?.id === 'default' && 'bg-accent',
          )"
          @click="selectPrompt('default')"
        >
          默认提示词
        </div>
        <div v-if="promptStore.prompts.length > 0" class="my-1 border-t" />
        <div
          v-for="prompt in promptStore.prompts"
          :key="prompt.id"
          :class="cn(
            'flex cursor-pointer items-center rounded-sm px-2 py-1.5 text-sm transition-colors hover:bg-foreground/5',
            currentPrompt?.id === prompt.id && 'bg-accent',
          )"
          @click="selectPrompt(prompt.id)"
        >
          {{ prompt.name }}
        </div>
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
          'min-h-[80px] max-h-[240px] resize-none flex-1 text-sm leading-relaxed',
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

    <!-- Search Toggle + Prompt Switcher + Model Switcher -->
    <div class="mt-1 flex items-center gap-1">
      <button
        :class="cn(
          'flex items-center gap-1 rounded px-1.5 py-0.5 text-xs transition-colors hover:bg-foreground/5',
          searchEnabled ? 'text-blue-500 bg-blue-500/10' : 'text-muted-foreground',
        )"
        @click="toggleSearch"
      >
        <Globe class="size-3" />
        <span>搜索</span>
      </button>
      <button
        class="flex items-center gap-1 rounded px-1.5 py-0.5 text-xs text-muted-foreground transition-colors hover:bg-foreground/5"
        @click="showPromptMenu = !showPromptMenu; showModelMenu = false"
      >
        <FileText class="size-3" />
        <span class="max-w-24 truncate">
          {{ currentPrompt?.name ?? '无' }}
        </span>
        <ChevronDown class="size-3" />
      </button>
      <button
        class="flex items-center gap-1 rounded px-1.5 py-0.5 text-xs text-muted-foreground transition-colors hover:bg-foreground/5"
        @click="showModelMenu = !showModelMenu; showPromptMenu = false"
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
