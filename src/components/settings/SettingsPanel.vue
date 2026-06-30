<script setup lang="ts">
import { reactive, watch } from 'vue'
import type { Settings } from '../../types'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { Slider } from '@/components/ui/slider'
import { Label } from '@/components/ui/label'

const props = defineProps<{
  settings: Settings
}>()

const emit = defineEmits<{
  update: [settings: Partial<Settings>]
  'test-connection': []
}>()

const formState = reactive<Settings>({ ...props.settings })

watch(() => props.settings, (val) => {
  Object.assign(formState, val)
}, { deep: true })

function handleSave() {
  emit('update', { ...formState })
}
</script>

<template>
  <form class="flex flex-col gap-normal">
    <div class="flex items-center gap-normal">
      <Label class="w-25 flex-shrink-0 text-sub">API 地址</Label>
      <Input v-model="formState.base_url" placeholder="https://api.openai.com/v1" class="flex-1" />
    </div>
    <div class="flex items-center gap-normal">
      <Label class="w-25 flex-shrink-0 text-sub">API Key</Label>
      <Input v-model="formState.api_key" type="password" placeholder="sk-..." class="flex-1" />
    </div>
    <div class="flex items-center gap-normal">
      <Label class="w-25 flex-shrink-0 text-sub">模型</Label>
      <Input v-model="formState.model" placeholder="输入模型名称，如 deepseek-chat" class="flex-1" />
    </div>
    <div class="flex items-center gap-normal">
      <Label class="w-25 flex-shrink-0 text-sub">温度</Label>
      <Slider :min="0" :max="2" :step="0.1" :model-value="[formState.temperature]" @update:model-value="formState.temperature = ($event as number[])[0]" class="flex-1" />
      <span class="text-sub text-small w-8 text-right">{{ formState.temperature.toFixed(1) }}</span>
    </div>
    <div class="flex gap-normal">
      <Button variant="outline" @click="emit('test-connection')">测试连接</Button>
      <Button @click="handleSave">保存设置</Button>
    </div>
  </form>
</template>
