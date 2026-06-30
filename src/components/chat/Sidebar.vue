<script setup lang="ts">
import { computed, ref, nextTick, onMounted, onUnmounted } from 'vue'
import type { Conversation } from '../../types'
import { SettingsIcon, ChevronLeftIcon } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Separator } from '@/components/ui/separator'

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

const hoveredKey = ref<string | null>(null)
const editingId = ref<string | null>(null)
const editingTitle = ref('')
const searchQuery = ref('')
const forceUpdateKey = ref(0)

// Context menu state
const contextMenuVisible = ref(false)
const contextMenuX = ref(0)
const contextMenuY = ref(0)
const contextMenuConvId = ref<string | null>(null)

const filteredConversations = computed(() =>
  props.conversations.filter(conv =>
    conv.title.toLowerCase().includes(searchQuery.value.toLowerCase())
  )
)

const sortedConversations = computed(() =>
  [...filteredConversations.value].sort((a, b) => {
    if (a.pinned === b.pinned) return 0
    return a.pinned ? -1 : 1
  })
)

function handleSelect(id: string) {
  emit('select', id)
}

function handleDelete(id: string) {
  emit('close', id)
  closeContextMenu()
}

function startEdit(id: string) {
  closeContextMenu()
  const conv = props.conversations.find(c => c.id === id)
  if (conv) {
    editingId.value = id
    editingTitle.value = conv.title
    nextTick(() => {
      const el = document.querySelector('.inline-rename-input') as HTMLInputElement
      el?.focus()
      el?.select()
    })
  }
}

function confirmEdit() {
  if (editingId.value && editingTitle.value.trim()) {
    emit('rename', editingId.value, editingTitle.value.trim())
  }
  editingId.value = null
  editingTitle.value = ''
}

function cancelEdit() {
  editingId.value = null
  editingTitle.value = ''
}

function handleTogglePin(id: string) {
  const conv = props.conversations.find(c => c.id === id)
  if (conv?.pinned) {
    emit('unpin', id)
  } else {
    emit('pin', id)
  }
  forceUpdateKey.value++
  closeContextMenu()
}

function handleContextMenu(e: MouseEvent, convId: string) {
  e.preventDefault()
  contextMenuConvId.value = convId
  contextMenuX.value = e.clientX
  contextMenuY.value = e.clientY
  contextMenuVisible.value = true
}

function closeContextMenu() {
  contextMenuVisible.value = false
  contextMenuConvId.value = null
}

function handleDocumentClick(e: MouseEvent) {
  if (contextMenuVisible.value) {
    const target = e.target as HTMLElement
    if (!target.closest('.context-menu')) {
      closeContextMenu()
    }
  }
}

onMounted(() => {
  document.addEventListener('click', handleDocumentClick)
})

onUnmounted(() => {
  document.removeEventListener('click', handleDocumentClick)
})
</script>

<template>
  <div class="flex flex-col h-full">
    <!-- 椤堕儴锛氭柊寤哄璇濇寜閽?+ 鎶樺彔鎸夐挳 -->
    <div class="flex items-center gap-normal px-loose pb-normal">
      <Button variant="outline" size="sm" @click="emit('create')" class="flex-1 text-small">
        鏂板缓瀵硅瘽
      </Button>
      <Button variant="ghost" size="icon-sm" @click="emit('toggle-collapse')">
        <ChevronLeftIcon class="size-4" />
      </Button>
    </div>

    <!-- 鎼滅储妗?-->
    <div class="px-loose pb-normal">
      <Input v-model="searchQuery" placeholder="鎼滅储瀵硅瘽..." />
    </div>

    <Separator class="my-0" />

    <!-- 涓儴锛氬璇濆垪琛紙鍙粴鍔級 -->
    <div :key="forceUpdateKey" class="flex-1 overflow-y-auto min-h-0 py-tight">
      <div
        v-for="conv in sortedConversations"
        :key="conv.id"
        class="flex items-center justify-between px-loose py-normal cursor-pointer rounded-soft mx-tight"
        :class="conv.id === activeId ? 'bg-active' : hoveredKey === conv.id ? 'bg-hover' : ''"
        @click="handleSelect(conv.id)"
        @contextmenu="handleContextMenu($event, conv.id)"
        @mouseenter="hoveredKey = conv.id"
        @mouseleave="hoveredKey = null"
      >
        <span v-if="conv.pinned" class="mr-tight flex-shrink-0 leading-none">馃搶</span>
        <Input
          v-if="editingId === conv.id"
          v-model="editingTitle"
          class="inline-rename-input flex-1 min-w-0"
          @keyup.enter="confirmEdit"
          @keyup.escape="cancelEdit"
          @blur="confirmEdit"
          @click.stop
        />
        <span
          v-else
          class="flex-1 min-w-0 truncate line-clampx-tight py-tight text-small"
          :class="conv.id === activeId ? 'text-main' : 'text-sub'"
        >
          {{ conv.title }}
        </span>
        <div v-if="hoveredKey === conv.id && editingId !== conv.id" class="flex gap-tight flex-shrink-0">
          <Button variant="ghost" size="sm" class="text-danger h-auto px-tight py-tight text-small" @click.stop="handleDelete(conv.id)">
            鍒犻櫎
          </Button>
        </div>
      </div>
    </div>

    <!-- 搴曢儴锛氳缃叆鍙?-->
    <div class="flex-shrink-0">
      <Separator class="my-0" />
      <Button
        variant="ghost"
        class="w-full justify-start px-section py-loose rounded-sharp"
        @click="emit('open-settings')"
      >
        <SettingsIcon class="size-5 mr-normal" />
        璁剧疆
      </Button>
    </div>
  </div>

  <!-- 鍙抽敭涓婁笅鏂囪彍鍗?-->
  <teleport to="body">
    <div
      v-if="contextMenuVisible"
      class="context-menu fixed bg-surface border border-border rounded-soft shadow-md py-tight min-w-30"
      :style="{
        left: contextMenuX + 'px',
        top: contextMenuY + 'px',
        zIndex: 9999,
      }"
    >
      <div class="px-normal py-tight">
        <Button variant="ghost" class="w-full justify-start" @click="handleTogglePin(contextMenuConvId!)">
          {{ conversations.find(c => c.id === contextMenuConvId)?.pinned ? '鍙栨秷缃《' : '缃《' }}
        </Button>
      </div>
      <div class="px-normal py-tight">
        <Button variant="ghost" class="w-full justify-start text-warning" @click="startEdit(contextMenuConvId!)">
          閲嶅懡鍚?        </Button>
      </div>
    </div>
  </teleport>
</template>





