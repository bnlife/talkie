<script setup lang="ts">
import { FileText } from 'lucide-vue-next'
import { Select, SelectContent, SelectItem, SelectSeparator, SelectTrigger, SelectValue } from '@/components/ui/select'
import { usePromptSelect } from '@/composables/usePromptSelect'
import { usePromptStore } from '@/stores/promptStore'

const promptStore = usePromptStore()
const { promptValue, handlePromptChange } = usePromptSelect()
</script>

<template>
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
</template>