import { computed } from 'vue'
import { useChatStore } from '@/stores/chatStore'
import { useMcpStore } from '@/stores/mcpStore'

export function useSearchSelect() {
  const chatStore = useChatStore()
  const mcpStore = useMcpStore()

  const searchEnabled = computed(() => chatStore.searchEnabled)
  const searchEngine = computed(() => chatStore.searchEngine)

  const searchInstances = computed(() => {
    return mcpStore.instances.filter(i =>
      i.server_id === 'brave-search' || i.server_id === 'duckduckgo'
      || i.server_id === 'bocha-search' || i.server_id === 'local:bocha-search'
      || i.server_id === 'tavily-search' || i.server_id.includes('search')
    )
  })

  const searchValue = computed(() => {
    if (!searchEnabled.value) return '__none__'
    return searchEngine.value || '__enabled__'
  })

  function handleSearchChange(value: unknown) {
    const str = String(value ?? '')
    if (str === '__none__') {
      chatStore.selectSearchEngine('')
    } else {
      chatStore.selectSearchEngine(str)
    }
  }

  return {
    searchEnabled,
    searchEngine,
    searchInstances,
    searchValue,
    handleSearchChange,
  }
}