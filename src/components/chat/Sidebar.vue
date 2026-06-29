<script setup lang="ts">
import { computed, ref } from 'vue'
import type { Conversation } from '../../types'
import type { MenuOption } from 'naive-ui'
import { SettingsOutline } from '@vicons/ionicons5'

const props = defineProps<{
  conversations: Conversation[]
  activeId: string | null
}>()

const emit = defineEmits<{
  select: [id: string]
  create: []
  close: [id: string]
  rename: [id: string, title: string]
  'open-settings': []
}>()

const hoveredKey = ref<string | null>(null)
const showRenameModal = ref(false)
const renameTargetId = ref<string | null>(null)
const newTitle = ref('')

const menuOptions = computed<MenuOption[]>(() =>
  props.conversations.map(conv => ({
    key: conv.id,
    label: conv.title,
  }))
)

function handleSelect(key: string | number) {
  emit('select', String(key))
}

function handleDelete(key: string | number) {
  emit('close', String(key))
}

function openRename(key: string | number) {
  const conv = props.conversations.find(c => c.id === key)
  if (conv) {
    renameTargetId.value = String(key)
    newTitle.value = conv.title
    showRenameModal.value = true
  }
}

function confirmRename() {
  if (renameTargetId.value && newTitle.value.trim()) {
    emit('rename', renameTargetId.value, newTitle.value.trim())
  }
  showRenameModal.value = false
  renameTargetId.value = null
  newTitle.value = ''
}
</script>

<template>
  <div style="display: flex; flex-direction: column; height: 100%;">
    <!-- 顶部：新建对话按钮 -->
    <div style="flex-shrink: 0; padding: 12px;">
      <n-button block secondary @click="emit('create')">
        新建对话
      </n-button>
    </div>

    <!-- 中部：对话列表（可滚动） -->
    <div style="flex: 1; overflow-y: auto; min-height: 0;">
      <n-menu
        :value="activeId"
        :options="menuOptions"
        @update:value="handleSelect"
      >
        <template #render-label="{ option }">
          <div
            style="display: flex; align-items: center; justify-content: space-between; width: 100%; min-width: 0;"
            @mouseenter="hoveredKey = String(option.key)"
            @mouseleave="hoveredKey = null"
          >
            <n-ellipsis :line-clamp="1" style="flex: 1; max-width: 120px;">
              {{ option.label }}
            </n-ellipsis>
            <n-space v-if="hoveredKey === option.key" :size="2">
              <n-button text type="warning" size="tiny" @click.stop="openRename(option.key)">
                重命名
              </n-button>
              <n-button text type="error" size="tiny" @click.stop="handleDelete(option.key)">
                删除
              </n-button>
            </n-space>
          </div>
        </template>
      </n-menu>
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
