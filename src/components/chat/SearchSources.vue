<script setup lang="ts">
import { ref, computed } from 'vue'
import type { SearchResult } from '@/types'
import { Button } from '@/components/ui/button'
import { Globe, ChevronDown, ChevronUp } from 'lucide-vue-next'
import { openUrl } from '@/bridge/settings'
import { getDomain, getFaviconUrl } from '@/composables/useMessageRender'

const props = defineProps<{
  searchResults: SearchResult[]
}>()

const searchExpanded = ref(false)
const hasMoreResults = computed(() => props.searchResults.length > 3)
const visibleResults = computed(() => {
  if (searchExpanded.value || props.searchResults.length <= 3) return props.searchResults
  return props.searchResults.slice(0, 3)
})
</script>

<template>
  <div v-if="searchResults.length > 0" class="mt-2">
    <div class="flex flex-wrap gap-1.5">
      <a
        v-for="(result, idx) in visibleResults"
        :key="idx"
        href="#"
        class="inline-flex items-center gap-1.5 rounded-md bg-muted/40 px-2 py-1 text-xs text-muted-foreground transition-colors hover:bg-muted/70 max-w-48"
        @click.prevent="openUrl(result.url)"
      >
        <img
          :src="getFaviconUrl(result.url)"
          class="size-3 shrink-0 rounded-sm"
          @error="($event.target as HTMLImageElement).style.display='none'"
        />
        <span class="truncate">{{ result.title || getDomain(result.url) }}</span>
      </a>
    </div>
    <div class="mt-1 flex items-center gap-2">
      <span class="text-xs text-muted-foreground">
        <Globe class="size-3 inline -mt-0.5" />
        {{ searchResults.length }} 个来源
      </span>
      <Button v-if="hasMoreResults" variant="ghost" size="default" class="text-xs" @click="searchExpanded = !searchExpanded">
        <template v-if="searchExpanded">
          <ChevronUp class="size-3" /> 收起
        </template>
        <template v-else>
          展开更多
        </template>
      </Button>
    </div>
  </div>
</template>