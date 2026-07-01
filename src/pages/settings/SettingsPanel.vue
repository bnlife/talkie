<script setup lang="ts">
import { reactive, watch, ref } from 'vue'
import type { ModelProvider } from '@/types'
import { useSettingsStore } from '@/stores/settingsStore'
import { cn } from '@/lib/utils'
import { toast } from 'vue-sonner'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Separator } from '@/components/ui/separator'
import {
  Eye, EyeOff, RefreshCw, Plus, Check, XIcon,
} from 'lucide-vue-next'

const props = defineProps<{
  provider: ModelProvider
}>()

const settingsStore = useSettingsStore()

const form = reactive({
  name: props.provider.name,
  base_url: props.provider.base_url,
  api_key: props.provider.api_key,
  enabled: props.provider.enabled,
  temperature: settingsStore.temperature,
  top_p: settingsStore.top_p,
})

const showApiKey = ref(false)
const testResult = ref<{ ok: boolean; error?: string } | null>(null)
const isTesting = ref(false)
const isFetching = ref(false)
const newModelName = ref('')

watch(() => props.provider, (p) => {
  form.name = p.name
  form.base_url = p.base_url
  form.api_key = p.api_key
  form.enabled = p.enabled
}, { deep: true })

async function saveName() {
  await settingsStore.updateProvider(props.provider.id, { name: form.name })
}

async function saveBaseUrl() {
  await settingsStore.updateProvider(props.provider.id, { base_url: form.base_url })
}

async function saveApiKey() {
  await settingsStore.updateProvider(props.provider.id, { api_key: form.api_key })
}

async function toggleEnabled() {
  form.enabled = !form.enabled
  await settingsStore.updateProvider(props.provider.id, { enabled: form.enabled })
}

async function handleTest() {
  if (!form.base_url.trim()) { toast.warning('请先填写 API 地址'); return }
  if (!form.api_key.trim()) { toast.warning('请先填写 API Key'); return }
  isTesting.value = true
  testResult.value = null
  await settingsStore.updateProvider(props.provider.id, {
    base_url: form.base_url,
    api_key: form.api_key,
  })
  // Auto-fetch models to get a real model name for testing
  isFetching.value = true
  await settingsStore.fetchModels(props.provider.id)
  isFetching.value = false
  testResult.value = await settingsStore.testConnection(props.provider.id)
  isTesting.value = false
}

async function handleFetchModels() {
  if (!form.base_url.trim()) { toast.warning('请先填写 API 地址'); return }
  if (!form.api_key.trim()) { toast.warning('请先填写 API Key'); return }
  isFetching.value = true
  await settingsStore.fetchModels(props.provider.id)
  isFetching.value = false
}

function handleAddModel() {
  if (newModelName.value.trim()) {
    settingsStore.addModel(props.provider.id, newModelName.value.trim())
    newModelName.value = ''
  }
}

async function handleParamChange() {
  settingsStore.temperature = form.temperature
  settingsStore.top_p = form.top_p
  await settingsStore.saveSettings()
}
</script>

<template>
  <div :class="cn('flex flex-col gap-1.5 p-3 text-sm')">
    <!-- 1. Provider 标题栏 -->
    <div :class="cn('flex items-center justify-between')">
      <Input
        v-model="form.name"
        class="h-6 w-40 border-none bg-transparent px-1 text-sm font-medium shadow-none focus-visible:ring-0"
        @blur="saveName"
        @keyup.enter="saveName"
      />
      <div class="flex items-center gap-1.5">
        <span class="text-xs text-muted-foreground">{{ form.enabled ? '已启用' : '已禁用' }}</span>
        <button
          type="button"
          role="switch"
          :aria-checked="form.enabled"
          :class="cn(
            'peer inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full',
            'border-2 border-transparent transition-colors',
            form.enabled ? 'bg-primary' : 'bg-input',
          )"
          @click="toggleEnabled"
        >
          <span
            :class="cn(
              'pointer-events-none block h-4 w-4 rounded-full bg-background shadow-lg transition-transform',
              form.enabled ? 'translate-x-4' : 'translate-x-0',
            )"
          />
        </button>
      </div>
    </div>

    <Separator />

    <!-- 2. API 密钥 -->
    <div :class="cn('flex flex-col gap-1')">
      <span :class="cn('text-xs font-medium text-foreground')">API 密钥</span>
      <div class="flex items-center gap-1.5">
        <div class="relative flex-1">
          <Input
            v-model="form.api_key"
            :type="showApiKey ? 'text' : 'password'"
            placeholder="sk-..."
            class="h-7 pr-16 text-sm"
            @blur="saveApiKey"
          />
          <div class="absolute right-1 top-1/2 flex -translate-y-1/2 items-center gap-0.5">
            <Check v-if="testResult?.ok" class="size-3.5 text-green-600" />
            <XIcon v-else-if="testResult && !testResult.ok" class="size-3.5 text-destructive" />
            <Button
              variant="ghost"
              size="icon-sm"
              class="size-5"
              @click="showApiKey = !showApiKey"
            >
              <Eye v-if="!showApiKey" class="size-3" />
              <EyeOff v-else class="size-3" />
            </Button>
          </div>
        </div>
        <Button
          :variant="testResult?.ok ? 'default' : testResult && !testResult.ok ? 'destructive' : 'outline'"
          size="sm"
          class="h-7"
          :disabled="isTesting"
          @click="handleTest"
        >
          {{ isTesting ? '检测中...' : '检测' }}
        </Button>
      </div>
      <div :class="cn('h-4 text-xs', testResult?.ok ? 'text-green-600' : 'text-destructive')">
        <span v-if="testResult">{{ testResult.ok ? '连接成功' : testResult.error }}</span>
      </div>
    </div>

    <Separator />

    <!-- 3. API 地址 -->
    <div :class="cn('flex flex-col gap-1')">
      <span :class="cn('text-xs font-medium text-foreground')">API 地址</span>
      <Input
        v-model="form.base_url"
        placeholder="https://api.openai.com/v1"
        class="h-7 text-sm"
        @blur="saveBaseUrl"
      />
      <span class="text-xs text-muted-foreground">
        预览: {{ form.base_url.replace(/\/+$/, '') }}/chat/completions
      </span>
    </div>

    <Separator />

    <!-- 4. 参数设置 -->
    <div :class="cn('flex flex-col gap-1')">
      <span :class="cn('text-xs font-medium text-foreground')">参数设置</span>
      <div class="flex items-center gap-2">
        <span class="w-10 text-right text-xs text-muted-foreground">温度</span>
        <Input
          v-model.number="form.temperature"
          type="number"
          :min="0"
          :max="2"
          :step="0.1"
          class="h-7 w-20 text-sm"
          @blur="handleParamChange"
        />
        <span class="w-10 text-right text-xs text-muted-foreground">Top-P</span>
        <Input
          v-model.number="form.top_p"
          type="number"
          :min="0"
          :max="1"
          :step="0.05"
          class="h-7 w-20 text-sm"
          @blur="handleParamChange"
        />
      </div>
    </div>

    <Separator />

    <!-- 5. 模型列表 -->
    <div :class="cn('flex flex-col gap-1')">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-1.5">
          <span :class="cn('text-xs font-medium text-foreground')">模型</span>
          <span :class="cn('rounded-full bg-muted px-1 py-0.5 text-xs text-muted-foreground')">
            {{ provider.models.length }}
          </span>
        </div>
        <Button variant="ghost" size="icon-sm" class="size-5" :disabled="isFetching" @click="handleFetchModels">
          <RefreshCw :class="cn('size-3', isFetching && 'animate-spin')" />
        </Button>
      </div>

      <!-- 添加模型 -->
      <div class="flex items-center gap-1.5">
        <Input
          v-model="newModelName"
          placeholder="输入模型名称..."
          class="h-7 text-sm"
          @keyup.enter="handleAddModel"
        />
        <Button variant="outline" size="sm" class="h-7" @click="handleAddModel">
          <Plus class="size-3" />
          添加
        </Button>
      </div>

      <!-- 模型列表 -->
      <div class="flex flex-col gap-0.5">
        <div
          v-for="model in provider.models"
          :key="model"
          class="rounded-sm px-1.5 py-1 hover:bg-foreground/5"
        >
          <span class="text-sm">{{ model }}</span>
        </div>
        <div v-if="provider.models.length === 0" class="py-2 text-center text-xs text-muted-foreground">
          暂无模型，点击"获取"拉取或手动添加
        </div>
      </div>
    </div>
  </div>
</template>
