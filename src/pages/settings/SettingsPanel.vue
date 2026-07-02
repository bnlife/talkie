<script setup lang="ts">
import { reactive, watch, ref } from 'vue'
import type { ModelProvider } from '@/types'
import { useSettingsStore } from '@/stores/settingsStore'
import { log } from '@/bridge/log'
import { cn } from '@/lib/utils'
import { toast } from 'vue-sonner'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Separator } from '@/components/ui/separator'
import { Switch } from '@/components/ui/switch'
import {
  Eye, EyeOff, RefreshCw, Plus, Check, XIcon,
} from 'lucide-vue-next'
import ModelAddDialog from './ModelAddDialog.vue'

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
const showAddDialog = ref(false)

watch(() => props.provider, (p) => {
  form.name = p.name
  form.base_url = p.base_url
  form.api_key = p.api_key
  form.enabled = p.enabled
}, { deep: true })

watch(() => form.enabled, async (newValue) => {
  await log('info', `FE::SettingsPanel | enabled changed | id=${props.provider.id} value=${newValue}`)
  await settingsStore.updateProvider(props.provider.id, { enabled: newValue })
})

async function saveName() {
  await settingsStore.updateProvider(props.provider.id, { name: form.name })
}

async function saveBaseUrl() {
  await settingsStore.updateProvider(props.provider.id, { base_url: form.base_url })
}

async function saveApiKey() {
  await settingsStore.updateProvider(props.provider.id, { api_key: form.api_key })
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

function handleAddModelConfirm(modelName: string) {
  settingsStore.addModel(props.provider.id, modelName)
  showAddDialog.value = false
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
        size="bare"
        class="w-40 font-medium"
        @blur="saveName"
        @keyup.enter="saveName"
      />
      <div class="flex items-center gap-1.5">
        <span class="text-xs text-muted-foreground">{{ form.enabled ? '已启用' : '已禁用' }}</span>
        <Switch v-model="form.enabled" />
      </div>
    </div>

    <Separator />

    <!-- 2. API 密钥 -->
    <div :class="cn('flex flex-col gap-1')">
      <span :class="cn('text-xs font-medium text-foreground')">API 密钥</span>
      <div class="flex items-center border rounded-md h-8">
        <div class="relative flex-1">
          <Input
            v-model="form.api_key"
            :type="showApiKey ? 'text' : 'password'"
            placeholder="sk-..."
            size="inline"
            class="pr-16"
            @blur="saveApiKey"
          />
          <div class="absolute right-1 top-1/2 flex -translate-y-1/2 items-center gap-0.5">
            <Check v-if="testResult?.ok" class="size-3.5 text-success" />
            <XIcon v-else-if="testResult && !testResult.ok" class="size-3.5 text-error" />
            <Button
              variant="ghost"
              size="icon"
              @click="showApiKey = !showApiKey"
            >
              <Eye v-if="!showApiKey" class="size-3" />
              <EyeOff v-else class="size-3" />
            </Button>
          </div>
        </div>
        <div class="border-l h-7" />
        <Button
          size="default"
          :variant="testResult?.ok ? 'default' : testResult && !testResult.ok ? 'destructive' : 'ghost'"
          class="h-full rounded-none px-3"
          :disabled="isTesting"
          @click="handleTest"
        >
          {{ isTesting ? '检测中...' : '检测' }}
        </Button>
      </div>
      <div :class="cn('h-4 text-xs', testResult?.ok ? 'text-success' : 'text-error')">
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
          size="sm"
          class="w-20"
          @blur="handleParamChange"
        />
        <span class="w-10 text-right text-xs text-muted-foreground">Top-P</span>
        <Input
          v-model.number="form.top_p"
          type="number"
          :min="0"
          :max="1"
          :step="0.05"
          size="sm"
          class="w-20"
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
        <div class="flex items-center gap-1">
          <Button variant="ghost" size="icon" @click="showAddDialog = true">
            <Plus class="size-3" />
          </Button>
          <Button variant="ghost" size="icon" :disabled="isFetching" @click="handleFetchModels">
            <RefreshCw :class="cn('size-3', isFetching && 'animate-spin')" />
          </Button>
        </div>
      </div>

      <!-- 模型列表 -->
      <div class="flex flex-col gap-0.5">
        <div
          v-for="model in provider.models"
          :key="model"
          class="rounded-sm px-1.5 py-1 hover:bg-hover"
        >
          <span class="text-sm">{{ model }}</span>
        </div>
        <div v-if="provider.models.length === 0" class="py-2 text-center text-xs text-muted-foreground">
          暂无模型，点击"获取"拉取或手动添加
        </div>
      </div>
    </div>

    <ModelAddDialog
      :visible="showAddDialog"
      :provider="provider"
      @confirm="handleAddModelConfirm"
      @cancel="showAddDialog = false"
    />
  </div>
</template>
