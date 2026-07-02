<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import { Send, Square, ChevronDown, Bot, Sparkles, Brain, Diamond, Server, Settings, Globe, FileText, Paperclip, X } from 'lucide-vue-next'
import { useSettingsStore } from '@/stores/settingsStore'
import { usePromptStore } from '@/stores/promptStore'
import { useChatStore } from '@/stores/chatStore'
import { useMcpStore } from '@/stores/mcpStore'
import { useAttachment } from '@/composables/useAttachment'

const props = defineProps<{
  disabled?: boolean
  streaming?: boolean
}>()

const emit = defineEmits<{
  (e: 'send', displayContent: string, fullContent: string, attachments?: import('@/lib/attachment').AttachmentMeta[]): void
  (e: 'stop-stream'): void
}>()

const settingsStore = useSettingsStore()
const promptStore = usePromptStore()
const chatStore = useChatStore()
const mcpStore = useMcpStore()

const {
  attachments, fileInputRef, justAdded,
  addFiles, removeAttachment, triggerFileInput, handleFileChange,
  handleDragOver, handleDrop, formatSize, canSend: hasAttachments,
  buildContent, clearAttachments,
} = useAttachment()

defineExpose({ addFiles })

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
  clearAttachments()
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
    || i.server_id === 'tavily-search' || i.server_id.includes('search')
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

// --- Send logic ---

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

function handleOutsideClick(e: MouseEvent) {
  const target = e.target as HTMLElement
  if (!target.closest('[data-model-menu]') && !target.closest('[data-prompt-menu]') && !target.closest('[data-search-menu]') && !target.closest('[data-menu-trigger]')) {
    showModelMenu.value = false
    showPromptMenu.value = false
    showSearchMenu.value = false
  }
}

onMounted(() => {
  document.addEventListener('click', handleOutsideClick, true)
})

onUnmounted(() => {
  document.removeEventListener('click', handleOutsideClick, true)
})
</script>

<template>
  <div class="relative bg-background px-3 pt-1 pb-2">
    <!-- Model Switcher Dropdown -->
    <div v-if="showModelMenu" class="absolute bottom-full left-3 z-50 mb-1 w-72 rounded-xl bg-popover p-1 shadow-lg" data-model-menu>
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
              'flex cursor-pointer items-center gap-2 rounded-sm px-6 py-1.5 text-sm transition-colors hover:bg-hover',
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
    <div v-if="showPromptMenu" class="absolute bottom-full left-3 z-50 mb-1 w-64 rounded-xl bg-popover p-1 shadow-lg" data-prompt-menu>
      <div class="max-h-64 overflow-y-auto">
        <div
          :class="cn(
            'flex cursor-pointer items-center rounded-sm px-2 py-1.5 text-sm transition-colors hover:bg-hover',
            currentPrompt?.id === null && 'bg-accent',
          )"
          @click="selectPrompt(null)"
        >
          无
        </div>
        <div
          :class="cn(
            'flex cursor-pointer items-center rounded-sm px-2 py-1.5 text-sm transition-colors hover:bg-hover',
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
            'flex cursor-pointer items-center rounded-sm px-2 py-1.5 text-sm transition-colors hover:bg-hover',
            currentPrompt?.id === prompt.id && 'bg-accent',
          )"
          @click="selectPrompt(prompt.id)"
        >
          {{ prompt.name }}
        </div>
      </div>
    </div>

    <!-- Search Engine Switcher Dropdown -->
    <div v-if="showSearchMenu" class="absolute bottom-full left-3 z-50 mb-1 w-56 rounded-xl bg-popover p-1 shadow-lg" data-search-menu>
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
            'flex cursor-pointer items-center gap-2 rounded-sm px-2 py-1.5 text-sm transition-colors hover:bg-hover',
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
        <button
          class="ml-0.5 rounded-sm p-0.5 hover:bg-hover-strong"
          @click="removeAttachment(idx)"
        >
          <X class="size-3" />
        </button>
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
          class="text-muted-foreground hover:text-foreground"
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
      <Button
        variant="ghost"
        size="default"
        :class="cn(
          'h-6 gap-1 px-2.5 text-xs',
          searchEnabled
            ? 'bg-muted text-foreground font-medium'
            : 'text-muted-foreground',
        )"
        data-menu-trigger
        @click.stop="showSearchMenu = !showSearchMenu; showModelMenu = false; showPromptMenu = false"
      >
        <Globe class="size-3 shrink-0" />
        <span>{{ searchEnabled ? (searchEngineName ?? '搜索') : '搜索' }}</span>
        <ChevronDown class="size-2.5 shrink-0 opacity-60" />
      </Button>
      <Button
        variant="ghost"
        size="default"
        :class="cn(
          'h-6 max-w-28 gap-1 px-2.5 text-xs',
          currentPrompt?.id
            ? 'bg-muted text-foreground'
            : 'text-muted-foreground',
        )"
        data-menu-trigger
        @click.stop="showPromptMenu = !showPromptMenu; showModelMenu = false; showSearchMenu = false"
      >
        <FileText class="size-3 shrink-0" />
        <span class="truncate">{{ currentPrompt?.name ?? '提示词' }}</span>
        <ChevronDown class="size-2.5 shrink-0 opacity-60" />
      </Button>
      <Button
        variant="ghost"
        size="default"
        :class="cn(
          'h-6 max-w-36 gap-1 px-2.5 text-xs',
          currentModel?.model
            ? 'bg-muted text-foreground'
            : 'text-muted-foreground',
        )"
        data-menu-trigger
        @click.stop="showModelMenu = !showModelMenu; showPromptMenu = false; showSearchMenu = false"
      >
        <component :is="getIcon(currentModel?.provider?.icon)" class="size-3 shrink-0" />
        <span class="truncate">{{ currentModel?.model ?? '模型' }}</span>
        <ChevronDown class="size-2.5 shrink-0 opacity-60" />
      </Button>
    </div>
  </div>
</template>
