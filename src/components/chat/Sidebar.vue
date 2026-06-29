<script setup lang="ts">
import { computed, ref, nextTick, onMounted, onUnmounted } from 'vue'
import type { Conversation } from '../../types'
import { SettingsOutline, ChevronBackOutline } from '@vicons/ionicons5'
import { useThemeVars } from 'naive-ui'

const themeVars = useThemeVars()

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
  <div style="display: flex; flex-direction: column; height: 100%;">
    <!-- 顶部：新建对话按钮 -->
    <div style="flex-shrink: 0; display: flex; align-items: center; gap: 4px; padding: 0 12px 8px;">
      <n-button secondary @click="emit('create')" style="flex: 1;">
        新建对话
      </n-button>
      <n-button text @click="emit('toggle-collapse')" style="width: 28px;">
        <template #icon><n-icon :component="ChevronBackOutline" /></template>
      </n-button>
    </div>

    <!-- 搜索框 -->
    <div style="flex-shrink: 0; padding: 0 12px 8px;">
      <n-input
        v-model:value="searchQuery"
        placeholder="搜索对话..."
        clearable
      />
    </div>

    <n-divider style="margin: 0;" />

    <!-- 中部：对话列表（可滚动） -->
    <div :key="forceUpdateKey" style="flex: 1; overflow-y: auto; min-height: 0; padding: 4px 0;">
      <div
        v-for="conv in sortedConversations"
        :key="conv.id"
        class="conv-item"
        :style="{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: '8px 12px',
          cursor: 'pointer',
          borderRadius: '8px',
          margin: '0 4px',
          background: conv.id === activeId
            ? themeVars.pressedColor
            : hoveredKey === conv.id
              ? themeVars.hoverColor
              : 'transparent',
        }"
        @click="handleSelect(conv.id)"
        @contextmenu="handleContextMenu($event, conv.id)"
        @mouseenter="hoveredKey = conv.id"
        @mouseleave="hoveredKey = null"
      >
        <span v-if="conv.pinned" style="margin-right: 4px; flex-shrink: 0; line-height: 1;">📌</span>
        <n-input
          v-if="editingId === conv.id"
          v-model:value="editingTitle"
          class="inline-rename-input"
          style="flex: 1; min-width: 0;"
          @keyup.enter="confirmEdit"
          @keyup.escape="cancelEdit"
          @blur="confirmEdit"
          @click.stop
        />
        <n-ellipsis
          v-else
          :line-clamp="1"
          :style="{
            flex: '1',
            minWidth: 0,
            color: conv.id === activeId ? themeVars.textColor1 : themeVars.textColor2,
          }"
        >
          {{ conv.title }}
        </n-ellipsis>
        <div v-if="hoveredKey === conv.id" style="display: flex; gap: 2px; flex-shrink: 0;">
          <n-button text type="error" @click.stop="handleDelete(conv.id)">
            删除
          </n-button>
        </div>
      </div>
    </div>

    <!-- 底部：设置入口 -->
    <div style="flex-shrink: 0;">
      <n-divider style="margin: 0;" />
      <n-button
        text
        block
        style="justify-content: flex-start; padding: 12px 16px; border-radius: 0;"
        @click="emit('open-settings')"
      >
        <n-icon :component="SettingsOutline" :size="18" style="margin-right: 8px;" />
        设置
      </n-button>
    </div>
  </div>

  <!-- 右键上下文菜单 -->
  <teleport to="body">
    <div
      v-if="contextMenuVisible"
      class="context-menu"
      :style="{
        position: 'fixed',
        left: contextMenuX + 'px',
        top: contextMenuY + 'px',
        zIndex: 9999,
        background: themeVars.cardColor,
        borderRadius: '8px',
        border: '1px solid ' + themeVars.borderColor,
        boxShadow: themeVars.boxShadow2,
        padding: '4px 0',
        minWidth: '120px',
      }"
    >
      <div style="padding: 4px 8px;">
        <n-button text style="width: 100%; justify-content: flex-start;" @click="handleTogglePin(contextMenuConvId!)">
          {{ conversations.find(c => c.id === contextMenuConvId)?.pinned ? '取消置顶' : '置顶' }}
        </n-button>
      </div>
      <div style="padding: 4px 8px;">
        <n-button text type="warning" style="width: 100%; justify-content: flex-start;" @click="startEdit(contextMenuConvId!)">
          重命名
        </n-button>
      </div>
    </div>
  </teleport>


</template>
