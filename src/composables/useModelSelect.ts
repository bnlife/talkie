import { computed } from 'vue'
import { Bot, Sparkles, Brain, Diamond, Server, Settings } from 'lucide-vue-next'
import { useSettingsStore } from '@/stores/settingsStore'
import { useChatStore } from '@/stores/chatStore'

export function useModelSelect() {
  const settingsStore = useSettingsStore()
  const chatStore = useChatStore()

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

  const modelValue = computed(() => {
    if (!currentModel.value?.model) return ''
    return `${currentModel.value.provider?.id}::${currentModel.value.model}`
  })

  async function selectModel(providerId: string, model: string) {
    await chatStore.switchModel(providerId, model)
  }

  function handleModelChange(value: unknown) {
    if (!value) return
    const str = String(value)
    const [providerId, ...modelParts] = str.split('::')
    const model = modelParts.join('::')
    selectModel(providerId, model)
  }

  return {
    iconMap,
    getIcon,
    currentModel,
    modelValue,
    selectModel,
    handleModelChange,
  }
}