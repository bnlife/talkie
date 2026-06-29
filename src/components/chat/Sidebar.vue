<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue'
import type { Conversation } from '../../types'
import { SettingsOutline, PinOutline, Pin } from '@vicons/ionicons5'
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
  'open-settings': []
}>()

const hoveredKey = ref<string | null>(null)
const showRenameModal = ref(false)
const renameTargetId = ref<string | null>(null)
const newTitle = ref('')
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

function openRename(id: string) {
  const conv = props.conversations.find(c => c.id === id)
  if (conv) {
    renameTargetId.value = id
    newTitle.value = conv.title
    showRenameModal.value = true
  }
  closeContextMenu()
}

function confirmRename() {
  if (renameTargetId.value && newTitle.value.trim()) {
    emit('rename', renameTargetId.value, newTitle.value.trim())
  }
  showRenameModal.value = false
  renameTargetId.value = null
  newTitle.value = ''
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
    <div style="flex-shrink: 0; padding: 12px; padding-bottom: 8px;">
      <n-button block secondary @click="emit('create')">
        新建对话
      </n-button>
    </div>

    <!-- 搜索框 -->
    <div style="flex-shrink: 0; padding: 0 12px 12px;">
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
          borderRadius: '6px',
          margin: '0 4px',
          background: conv.id === activeId ? themeVars.hoverColor : 'transparent',
        }"
        @click="handleSelect(conv.id)"
        @contextmenu="handleContextMenu($event, conv.id)"
        @mouseenter="hoveredKey = conv.id"
        @mouseleave="hoveredKey = null"
      >
        <n-icon
          v-if="conv.pinned"
          :component="Pin"
          :size="14"
          style="margin-right: 4px; color: #4b5563; flex-shrink: 0;"
        />
        <n-ellipsis
          :line-clamp="1"
          :style="{
            flex: '1',
            minWidth: 0,
            maxWidth: '120px',
            fontSize: '14px',
            color: conv.id === activeId ? themeVars.textColor1 : themeVars.textColor2,
          }"
        >
          {{ conv.title }}
        </n-ellipsis>
        <div v-if="hoveredKey === conv.id" style="display: flex; gap: 2px; flex-shrink: 0;">
          <n-button text type="error" size="tiny" @click.stop="handleDelete(conv.id)">
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
        <n-button text type="warning" style="width: 100%; justify-content: flex-start;" @click="openRename(contextMenuConvId!)">
          重命名
        </n-button>
      </div>
    </div>
  </teleport>

  <n-modal v-model:show="showRenameModal" title="重命名对话" preset="dialog" style="width: 360px;">
    <n-input
      v-model:value="newTitle"
      placeholder="请输入新标题"
      @keyup.enter="confirmRename"
    />
    <template #footer>
      <n-space justify="end" :size="8">
        <n-button @click="showRenameModal = false">取消</n-button>
        <n-button type="primary" @click="confirmRename">确认</n-button>
      </n-space>
    </template>
  </n-modal>
</template>
