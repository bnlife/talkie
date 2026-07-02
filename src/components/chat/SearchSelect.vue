<script setup lang="ts">
import { Globe } from 'lucide-vue-next'
import { Select, SelectContent, SelectItem, SelectLabel, SelectSeparator, SelectTrigger, SelectValue } from '@/components/ui/select'
import { useSearchSelect } from '@/composables/useSearchSelect'

const { searchInstances, searchValue, handleSearchChange } = useSearchSelect()
</script>

<template>
  <Select :model-value="searchValue" @update:model-value="handleSearchChange">
    <SelectTrigger variant="ghost" size="xs">
      <Globe class="size-3 shrink-0" />
      <SelectValue placeholder="搜索" />
    </SelectTrigger>
    <SelectContent side="top" :side-offset="4" class="w-64">
      <SelectItem value="__none__">
        无搜索
      </SelectItem>
      <SelectSeparator v-if="searchInstances.length > 0" />
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
</template>