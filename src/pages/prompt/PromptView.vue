<script setup lang="ts">
import { ref, computed, nextTick, onMounted, onUnmounted } from 'vue'
import { usePromptStore } from '@/stores/promptStore'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { Separator } from '@/components/ui/separator'
import {
  PanelLeftOpen,
  PanelLeftClose,
  Minus,
  Maximize2,
  Minimize2,
  X,
  Plus,
  Search,
  Edit2,
  Trash2,
  Star,
} from 'lucide-vue-next'

const promptStore = usePromptStore()
const appWindow = getCurrentWindow()
const isMaximized = ref(false)
const sidebarCollapsed = ref(false)
const searchQuery = ref('')

const editingId = ref<string | null>(null)
const editName = ref('')
const editContent = ref('')
const isCreating = ref(false)

// --- 搜索过滤 ---
const filteredPrompts = computed(() => {
  const q = searchQuery.value.toLowerCase()
  return promptStore.prompts.filter(p =>
    p.name.toLowerCase().includes(q)
  )
})

// --- 窗口控制 ---
function toggleSidebar() { sidebarCollapsed.value = !sidebarCollapsed.value }
async function minimizeWindow() { await appWindow.minimize() }
async function toggleMaximize() {
  if (isMaximized.value) { await appWindow.unmaximize() } else { await appWindow.maximize() }
  isMaximized.value = await appWindow.isMaximized()
}
async function closeWindow() { await appWindow.close() }

// --- 提示词操作 ---
function selectPrompt(id: string) {
  promptStore.selectPrompt(id)
  const prompt = promptStore.prompts.find(p => p.id === id)
  if (prompt) {
    editingId.value = id
    editName.value = prompt.name
    editContent.value = prompt.content
    isCreating.value = false
  }
}

function createNew() {
  editingId.value = null
  editName.value = ''
  editContent.value = ''
  isCreating.value = true
}

async function save() {
  if (!editName.value.trim() || !editContent.value.trim()) return

  if (editingId.value) {
    await promptStore.updatePrompt(editingId.value, editName.value, editContent.value)
  } else if (isCreating.value) {
    const newPrompt = await promptStore.createPrompt(editName.value, editContent.value)
    editingId.value = newPrompt.id
    isCreating.value = false
  }
}

async function removePrompt(id: string) {
  if (editingId.value === id) {
    editingId.value = null
    editName.value = ''
    editContent.value = ''
    isCreating.value = false
  }
  await promptStore.deletePrompt(id)
}

// --- 右键菜单 ---
const contextMenuVisible = ref(false)
const contextMenuX = ref(0)
const contextMenuY = ref(0)
const contextMenuPromptId = ref<string | null>(null)

function showContextMenu(e: MouseEvent, prompt: { id: string; is_default: boolean }) {
  e.preventDefault()
  contextMenuPromptId.value = prompt.id
  contextMenuX.value = e.clientX
  contextMenuY.value = e.clientY
  contextMenuVisible.value = true
}

function hideContextMenu() {
  contextMenuVisible.value = false
  contextMenuPromptId.value = null
}

function handleContextMenuDelete() {
  if (!contextMenuPromptId.value) return
  const id = contextMenuPromptId.value
  hideContextMenu()
  if (editingId.value === id) {
    editingId.value = null
    editName.value = ''
    editContent.value = ''
    isCreating.value = false
  }
  promptStore.deletePrompt(id)
}

function handleContextMenuDefault() {
  if (!contextMenuPromptId.value) return
  const id = contextMenuPromptId.value
  hideContextMenu()
  promptStore.setDefaultPrompt(id)
  if (editingId.value === id) {
    // refresh local state
  }
}

function onDocumentClick() {
  if (contextMenuVisible.value) hideContextMenu()
}

onMounted(async () => {
  isMaximized.value = await appWindow.isMaximized()
  await promptStore.loadPrompts()
  document.addEventListener('click', onDocumentClick)
})

onUnmounted(() => {
  document.removeEventListener('click', onDocumentClick)
})
</script>

<template>
  <div class="flex h-full flex-col">
    <!-- Header -->
    <header
      data-tauri-drag-region
      class="flex h-9 shrink-0 items-center justify-between bg-muted px-3 select-none"
    >
      <div class="flex items-center gap-2">
        <Button variant="ghost" size="icon" @click.stop="toggleSidebar">
          <PanelLeftClose v-if="!sidebarCollapsed" class="h-3.5 w-3.5" />
          <PanelLeftOpen v-else class="h-3.5 w-3.5" />
        </Button>
        <span class="text-sm font-medium text-muted-foreground">提示词</span>
      </div>
      <div class="flex items-center gap-0.5">
        <Button variant="ghost" size="icon" @click="minimizeWindow"><Minus class="h-3.5 w-3.5" /></Button>
        <Button variant="ghost" size="icon" @click="toggleMaximize">
          <Maximize2 v-if="!isMaximized" class="h-3.5 w-3.5" />
          <Minimize2 v-else class="h-3.5 w-3.5" />
        </Button>
        <Button variant="ghost" size="icon" @click="closeWindow"><X class="h-3.5 w-3.5" /></Button>
      </div>
    </header>

    <!-- Content -->
    <div class="flex flex-1 overflow-hidden p-1">
      <div class="flex flex-1 overflow-hidden rounded-lg border">
        <!-- Sidebar -->
        <aside
          v-show="!sidebarCollapsed"
          class="w-[220px] shrink-0 border-r bg-background overflow-hidden flex flex-col"
        >
          <div class="sidebar-container h-full">
            <!-- 搜索栏 -->
            <div class="relative">
              <Search class="absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground" />
              <Input
                v-model="searchQuery"
                placeholder="搜索提示词..."
                class="sidebar-search h-8 pl-8"
              />
            </div>

            <!-- 新建提示词 -->
            <div
              class="sidebar-action"
              @click="createNew"
            >
              <Plus class="size-3.5" />
              <span>新建提示词</span>
            </div>

            <!-- 提示词列表 -->
            <div class="flex-1 overflow-y-auto">
              <div
                v-for="prompt in filteredPrompts"
                :key="prompt.id"
                :class="
                  cn(
                    'group relative sidebar-item',
                    prompt.id === editingId && 'bg-accent text-accent-foreground',
                  )
                "
                @click="selectPrompt(prompt.id)"
                @contextmenu="showContextMenu($event, prompt)"
              >
                <div class="sidebar-item-content">
                  <Star v-if="prompt.is_default" class="size-3 shrink-0 text-warning" />
                  <span class="truncate text-sm text-muted-foreground">{{ prompt.name }}</span>
                </div>

                <div
                  :class="
                    cn(
                      'sidebar-item-actions',
                      'opacity-0 group-hover:opacity-100',
                    )
                  "
                >
                  <Button
                    variant="ghost"
                    size="icon"
                    @click.stop="selectPrompt(prompt.id)"
                  >
                    <Edit2 class="size-3" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon"
                    @click.stop="removePrompt(prompt.id)"
                  >
                    <Trash2 class="size-3" />
                  </Button>
                </div>
              </div>

              <!-- 空状态 -->
              <div
                v-if="filteredPrompts.length === 0"
                class="flex flex-col items-center py-8 text-muted-foreground"
              >
                <span class="text-sm">{{ searchQuery ? '无匹配结果' : '暂无提示词' }}</span>
              </div>
            </div>
          </div>
        </aside>

        <!-- Main content -->
        <main class="relative flex flex-1 flex-col overflow-hidden bg-background">
          <div v-if="editingId !== null || isCreating" class="flex flex-1 flex-col overflow-hidden p-4">
            <div class="flex flex-1 flex-col gap-3">
              <Input
                id="prompt-name"
                v-model="editName"
                placeholder="模板名称"
                class="h-7 text-sm font-medium"
                @blur="save"
              />
              <Textarea
                id="prompt-content"
                v-model="editContent"
                placeholder="输入提示词内容..."
                class="flex-1 resize-none text-sm"
                @blur="save"
              />
            </div>
          </div>

          <!-- 空状态 -->
          <div
            v-else
            class="flex flex-1 items-center justify-center text-sm text-muted-foreground"
          >
            选择或创建一个提示词模板
          </div>
        </main>
      </div>
    </div>

    <!-- 右键菜单 -->
    <Teleport to="body">
      <div
        v-if="contextMenuVisible"
        :style="{ left: `${contextMenuX}px`, top: `${contextMenuY}px` }"
        class="fixed z-50 min-w-28 overflow-hidden rounded-md border bg-popover p-1 text-popover-foreground shadow-md"
        @click.stop
      >
        <button
          class="flex w-full cursor-default items-center gap-2 rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-accent hover:text-accent-foreground"
          @click="handleContextMenuDefault"
        >
          <Star class="size-3.5" />
          设为默认
        </button>
        <button
          class="flex w-full cursor-default items-center gap-2 rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-accent hover:text-accent-foreground"
          @click="handleContextMenuDelete"
        >
          <Trash2 class="size-3.5" />
          删除
        </button>
      </div>
    </Teleport>
  </div>
</template>
