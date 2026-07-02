<script setup lang="ts">
import { Select, SelectContent, SelectItem, SelectLabel, SelectSeparator, SelectTrigger, SelectValue } from '@/components/ui/select'
import { useModelSelect } from '@/composables/useModelSelect'
import { useSettingsStore } from '@/stores/settingsStore'

const settingsStore = useSettingsStore()
const { getIcon, currentModel, modelValue, handleModelChange } = useModelSelect()
</script>

<template>
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
</template>