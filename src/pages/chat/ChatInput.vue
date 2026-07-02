<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import { Select, SelectContent, SelectItem, SelectLabel, SelectSeparator, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Send, Square, Bot, Sparkles, Brain, Diamond, Server, Settings, Globe, FileText, Paperclip, X } from 'lucide-vue-next'
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

// Search select value (empty string when disabled)
const searchValue = computed(() => {
  if (!searchEnabled.value) return ''
  return searchEngine.value || '__enabled__'
})

function handleSearchChange(value: unknown) {
  chatStore.selectSearchEngine(String(value ?? ''))
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
}

// Model select value
const modelValue = computed(() => {
  if (!currentModel.value?.model) return ''
  return `${currentModel.value.provider?.id}::${currentModel.value.model}`
})

function handleModelChange(value: unknown) {
  if (!value) return
  const str = String(value)
  const [providerId, ...modelParts] = str.split('::')
  const model = modelParts.join('::')
  selectModel(providerId, model)
}

async function selectPrompt(promptId: string | null) {
  await chatStore.selectPrompt(promptId)
}

// Prompt select value
const promptValue = computed(() => {
  if (!currentPrompt.value?.id) return '__none__'
  return currentPrompt.value.id
})

function handlePromptChange(value: unknown) {
  const str = value ? String(value) : null
  selectPrompt(str === '__none__' ? null : str)
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
      <Select :model-value="searchValue" @update:model-value="handleSearchChange">
        <SelectTrigger variant="ghost" size="xs">
          <Globe class="size-3 shrink-0" />
          <SelectValue placeholder="搜索" />
        </SelectTrigger>
        <SelectContent side="top" :side-offset="4" class="w-64">
          <div
            v-if="searchInstances.length === 0"
            class="px-2 py-1.5 text-xs text-muted-foreground italic"
          >
            无已安装的搜索引擎
          </div>
          <SelectItem
            v-for="inst in searchInstances"
            :key="inst.id"
            :value="inst.server_id"
          >
            {{ inst.name }}
          </SelectItem>
        </SelectContent>
      </Select>

      <Select :model-value="promptValue" @update:model-value="handlePromptChange">
        <SelectTrigger variant="ghost" size="xs">
          <FileText class="size-3 shrink-0" />
          <SelectValue placeholder="提示词" />
        </SelectTrigger>
        <SelectContent side="top" :side-offset="4" class="w-64">
          <SelectItem value="__none__">
            无
          </SelectItem>
          <SelectSeparator v-if="promptStore.prompts.length > 0" />
          <SelectItem
            v-for="prompt in promptStore.prompts"
            :key="prompt.id"
            :value="prompt.id"
          >
            {{ prompt.name }}
          </SelectItem>
        </SelectContent>
      </Select>

      <Select :model-value="modelValue" @update:model-value="handleModelChange">
        <SelectTrigger variant="ghost" size="xs">
          <component :is="getIcon(currentModel?.provider?.icon)" class="size-3 shrink-0" />
          <SelectValue placeholder="模型" />
        </SelectTrigger>
        <SelectContent side="top" :side-offset="4" class="w-64">
          <template v-for="provider in settingsStore.enabledProviders" :key="provider.id">
            <SelectLabel class="flex items-center gap-2 text-xs font-medium text-muted-foreground">
              <component :is="getIcon(provider.icon)" class="size-3" />
              {{ provider.name }}
            </SelectLabel>
            <SelectItem
              v-for="model in provider.models"
              :key="`${provider.id}-${model}`"
              :value="`${provider.id}::${model}`"
              class="pl-6"
            >
              {{ model }}
            </SelectItem>
            <div v-if="provider.models.length === 0" class="px-2 py-1.5 text-xs text-muted-foreground italic">
              无模型
            </div>
            <SelectSeparator v-if="provider.models.length > 0" />
          </template>
        </SelectContent>
      </Select>
    </div>
  </div>
</template>
