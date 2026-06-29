<script setup lang="ts">
import { reactive, watch } from 'vue'
import type { Settings } from '../../types'

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
  <n-form label-placement="left" label-width="100px">
    <n-form-item label="API 地址" path="base_url">
      <n-input
        v-model:value="formState.base_url"
        placeholder="https://api.openai.com/v1"
        size="small"
      />
    </n-form-item>
    <n-form-item label="API Key" path="api_key">
      <n-input
        v-model:value="formState.api_key"
        type="password"
        placeholder="sk-..."
        show-password-on="click"
        size="small"
      />
    </n-form-item>
    <n-form-item label="模型" path="model">
      <n-input
        v-model:value="formState.model"
        placeholder="输入模型名称，如 deepseek-chat"
        size="small"
      />
    </n-form-item>
    <n-form-item label="温度" path="temperature">
      <n-slider
        v-model:value="formState.temperature"
        :min="0"
        :max="2"
        :step="0.1"
        style="width: 100%;"
      />
      <n-text depth="3" style="margin-left: 12px; min-width: 32px;">
        {{ formState.temperature.toFixed(1) }}
      </n-text>
    </n-form-item>
    <n-form-item>
      <n-space :size="8">
        <n-button size="small" @click="emit('test-connection')">
          测试连接
        </n-button>
        <n-button size="small" type="primary" @click="handleSave">
          保存设置
        </n-button>
      </n-space>
    </n-form-item>
  </n-form>
</template>
