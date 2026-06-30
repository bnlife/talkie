<script setup lang="ts">
import { ref, computed, nextTick, onMounted, onBeforeUnmount } from 'vue'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Separator } from '@/components/ui/separator'
import {
  Search,
  Plus,
  Pin,
  PinOff,
  Trash2,
  Settings,
  Edit2,
  Check,
  X,
} from 'lucide-vue-next'
import type { Conversation } from '@/types'

const props = defineProps<{
  conversations: Conversation[]
  activeId: string | null
}>()

const emit = defineEmits<{
  select: [id: string]
  create: []
  close: [id: string]
  rename: [id: string, title: string]
  pin: [id: string]
  unpin: [id: string]
  'toggle-collapse': []
  'open-settings': []
}>()

// --- 搜索 ---
const searchQuery = ref('')

const filteredConversations = computed(() => {
  const q = searchQuery.value.toLowerCase()
  const list = props.conversations.filter((c) =>
    c.title.toLowerCase().includes(q)
  )
  return list.sort((a, b) => {
    if (a.pinned !== b.pinned) return a.pinned ? -1 : 1
    return b.updated_at - a.updated_at
  })
})

// --- 内联重命名 ---
const editingId = ref<string | null>(null)
const editingTitle = ref('')

function startRename(conv: Conversation) {
  editingId.value = conv.id
  editingTitle.value = conv.title
  nextTick(() => {
    const el = document.querySelector<HTMLInputElement>(
      `[data-rename-input="${conv.id}"]`,
    )
    el?.focus()
    el?.select()
  })
}

function confirmRename() {
  if (editingId.value && editingTitle.value.trim()) {
    emit('rename', editingId.value, editingTitle.value.trim())
  }
  editingId.value = null
  editingTitle.value = ''
}

function cancelRename() {
  editingId.value = null
  editingTitle.value = ''
}

// --- 右键菜单 ---
const contextMenuVisible = ref(false)
const contextMenuX = ref(0)
const contextMenuY = ref(0)
const contextMenuConvId = ref<string | null>(null)

function showContextMenu(e: MouseEvent, conv: Conversation) {
  e.preventDefault()
  contextMenuConvId.value = conv.id
  contextMenuX.value = e.clientX
  contextMenuY.value = e.clientY
  contextMenuVisible.value = true
}

function hideContextMenu() {
  contextMenuVisible.value = false
  contextMenuConvId.value = null
}

function handlePin() {
  if (!contextMenuConvId.value) return
  const conv = props.conversations.find((c) => c.id === contextMenuConvId.value)
  if (!conv) return
  if (conv.pinned) {
    emit('unpin', conv.id)
  } else {
    emit('pin', conv.id)
  }
  hideContextMenu()
}

function handleRenameFromMenu() {
  if (!contextMenuConvId.value) return
  const conv = props.conversations.find((c) => c.id === contextMenuConvId.value)
  hideContextMenu()
  if (conv) {
    nextTick(() => startRename(conv))
  }
}

function isPinned() {
  if (!contextMenuConvId.value) return false
  const conv = props.conversations.find((c) => c.id === contextMenuConvId.value)
  return conv?.pinned ?? false
}

function onDocumentClick() {
  if (contextMenuVisible.value) {
    hideContextMenu()
  }
}

onMounted(() => {
  document.addEventListener('click', onDocumentClick)
})

onBeforeUnmount(() => {
  document.removeEventListener('click', onDocumentClick)
})
</script>

<template>
  <div class="flex h-full flex-col border-r bg-background text-sm">
    <!-- 顶部操作栏 -->
    <div class="flex items-center gap-1 px-2 pt-2 pb-1">
      <Button variant="ghost" size="icon-sm" @click="emit('create')">
        <Plus class="size-4" />
      </Button>
      <Button variant="ghost" size="icon-sm" @click="emit('toggle-collapse')">
        <X class="size-4" />
      </Button>
    </div>

    <!-- 搜索框 -->
    <div class="px-2 pb-2">
      <div class="relative">
        <Search
          class="absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground"
        />
        <Input
          v-model="searchQuery"
          placeholder="搜索对话..."
          class="h-8 pl-8 text-xs"
        />
      </div>
    </div>

    <Separator />

    <!-- 对话列表 -->
    <div class="flex-1 overflow-y-auto px-1 py-1">
      <div
        v-for="conv in filteredConversations"
        :key="conv.id"
        :class="
          cn(
            'group relative flex cursor-pointer items-center justify-between rounded-md px-2 py-1.5 transition-colors hover:bg-accent/50',
            conv.id === activeId && 'bg-accent text-accent-foreground',
          )
        "
        @click="emit('select', conv.id)"
        @contextmenu="showContextMenu($event, conv)"
      >
        <!-- 标题 / 重命名输入框 -->
        <div class="min-w-0 flex-1 truncate">
          <template v-if="editingId === conv.id">
            <div class="flex items-center gap-1">
              <Input
                v-model="editingTitle"
                :data-rename-input="conv.id"
                class="h-6 px-1 text-xs"
                @keyup.enter="confirmRename"
                @keyup.escape="cancelRename"
                @blur="confirmRename"
              />
              <Button
                variant="ghost"
                size="icon-sm"
                class="size-5 shrink-0"
                @click.stop="confirmRename"
              >
                <Check class="size-3" />
              </Button>
              <Button
                variant="ghost"
                size="icon-sm"
                class="size-5 shrink-0"
                @click.stop="cancelRename"
              >
                <X class="size-3" />
              </Button>
            </div>
          </template>
          <template v-else>
            <span class="block truncate text-xs">{{ conv.title }}</span>
          </template>
        </div>

        <!-- 右侧操作按钮 -->
        <div
          :class="
            cn(
              'ml-1 flex shrink-0 items-center gap-0.5',
              editingId === conv.id ? 'invisible' : 'opacity-0 group-hover:opacity-100',
            )
          "
        >
          <Button
            variant="ghost"
            size="icon-sm"
            class="size-5"
            @click.stop="startRename(conv)"
          >
            <Edit2 class="size-3" />
          </Button>
          <Button
            variant="ghost"
            size="icon-sm"
            class="size-5"
            @click.stop="emit('close', conv.id)"
          >
            <Trash2 class="size-3" />
          </Button>
        </div>

        <!-- 置顶图标 -->
        <Pin
          v-if="conv.pinned && editingId !== conv.id"
          class="ml-1 size-3 shrink-0 text-muted-foreground"
        />
      </div>

      <!-- 空状态 -->
      <div
        v-if="filteredConversations.length === 0"
        class="flex flex-col items-center py-8 text-muted-foreground"
      >
        <span class="text-xs">{{ searchQuery ? '无匹配结果' : '暂无对话' }}</span>
      </div>
    </div>

    <!-- 底部设置 -->
    <div class="px-2 py-2">
      <Button
        variant="ghost"
        class="w-full justify-start gap-2 text-xs"
        @click="emit('open-settings')"
      >
        <Settings class="size-4" />
        设置
      </Button>
    </div>

    <!-- 右键菜单（teleport 到 body） -->
    <Teleport to="body">
      <div
        v-if="contextMenuVisible"
        :style="{ left: `${contextMenuX}px`, top: `${contextMenuY}px` }"
        class="fixed z-50 min-w-28 overflow-hidden rounded-md border bg-popover p-1 text-popover-foreground shadow-md"
        @click.stop
      >
        <button
          class="flex w-full cursor-default items-center gap-2 rounded-sm px-2 py-1.5 text-xs outline-none transition-colors hover:bg-accent hover:text-accent-foreground"
          @click="handlePin"
        >
          <template v-if="isPinned()">
            <PinOff class="size-3.5" />
            取消置顶
          </template>
          <template v-else>
            <Pin class="size-3.5" />
            置顶
          </template>
        </button>
        <Separator class="my-0.5" />
        <button
          class="flex w-full cursor-default items-center gap-2 rounded-sm px-2 py-1.5 text-xs outline-none transition-colors hover:bg-accent hover:text-accent-foreground"
          @click="handleRenameFromMenu"
        >
          <Edit2 class="size-3.5" />
          重命名
        </button>
      </div>
    </Teleport>
  </div>
</template>
