<script setup lang="ts">
import { ref, computed, nextTick } from 'vue'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { ContextMenu, ContextMenuTrigger, ContextMenuContent, ContextMenuItem } from '@/components/ui/context-menu'
import {
  Plus,
  Pin,
  PinOff,
  Trash2,
  Edit2,
  Search,
} from 'lucide-vue-next'
import type { ConversationView } from '@/types'

const props = defineProps<{
  conversations: ConversationView[]
  activeId: string | null
  searchQuery: string
}>()

const emit = defineEmits<{
  select: [id: string]
  create: []
  close: [id: string]
  rename: [id: string, title: string]
  pin: [id: string]
  unpin: [id: string]
  'update:searchQuery': [value: string]
  'toggle-collapse': []
}>()

// --- 搜索 ---

const filteredConversations = computed(() => {
  const q = props.searchQuery.toLowerCase()
  const list = props.conversations.filter((c) =>
    c.title.toLowerCase().includes(q),
  )
  return list.sort((a, b) => {
    if (a.pinned !== b.pinned) return a.pinned ? -1 : 1
    return b.updated_at - a.updated_at
  })
})

// --- 内联重命名 ---
const editingId = ref<string | null>(null)
const editingTitle = ref('')

function startRename(conv: ConversationView) {
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
const contextMenuConvId = ref<string | null>(null)

function handlePin() {
  if (!contextMenuConvId.value) return
  const conv = props.conversations.find((c) => c.id === contextMenuConvId.value)
  if (!conv) return
  if (conv.pinned) {
    emit('unpin', conv.id)
  } else {
    emit('pin', conv.id)
  }
}

function handleRenameFromMenu() {
  if (!contextMenuConvId.value) return
  const conv = props.conversations.find((c) => c.id === contextMenuConvId.value)
  if (conv) {
    nextTick(() => startRename(conv))
  }
}

function isPinned() {
  if (!contextMenuConvId.value) return false
  const conv = props.conversations.find((c) => c.id === contextMenuConvId.value)
  return conv?.pinned ?? false
}
</script>

<template>
  <div class="sidebar-container h-full">
    <!-- 搜索栏 -->
    <div class="relative">
      <Search class="absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground" />
      <Input
        :model-value="searchQuery"
        placeholder="搜索对话..."
        size="sidebar"
        class="sidebar-search"
        @update:model-value="(v: string | number) => emit('update:searchQuery', String(v))"
      />
    </div>

    <!-- 新建对话 -->
    <div
      class="sidebar-action"
      @click="emit('create')"
    >
      <Plus class="size-3.5" />
      <span>新建对话</span>
    </div>

    <!-- 对话列表 -->
    <ContextMenu>
      <ContextMenuTrigger as-child>
    <div class="flex-1 overflow-y-auto">
      <div
        v-for="conv in filteredConversations"
        :key="conv.id"
        :class="
          cn(
            'group relative sidebar-item',
            conv.id === activeId && 'bg-accent text-accent-foreground',
          )
        "
        @click="emit('select', conv.id)"
        @contextmenu="contextMenuConvId = conv.id"
      >
        <!-- 标题 / 重命名输入框 -->
        <div class="sidebar-item-content">
          <template v-if="editingId === conv.id">
            <Input
              v-model="editingTitle"
              :data-rename-input="conv.id"
              size="rename"
              class="w-full truncate"
              @keyup.enter="confirmRename"
              @keyup.escape="cancelRename"
              @blur="confirmRename"
            />
          </template>
          <template v-else>
            <span class="truncate text-sm text-muted-foreground">{{ conv.title }}</span>
          </template>
        </div>

        <!-- 右侧操作按钮 -->
        <div
          :class="
            cn(
              'sidebar-item-actions',
              editingId === conv.id ? 'invisible' : 'opacity-0 group-hover:opacity-100',
            )
          "
        >
          <Button
            variant="ghost"
            size="icon"
            @click.stop="startRename(conv)"
          >
            <Edit2 class="size-3" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
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
        <span class="text-sm">{{ searchQuery ? '无匹配结果' : '暂无对话' }}</span>
      </div>
    </div>
      </ContextMenuTrigger>
        <ContextMenuContent>
          <ContextMenuItem @select="handlePin">
            <template v-if="isPinned()">
              <PinOff class="size-3.5" />
              取消置顶
            </template>
            <template v-else>
              <Pin class="size-3.5" />
              置顶
            </template>
          </ContextMenuItem>
          <ContextMenuItem @select="handleRenameFromMenu">
            <Edit2 class="size-3.5" />
            重命名
          </ContextMenuItem>
        </ContextMenuContent>
      </ContextMenu>
    </div>
</template>