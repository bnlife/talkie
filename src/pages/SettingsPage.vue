<script setup lang="ts">
import { onMounted } from 'vue'
import { useSettingsStore } from '../stores/settingsStore'
import SettingsPanel from '../components/settings/SettingsPanel.vue'
import type { Settings } from '../types'

const settingsStore = useSettingsStore()

onMounted(() => {
  settingsStore.loadSettings()
})

function handleUpdate(partial: Partial<Settings>) {
  settingsStore.updateSettings(partial)
}

function handleTestConnection() {
  settingsStore.testConnection()
}
</script>

<template>
  <div style="display: flex; justify-content: center; align-items: center; min-height: 100%; padding: 32px;">
    <n-card title="设置" style="width: 480px;">
      <SettingsPanel
        :settings="settingsStore.$state"
        @update="handleUpdate"
        @test-connection="handleTestConnection"
      />
    </n-card>
  </div>
</template>
