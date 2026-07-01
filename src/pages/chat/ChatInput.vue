<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import { Send, Square, ChevronDown, Bot, Sparkles, Brain, Diamond, Server, Settings, Globe, FileText } from 'lucide-vue-next'
import { useSettingsStore } from '@/stores/settingsStore'
import { usePromptStore } from '@/stores/promptStore'
import { useChatStore } from '@/stores/chatStore'
import { useMcpStore } from '@/stores/mcpStore'

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
const mcpStore = useMcpStore()

const input = ref(chatStore.activeConversationId ? chatStore.getDraft(chatStore.activeConversationId) : '')
const showModelMenu = ref(false)
const showPromptMenu = ref(false)
const showSearchMenu = ref(false)

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

// Search state
const searchEnabled = computed(() => chatStore.searchEnabled)
const searchEngine = computed(() => chatStore.searchEngine)

// Installed search MCP instances
const searchInstances = computed(() => {
  return mcpStore.instances.filter(i =>
    i.server_id === 'brave-search' || i.server_id === 'duckduckgo'
    || i.server_id === 'bocha-search' || i.server_id === 'local:bocha-search'
    || i.server_id === 'tavily-search' || i.server_id.contains('search')
  )
})

// Display name for current search engine
const searchEngineName = computed(() => {
  if (!searchEnabled.value) return null
  const engine = searchEngine.value
  if (!engine) return '搜索'
  const inst = mcpStore.instances.find(i => i.server_id === engine)
  return inst?.name ?? engine
})

async function selectSearchEngine(engine: string) {
  await chatStore.selectSearchEngine(engine)
  showSearchMenu.value = false
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
  if (conv.prompt_id === 'default') return { name: '默认', id: 'default' }
  if (conv.prompt_id && conv.prompt_id !== '') {
    const prompt = promptStore.prompts.find(p => p.id === conv.prompt_id)
    return prompt ? { name: prompt.name, id: prompt.id } : null
  }
  // No prompt selected — fall back to default prompt (same as backend behavior)
  const def = promptStore.defaultPrompt
  return def ? { name: '默认', id: 'default' } : null
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
  if (!target.closest('[data-model-menu]') && !target.closest('[data-prompt-menu]') && !target.closest('[data-search-menu]')) {
    showModelMenu.value = false
    showPromptMenu.value = false
    showSearchMenu.value = false
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

    <!-- Search Engine Switcher Dropdown -->
    <div v-if="showSearchMenu" class="absolute bottom-full left-3 z-50 mb-1 w-56 rounded-lg border bg-popover p-1 shadow-md" data-search-menu>
      <div class="max-h-64 overflow-y-auto">
        <div
          v-if="searchInstances.length === 0"
          class="px-2 py-1.5 text-xs text-muted-foreground italic"
        >
          无已安装的搜索引擎
        </div>
        <div
          v-for="inst in searchInstances"
          :key="inst.id"
          :class="cn(
            'flex cursor-pointer items-center gap-2 rounded-sm px-2 py-1.5 text-sm transition-colors hover:bg-foreground/5',
            searchEnabled && searchEngine === inst.server_id && 'bg-accent',
          )"
          @click="selectSearchEngine(inst.server_id)"
        >
          <Globe class="size-3 shrink-0" />
          <span>{{ inst.name }}</span>
          <span v-if="searchEnabled && searchEngine === inst.server_id" class="ml-auto text-xs text-muted-foreground">✓</span>
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

    <!-- Search + Prompt Switcher + Model Switcher -->
    <div class="mt-1.5 flex items-center gap-1.5">
      <button
        :class="cn(
          'inline-flex items-center gap-1 rounded-full px-2.5 py-0.5 text-xs border transition-all',
          searchEnabled
            ? 'border-foreground/30 bg-foreground/10 text-foreground font-medium'
            : 'border-transparent text-muted-foreground hover:bg-foreground/5 hover:border-border',
        )"
        @click="showSearchMenu = !showSearchMenu; showModelMenu = false; showPromptMenu = false"
      >
        <Globe class="size-3 shrink-0" />
        <span>{{ searchEnabled ? (searchEngineName ?? '搜索') : '搜索' }}</span>
        <ChevronDown class="size-2.5 shrink-0 opacity-60" />
      </button>
      <button
        :class="cn(
          'inline-flex items-center gap-1 rounded-full px-2.5 py-0.5 text-xs border transition-all max-w-28',
          currentPrompt?.id
            ? 'border-foreground/30 bg-foreground/10 text-foreground'
            : 'border-transparent text-muted-foreground hover:bg-foreground/5 hover:border-border',
        )"
        @click="showPromptMenu = !showPromptMenu; showModelMenu = false; showSearchMenu = false"
      >
        <FileText class="size-3 shrink-0" />
        <span class="truncate">{{ currentPrompt?.name ?? '提示词' }}</span>
        <ChevronDown class="size-2.5 shrink-0 opacity-60" />
      </button>
      <button
        :class="cn(
          'inline-flex items-center gap-1 rounded-full px-2.5 py-0.5 text-xs border transition-all max-w-36',
          currentModel?.model
            ? 'border-foreground/30 bg-foreground/10 text-foreground'
            : 'border-transparent text-muted-foreground hover:bg-foreground/5 hover:border-border',
        )"
        @click="showModelMenu = !showModelMenu; showPromptMenu = false; showSearchMenu = false"
      >
        <component :is="getIcon(currentModel?.provider?.icon)" class="size-3 shrink-0" />
        <span class="truncate">{{ currentModel?.model ?? '模型' }}</span>
        <ChevronDown class="size-2.5 shrink-0 opacity-60" />
      </button>
    </div>
  </div>
</template>
