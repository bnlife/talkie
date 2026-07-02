<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import { useSettingsStore } from '@/stores/settingsStore'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import {
  Minus, Maximize2, Minimize2, X,
  Plus, Search, Star, Trash2, Edit2,
} from 'lucide-vue-next'
import SettingsPanel from './SettingsPanel.vue'

const settingsStore = useSettingsStore()
const appWindow = getCurrentWindow()
const isMaximized = ref(false)
const searchQuery = ref('')
const editingId = ref<string | null>(null)
const renamingId = ref<string | null>(null)
const renameValue = ref('')

onMounted(async () => {
  isMaximized.value = await appWindow.isMaximized()
  await settingsStore.loadSettings()
  document.addEventListener('click', hideContextMenu)
})

onBeforeUnmount(() => {
  document.removeEventListener('click', hideContextMenu)
})

async function minimizeWindow() { await appWindow.minimize() }
async function toggleMaximize() {
  if (isMaximized.value) { await appWindow.unmaximize() } else { await appWindow.maximize() }
  isMaximized.value = await appWindow.isMaximized()
}
async function closeWindow() { await appWindow.close() }

const filteredProviders = computed(() => {
  const q = searchQuery.value.toLowerCase()
  return settingsStore.providers.filter(p => p.name.toLowerCase().includes(q))
})

async function addCustom() {
  const p = await settingsStore.addProvider({ name: '新 Provider' })
  editingId.value = p.id
}

function selectProvider(id: string) {
  editingId.value = id
}

function startRename(id: string) {
  renamingId.value = id
  const p = settingsStore.providers.find(p => p.id === id)
  renameValue.value = p?.name ?? ''
}

async function handleDelete(id: string) {
  await settingsStore.removeProvider(id)
  if (editingId.value === id) editingId.value = null
}

const editingProvider = computed(() => {
  if (!editingId.value) return settingsStore.activeProvider
  return settingsStore.providers.find(p => p.id === editingId.value)
})

// --- 右键菜单 ---
const contextMenu = ref({ visible: false, x: 0, y: 0, providerId: '' })

function showContextMenu(e: MouseEvent, id: string) {
  e.preventDefault()
  contextMenu.value = { visible: true, x: e.clientX, y: e.clientY, providerId: id }
}

function hideContextMenu() {
  contextMenu.value.visible = false
}

async function handleSetDefault() {
  await settingsStore.setActiveProvider(contextMenu.value.providerId)
  hideContextMenu()
}

function handleRename() {
  renamingId.value = contextMenu.value.providerId
  const p = settingsStore.providers.find(p => p.id === contextMenu.value.providerId)
  renameValue.value = p?.name ?? ''
  hideContextMenu()
}

async function confirmRename() {
  if (renameValue.value.trim() && renamingId.value) {
    await settingsStore.updateProvider(renamingId.value, { name: renameValue.value.trim() })
  }
  renamingId.value = null
  renameValue.value = ''
}

function cancelRename() {
  renamingId.value = null
  renameValue.value = ''
}

function isDefault(id: string) {
  return settingsStore.active_provider_id === id
}
</script>

<template>
  <div class="flex h-full flex-col">
    <header
      data-tauri-drag-region
      class="flex h-9 shrink-0 items-center justify-between bg-muted px-3 select-none"
    >
      <span class="text-sm font-medium text-muted-foreground">设置</span>
      <div class="flex items-center gap-0.5">
        <Button variant="ghost" size="icon-sm" @click="minimizeWindow"><Minus class="h-3.5 w-3.5" /></Button>
        <Button variant="ghost" size="icon-sm" @click="toggleMaximize">
          <Maximize2 v-if="!isMaximized" class="h-3.5 w-3.5" />
          <Minimize2 v-else class="h-3.5 w-5.5" />
        </Button>
        <Button variant="ghost" size="icon-sm" class="hover:bg-destructive hover:text-destructive-foreground" @click="closeWindow"><X class="h-3.5 w-3.5" /></Button>
      </div>
    </header>
    <div class="flex flex-1 overflow-hidden p-1">
      <div class="flex flex-1 overflow-hidden rounded-lg border bg-background">
        <!-- Sidebar -->
        <div class="flex w-[220px] shrink-0 flex-col gap-1 border-r p-1.5 text-sm">
          <!-- 搜索 -->
          <div class="relative">
            <Search class="absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground" />
            <Input v-model="searchQuery" placeholder="搜索 Provider..." class="h-7 pl-8 text-sm" />
          </div>

          <!-- 新建按钮 -->
          <div
            class="flex cursor-pointer items-center justify-between rounded-md border border-dashed px-2 py-1.5 transition-colors hover:bg-foreground/5"
            @click="addCustom"
          >
            <div class="flex items-center gap-2">
              <Plus class="size-3.5" />
              <span>新建 Provider</span>
            </div>
          </div>

          <!-- 分隔 -->
          <div v-if="filteredProviders.length > 0" class="my-1 border-t" />

          <!-- Provider 列表 -->
          <div class="flex-1 overflow-y-auto">
            <div
              v-for="provider in filteredProviders"
              :key="provider.id"
              :class="cn(
                'group relative flex cursor-pointer items-center justify-between rounded-md px-2 py-1.5 transition-colors hover:bg-foreground/5',
                editingId === provider.id && 'bg-accent text-accent-foreground',
              )"
              @click="selectProvider(provider.id)"
              @contextmenu="showContextMenu($event, provider.id)"
            >
              <div class="flex min-w-0 items-center gap-2">
                <span
                  :class="cn(
                    'size-1.5 shrink-0 rounded-full',
                    provider.enabled ? 'bg-green-500' : 'bg-muted-foreground/30',
                  )"
                />
                <template v-if="renamingId === provider.id">
                  <input
                    v-model="renameValue"
                    class="w-full truncate rounded bg-background px-1 py-0.5 text-sm text-foreground outline-none ring-1 ring-ring"
                    @keyup.enter="confirmRename"
                    @keyup.escape="cancelRename"
                    @blur="confirmRename"
                  />
                </template>
                <template v-else>
                  <span class="truncate text-sm text-muted-foreground">{{ provider.name }}</span>
                </template>
                <Star
                  v-if="isDefault(provider.id) && renamingId !== provider.id"
                  class="size-3 shrink-0 fill-yellow-500 text-yellow-500"
                />
              </div>
              <div class="flex shrink-0 items-center gap-0.5 opacity-0 group-hover:opacity-100">
                <Button
                  variant="ghost"
                  size="icon-sm"
                  class="size-5"
                  @click.stop="startRename(provider.id)"
                >
                  <Edit2 class="size-3" />
                </Button>
                <Button
                  variant="ghost"
                  size="icon-sm"
                  class="size-5"
                  @click.stop="handleDelete(provider.id)"
                >
                  <Trash2 class="size-3" />
                </Button>
              </div>
            </div>

            <div v-if="filteredProviders.length === 0" class="flex flex-col items-center py-8 text-muted-foreground">
              <span class="text-sm">{{ searchQuery ? '无匹配结果' : '暂无 Provider' }}</span>
            </div>
          </div>
        </div>

        <!-- Main: ProviderEditor -->
        <div class="flex-1 overflow-y-auto">
          <SettingsPanel
            v-if="editingProvider"
            :provider="editingProvider"
          />
          <div v-else class="flex h-full items-center justify-center text-muted-foreground">
            <span class="text-sm">选择或新建一个 Provider</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 右键菜单 -->
    <Teleport to="body">
      <div
        v-if="contextMenu.visible"
        :style="{ left: `${contextMenu.x}px`, top: `${contextMenu.y}px` }"
        class="fixed z-50 min-w-28 overflow-hidden rounded-md border bg-popover p-1 text-popover-foreground shadow-md"
        @click.stop
      >
        <button
          class="flex w-full cursor-default items-center gap-2 rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-accent hover:text-accent-foreground"
          @click="handleRename"
        >
          <Edit2 class="size-3.5" />
          重命名
        </button>
        <button
          class="flex w-full cursor-default items-center gap-2 rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-accent hover:text-accent-foreground"
          @click="handleSetDefault"
        >
          设为默认
        </button>
      </div>
    </Teleport>
  </div>
</template>
