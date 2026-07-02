import { computed } from 'vue'
import { useChatStore } from '@/stores/chatStore'
import { usePromptStore } from '@/stores/promptStore'

export function usePromptSelect() {
  const chatStore = useChatStore()
  const promptStore = usePromptStore()

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

  const promptValue = computed(() => {
    if (!currentPrompt.value?.id) return '__none__'
    return currentPrompt.value.id
  })

  async function selectPrompt(promptId: string | null) {
    await chatStore.selectPrompt(promptId)
  }

  function handlePromptChange(value: unknown) {
    const str = value ? String(value) : null
    selectPrompt(str === '__none__' ? null : str)
  }

  return {
    currentPrompt,
    promptValue,
    selectPrompt,
    handlePromptChange,
  }
}