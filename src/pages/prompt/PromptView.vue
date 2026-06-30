<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { usePromptStore } from '@/stores/promptStore'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Minus, Maximize2, Minimize2, X, Plus, Trash2, Star } from 'lucide-vue-next'

const promptStore = usePromptStore()
const appWindow = getCurrentWindow()
const isMaximized = ref(false)

const editingId = ref<string | null>(null)
const editName = ref('')
const editContent = ref('')

onMounted(async () => {
  isMaximized.value = await appWindow.isMaximized()
  await promptStore.loadPrompts()
})

async function minimizeWindow() { await appWindow.minimize() }
async function toggleMaximize() {
  if (isMaximized.value) { await appWindow.unmaximize() } else { await appWindow.maximize() }
  isMaximized.value = await appWindow.isMaximized()
}
async function closeWindow() { await appWindow.close() }

function selectPrompt(id: string) {
  promptStore.selectPrompt(id)
  const prompt = promptStore.prompts.find(p => p.id === id)
  if (prompt) {
    editingId.value = id
    editName.value = prompt.name
    editContent.value = prompt.content
  }
}

function createNew() {
  editingId.value = null
  editName.value = ''
  editContent.value = ''
}

async function save() {
  if (!editName.value.trim() || !editContent.value.trim()) return

  if (editingId.value) {
    await promptStore.updatePrompt(editingId.value, editName.value, editContent.value)
  } else {
    const newPrompt = await promptStore.createPrompt(editName.value, editContent.value)
    editingId.value = newPrompt.id
  }
}

async function remove() {
  if (!editingId.value) return
  await promptStore.deletePrompt(editingId.value)
  editingId.value = null
  editName.value = ''
  editContent.value = ''
}

async function setDefault() {
  if (!editingId.value) return
  await promptStore.setDefaultPrompt(editingId.value)
}
</script>

<template>
  <div class="flex h-full flex-col">
    <header
      data-tauri-drag-region
      class="flex h-9 shrink-0 items-center justify-between bg-muted px-3 select-none"
    >
      <span class="text-sm font-medium text-muted-foreground">提示词</span>
      <div class="flex items-center gap-0.5">
        <Button variant="ghost" size="icon" class="h-6 w-6 hover:bg-background" @click="minimizeWindow"><Minus class="h-3.5 w-3.5" /></Button>
        <Button variant="ghost" size="icon" class="h-6 w-6 hover:bg-background" @click="toggleMaximize">
          <Maximize2 v-if="!isMaximized" class="h-3.5 w-3.5" />
          <Minimize2 v-else class="h-3.5 w-3.5" />
        </Button>
        <Button variant="ghost" size="icon" class="h-6 w-6 hover:bg-destructive hover:text-destructive-foreground" @click="closeWindow"><X class="h-3.5 w-3.5" /></Button>
      </div>
    </header>
    <div class="flex flex-1 overflow-hidden p-1">
      <div class="flex flex-1 overflow-hidden rounded-lg border bg-background">
        <!-- 左侧列表 -->
        <div class="w-48 shrink-0 border-r bg-muted">
          <div class="flex items-center justify-between p-2">
            <span class="text-xs font-medium text-muted-foreground">模板列表</span>
            <Button variant="ghost" size="icon" class="h-6 w-6 hover:bg-background" @click="createNew">
              <Plus class="h-3.5 w-3.5" />
            </Button>
          </div>
          <div class="flex flex-col gap-0.5 px-1">
            <button
              v-for="prompt in promptStore.prompts"
              :key="prompt.id"
              :class="cn(
                'flex items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm transition-colors',
                editingId === prompt.id
                  ? 'bg-background text-foreground shadow-sm'
                  : 'hover:bg-background'
              )"
              @click="selectPrompt(prompt.id)"
            >
              <Star v-if="prompt.is_default" class="h-3 w-3 shrink-0 text-yellow-500" />
              <span class="truncate">{{ prompt.name }}</span>
            </button>
          </div>
        </div>

        <!-- 右侧编辑区 -->
        <div class="flex flex-1 flex-col overflow-hidden">
          <div class="flex-1 overflow-y-auto p-4">
            <div v-if="editingId !== null || editName || editContent" class="flex flex-col gap-4">
              <div class="flex flex-col gap-2">
                <label class="text-sm font-medium text-muted-foreground">模板名称</label>
                <input
                  v-model="editName"
                  :class="cn(
                    'flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm',
                    'transition-colors placeholder:text-muted-foreground',
                    'focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring'
                  )"
                  placeholder="输入模板名称"
                />
              </div>
              <div class="flex flex-1 flex-col gap-2">
                <label class="text-sm font-medium text-muted-foreground">提示词内容</label>
                <textarea
                  v-model="editContent"
                  :class="cn(
                    'flex min-h-[200px] w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm',
                    'transition-colors placeholder:text-muted-foreground',
                    'focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring'
                  )"
                  placeholder="输入提示词内容"
                />
              </div>
              <div class="flex items-center gap-2">
                <Button size="sm" @click="save">
                  保存
                </Button>
                <Button v-if="editingId" variant="outline" size="sm" @click="setDefault">
                  <Star class="h-3.5 w-3.5" />
                  设为默认
                </Button>
                <Button v-if="editingId" variant="destructive" size="sm" @click="remove">
                  <Trash2 class="h-3.5 w-3.5" />
                  删除
                </Button>
              </div>
            </div>
            <div v-else class="flex h-full items-center justify-center text-sm text-muted-foreground">
              选择或创建一个提示词模板
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
