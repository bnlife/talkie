<script setup lang="ts">
import { reactive, watch } from 'vue'
import type { Settings } from '@/types'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Separator } from '@/components/ui/separator'
import { Slider } from '@/components/ui/slider'
import { Globe, Cpu, Plug, Save } from 'lucide-vue-next'

const props = defineProps<{
  settings: Settings
}>()

const emit = defineEmits<{
  (e: 'update', value: Partial<Settings>): void
  (e: 'test-connection'): void
}>()

const form = reactive({
  base_url: props.settings.base_url,
  api_key: props.settings.api_key,
  model: props.settings.model,
  temperature: props.settings.temperature,
  darkMode: props.settings.darkMode ?? false,
})

watch(
  () => props.settings,
  (val) => {
    form.base_url = val.base_url
    form.api_key = val.api_key
    form.model = val.model
    form.temperature = val.temperature
    form.darkMode = val.darkMode ?? false
  },
  { deep: true },
)

function handleSave() {
  emit('update', {
    base_url: form.base_url,
    api_key: form.api_key,
    model: form.model,
    temperature: form.temperature,
    darkMode: form.darkMode,
  })
}

function handleTestConnection() {
  emit('test-connection')
}
</script>

<template>
  <form
    :class="cn('flex flex-col gap-4')"
    @submit.prevent="handleSave"
  >
    <!-- 连接设置 -->
    <div :class="cn('flex flex-col gap-3')">
      <div :class="cn('flex items-center gap-2 text-sm font-medium text-muted-foreground')">
        <Globe :class="cn('size-4')" />
        <span>连接设置</span>
      </div>

      <div :class="cn('flex items-center gap-3')">
        <Label for="base-url" :class="cn('w-20 shrink-0 text-right text-sm')">
          Base URL
        </Label>
        <Input
          id="base-url"
          v-model="form.base_url"
          placeholder="https://api.openai.com/v1"
          :class="cn('flex-1')"
        />
      </div>

      <div :class="cn('flex items-center gap-3')">
        <Label for="api-key" :class="cn('w-20 shrink-0 text-right text-sm')">
          API Key
        </Label>
        <Input
          id="api-key"
          v-model="form.api_key"
          type="password"
          placeholder="sk-..."
          :class="cn('flex-1')"
        />
      </div>
    </div>

    <Separator />

    <!-- 模型设置 -->
    <div :class="cn('flex flex-col gap-3')">
      <div :class="cn('flex items-center gap-2 text-sm font-medium text-muted-foreground')">
        <Cpu :class="cn('size-4')" />
        <span>模型设置</span>
      </div>

      <div :class="cn('flex items-center gap-3')">
        <Label for="model" :class="cn('w-20 shrink-0 text-right text-sm')">
          模型
        </Label>
        <Input
          id="model"
          v-model="form.model"
          placeholder="gpt-3.5-turbo"
          :class="cn('flex-1')"
        />
      </div>

      <div :class="cn('flex items-center gap-3')">
        <Label for="temperature" :class="cn('w-20 shrink-0 text-right text-sm')">
          温度
        </Label>
        <div :class="cn('flex flex-1 items-center gap-3')">
          <Slider
            id="temperature"
            :model-value="[form.temperature]"
            :min="0"
            :max="2"
            :step="0.1"
            :class="cn('flex-1')"
            @update:model-value="(v) => { if (v) form.temperature = v[0] ?? form.temperature }"
          />
          <span :class="cn('w-10 text-right text-sm tabular-nums')">
            {{ form.temperature.toFixed(1) }}
          </span>
        </div>
      </div>
    </div>

    <Separator />

    <!-- 夜间模式 -->
    <div :class="cn('flex flex-col gap-3')">
      <div :class="cn('flex items-center justify-between')">
        <span :class="cn('text-sm')">夜间模式</span>
        <button
          type="button"
          role="switch"
          :aria-checked="form.darkMode"
          :class="cn(
            'peer inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full',
            'border-2 border-transparent transition-colors',
            'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2',
            form.darkMode ? 'bg-primary' : 'bg-input',
          )"
          @click="form.darkMode = !form.darkMode"
        >
          <span
            :class="cn(
              'pointer-events-none block h-5 w-5 rounded-full bg-background shadow-lg',
              'transition-transform',
              form.darkMode ? 'translate-x-5' : 'translate-x-0',
            )"
          />
        </button>
      </div>
    </div>

    <Separator />

    <!-- 操作按钮 -->
    <div :class="cn('flex flex-col gap-3')">
      <div :class="cn('flex items-center gap-2 text-sm font-medium text-muted-foreground')">
        <Plug :class="cn('size-4')" />
        <span>操作</span>
      </div>

      <div :class="cn('flex items-center justify-end gap-2')">
        <Button
          type="button"
          variant="outline"
          size="sm"
          @click="handleTestConnection"
        >
          <Plug :class="cn('size-4')" />
          测试连接
        </Button>
        <Button
          type="submit"
          size="sm"
        >
          <Save :class="cn('size-4')" />
          保存设置
        </Button>
      </div>
    </div>
  </form>
</template>
