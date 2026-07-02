<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useMcpStore } from '@/stores/mcpStore'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { toast } from 'vue-sonner'
import { Minus, Maximize2, Minimize2, X } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import type { McpServer, McpInstance } from '@/types'
import McpSidebar from './McpSidebar.vue'
import McpMarketGrid from './McpMarketGrid.vue'
import McpInstanceDetail from './McpInstanceDetail.vue'
import McpInstallDialog from './McpInstallDialog.vue'
import McpCustomDialog from './McpCustomDialog.vue'

const mcpStore = useMcpStore()
const appWindow = getCurrentWindow()
const isMaximized = ref(false)
const searchQuery = ref('')
const showInstallDialog = ref(false)
const installServer = ref<McpServer | null>(null)
const showCustomDialog = ref(false)
const testingId = ref<string | null>(null)
const testResult = ref<{ id: string; ok: boolean; msg: string } | null>(null)

onMounted(async () => {
  isMaximized.value = await appWindow.isMaximized()
  await mcpStore.loadData()
})

async function minimizeWindow() { await appWindow.minimize() }
async function toggleMaximize() {
  if (isMaximized.value) { await appWindow.unmaximize() } else { await appWindow.maximize() }
  isMaximized.value = await appWindow.isMaximized()
}
async function closeWindow() { await appWindow.close() }

function selectCategory(id: string) {
  mcpStore.activeCategoryId = id
  mcpStore.activeInstanceId = null
}

function selectInstance(id: string) {
  mcpStore.activeInstanceId = id
}

function handleInstall(server: McpServer) {
  installServer.value = server
  showInstallDialog.value = true
}

async function confirmInstall(config: Record<string, string>) {
  if (!installServer.value) return
  try {
    await mcpStore.installServer(installServer.value, config)
    showInstallDialog.value = false
    installServer.value = null
    toast.success('MCP 服务已添加')
  } catch (e) {
    toast.error(`添加失败: ${e}`)
  }
}

function cancelInstall() {
  showInstallDialog.value = false
  installServer.value = null
}

async function confirmCustom(form: { name: string; command: string; args: string; envKey: string; envValue: string }) {
  const argsArr = form.args.split(/\s+/).filter(Boolean)
  const env: Record<string, string> = {}
  if (form.envKey && form.envValue) {
    env[form.envKey] = form.envValue
  }
  const now = Math.floor(Date.now() / 1000)
  const instance: McpInstance = {
    id: crypto.randomUUID(),
    server_id: '',
    name: form.name || '自定义服务',
    enabled: false,
    transport: 'stdio',
    command: form.command,
    args: argsArr,
    env: Object.keys(env).length > 0 ? env : undefined,
    installed_at: now,
  }
  try {
    await mcpStore.addInstance(instance)
    showCustomDialog.value = false
    toast.success('自定义服务已添加')
  } catch (e) {
    toast.error(`添加失败: ${e}`)
  }
}

function cancelCustom() {
  showCustomDialog.value = false
}

async function handleUninstall(id: string) {
  await mcpStore.uninstallInstance(id)
}

async function handleToggle(id: string, enabled: boolean) {
  if (mcpStore.startingIds.has(id)) return
  try {
    await mcpStore.toggleInstance(id, !enabled)
    if (!enabled) {
      toast.success('正在启动 MCP 服务...')
    }
  } catch (e) {
    toast.error(`操作失败: ${e}`)
    mcpStore.startingIds.delete(id)
  }
}

async function handleTest(id: string) {
  testingId.value = id
  testResult.value = null
  try {
    const msg = await mcpStore.testInstance(id)
    testResult.value = { id, ok: true, msg }
    toast.success(msg)
  } catch (e) {
    const errMsg = String(e)
    testResult.value = { id, ok: false, msg: errMsg }
    toast.error(errMsg)
  } finally {
    testingId.value = null
  }
}
</script>

<template>
  <div class="flex h-full flex-col">
    <!-- Header -->
    <header
      data-tauri-drag-region
      class="flex h-9 shrink-0 items-center justify-between bg-muted px-3 select-none"
    >
      <span class="text-sm font-medium text-muted-foreground">MCP 服务</span>
      <div class="flex items-center gap-0.5">
        <Button variant="ghost" size="icon" @click="minimizeWindow"><Minus class="h-3.5 w-3.5" /></Button>
        <Button variant="ghost" size="icon" @click="toggleMaximize">
          <Maximize2 v-if="!isMaximized" class="h-3.5 w-3.5" />
          <Minimize2 v-else class="h-3.5 w-3.5" />
        </Button>
        <Button variant="ghost" size="icon" @click="closeWindow"><X class="h-3.5 w-3.5" /></Button>
      </div>
    </header>

    <div class="flex flex-1 overflow-hidden p-1">
      <div class="flex flex-1 overflow-hidden rounded-lg border bg-background">
        <!-- Sidebar -->
        <McpSidebar
          v-model:search-query="searchQuery"
          @select-category="selectCategory"
          @select-instance="selectInstance"
          @open-custom-dialog="showCustomDialog = true"
          @toggle-instance="handleToggle"
          @uninstall-instance="handleUninstall"
        />

        <!-- Main content -->
        <div class="flex-1 overflow-y-auto p-4">
          <!-- 市场视图：显示 server 卡片列表 -->
          <template v-if="mcpStore.activeCategoryId && !mcpStore.activeInstanceId">
            <McpMarketGrid
              :search-query="searchQuery"
              :installed-server-ids="mcpStore.installedServerIds"
              @install="handleInstall"
            />
          </template>

          <!-- 已安装实例详情 -->
          <template v-else-if="mcpStore.activeInstance">
            <McpInstanceDetail
              :testing-id="testingId"
              :test-result="testResult"
              @toggle="handleToggle"
              @test="handleTest"
              @uninstall="handleUninstall"
            />
          </template>

          <!-- 默认：未选择 -->
          <template v-else>
            <div class="flex h-full items-center justify-center text-muted-foreground">
              <span class="text-sm">选择分类浏览市场，或选择已安装服务查看详情</span>
            </div>
          </template>
        </div>
      </div>
    </div>

    <!-- 安装配置弹窗 -->
    <McpInstallDialog
      :server="installServer"
      @confirm="confirmInstall"
      @cancel="cancelInstall"
    />

    <!-- 自定义服务弹窗 -->
    <McpCustomDialog
      :visible="showCustomDialog"
      @confirm="confirmCustom"
      @cancel="cancelCustom"
    />
  </div>
</template>
