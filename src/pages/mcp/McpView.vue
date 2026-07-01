<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useMcpStore } from '@/stores/mcpStore'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { toast } from 'vue-sonner'
import {
  Minus, Maximize2, Minimize2, X,
  Plus, Search, Trash2, Settings,
} from 'lucide-vue-next'
import type { McpServer, McpInstance } from '@/types'

const mcpStore = useMcpStore()
const appWindow = getCurrentWindow()
const isMaximized = ref(false)
const searchQuery = ref('')
const selectedServerId = ref<string | null>(null)
const showInstallDialog = ref(false)
const installServer = ref<McpServer | null>(null)
const installConfig = ref<Record<string, string>>({})
const showCustomDialog = ref(false)
const testingId = ref<string | null>(null)
const testResult = ref<{ id: string; ok: boolean; msg: string } | null>(null)
const customForm = ref({
  name: '',
  command: 'npx',
  args: '-y @humansean/mcp-bocha',
  envKey: 'BOCHA_API_KEY',
  envValue: '',
})

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

const filteredServers = computed(() => {
  const q = searchQuery.value.toLowerCase()
  if (!q) return mcpStore.filteredServers
  return mcpStore.servers.filter(s =>
    s.name.toLowerCase().includes(q) || s.description.toLowerCase().includes(q)
  )
})

function selectCategory(id: string) {
  mcpStore.activeCategoryId = id
  selectedServerId.value = null
}

function selectInstance(id: string) {
  mcpStore.activeInstanceId = id
  selectedServerId.value = null
}

function isInstalled(serverId: string): boolean {
  return mcpStore.installedServerIds.has(serverId)
}

function handleInstall(server: McpServer) {
  installServer.value = server
  installConfig.value = {}
  // Pre-fill defaults
  if (server.env_vars) {
    for (const v of server.env_vars) {
      if (v.default) installConfig.value[v.name] = v.default
    }
  }
  if (server.args) {
    for (const a of server.args) {
      if (a.default) installConfig.value[a.valueHint || a.name || ''] = a.default
    }
  }
  showInstallDialog.value = true
}

async function confirmInstall() {
  if (!installServer.value) return
  try {
    await mcpStore.installServer(installServer.value, installConfig.value)
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

function openCustomDialog() {
  customForm.value = {
    name: '',
    command: 'cmd',
    args: '/c npx -y @humansean/mcp-bocha',
    envKey: 'BOCHA_API_KEY',
    envValue: '',
  }
  showCustomDialog.value = true
}

async function confirmCustom() {
  const argsArr = customForm.value.args.split(/\s+/).filter(Boolean)
  const env: Record<string, string> = {}
  if (customForm.value.envKey && customForm.value.envValue) {
    env[customForm.value.envKey] = customForm.value.envValue
  }
  const now = Math.floor(Date.now() / 1000)
  const instance: McpInstance = {
    id: crypto.randomUUID(),
    server_id: '',
    name: customForm.value.name || '自定义服务',
    enabled: false,
    transport: 'stdio',
    command: customForm.value.command,
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
  if (mcpStore.startingIds.has(id)) return // Already starting, ignore
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

function getInstanceStatus(inst: McpInstance): 'running' | 'starting' | 'stopped' {
  if (mcpStore.startingIds.has(inst.id)) return 'starting'
  if (inst.enabled) return 'running'
  return 'stopped'
}

function getInstanceStatusText(inst: McpInstance): string {
  const status = getInstanceStatus(inst)
  if (status === 'starting') return '启动中...'
  if (status === 'running') return '运行中'
  return '已暂停'
}

const selectedServer = computed(() => {
  if (!selectedServerId.value) return null
  return mcpStore.servers.find(s => s.id === selectedServerId.value) || null
})
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
        <Button variant="ghost" size="icon" class="h-6 w-6 hover:bg-background" @click="minimizeWindow"><Minus class="h-3.5 w-3.5" /></Button>
        <Button variant="ghost" size="icon" class="h-6 w-6 hover:bg-background" @click="toggleMaximize">
          <Maximize2 v-if="!isMaximized" class="h-3.5 w-3.5" />
          <Minimize2 v-else class="h-3.5 w-3.5" />
        </Button>
        <Button variant="ghost" size="icon" class="h-6 w-6 hover:bg-destructive hover:text-destructive-foreground" @click="closeWindow"><X class="h-3.5 w-3.5" /></Button>
      </div>
    </header>

    <div class="flex flex-1 overflow-hidden p-1">
      <div class="flex flex-1 overflow-hidden rounded-lg border bg-background">
        <!-- Sidebar -->
        <div class="flex w-60 shrink-0 flex-col gap-1 border-r p-1.5 text-sm">
          <!-- 搜索 -->
          <div class="relative">
            <Search class="absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground" />
            <Input v-model="searchQuery" placeholder="搜索 MCP 服务..." class="h-7 pl-8 text-sm" />
          </div>

          <!-- 添加自定义 -->
          <div
            class="flex cursor-pointer items-center gap-2 rounded-md border border-dashed px-2 py-1.5 transition-colors hover:bg-foreground/5"
            @click="openCustomDialog"
          >
            <Plus class="size-3.5" />
            <span>添加自定义服务</span>
          </div>

          <div class="my-1 border-t" />

          <!-- 市场分类 -->
          <div class="text-xs font-medium text-foreground px-1">市场</div>
          <div
            v-for="cat in mcpStore.categories"
            :key="cat.id"
            :class="cn(
              'flex cursor-pointer items-center gap-2 rounded-md px-2 py-1.5 transition-colors hover:bg-foreground/5',
              mcpStore.activeCategoryId === cat.id && 'bg-accent text-accent-foreground',
            )"
            @click="selectCategory(cat.id)"
          >
            <span>{{ cat.icon }}</span>
            <span class="truncate text-sm text-muted-foreground">{{ cat.name }}</span>
          </div>

          <div class="my-1 border-t" />

          <!-- 已安装 -->
          <div class="text-xs font-medium text-foreground px-1">已安装</div>
          <div class="flex-1 overflow-y-auto">
            <div
              v-for="inst in mcpStore.instances"
              :key="inst.id"
              :class="cn(
                'group flex cursor-pointer items-center justify-between rounded-md px-2 py-1.5 transition-colors hover:bg-foreground/5',
                mcpStore.activeInstanceId === inst.id && 'bg-accent text-accent-foreground',
              )"
              @click="selectInstance(inst.id)"
            >
              <div class="flex min-w-0 items-center gap-2">
                <span
                  :class="cn(
                    'size-1.5 shrink-0 rounded-full',
                    getInstanceStatus(inst) === 'running' ? 'bg-green-500' :
                    getInstanceStatus(inst) === 'starting' ? 'bg-yellow-500 animate-pulse' :
                    'bg-muted-foreground/30',
                  )"
                />
                <span class="truncate text-sm text-muted-foreground">{{ inst.name }}</span>
              </div>
              <div class="flex shrink-0 items-center gap-0.5 opacity-0 group-hover:opacity-100">
                <Button
                  variant="ghost"
                  size="icon-sm"
                  class="size-5"
                  @click.stop="handleToggle(inst.id, inst.enabled)"
                >
                  <Settings class="size-3" />
                </Button>
                <Button
                  variant="ghost"
                  size="icon-sm"
                  class="size-5"
                  @click.stop="handleUninstall(inst.id)"
                >
                  <Trash2 class="size-3" />
                </Button>
              </div>
            </div>

            <div v-if="mcpStore.instances.length === 0" class="py-4 text-center text-xs text-muted-foreground">
              暂无已安装服务
            </div>
          </div>
        </div>

        <!-- Main content -->
        <div class="flex-1 overflow-y-auto p-4">
          <!-- 市场视图：显示 server 卡片列表 -->
          <template v-if="mcpStore.activeCategoryId && !mcpStore.activeInstanceId">
            <div class="mb-3 flex items-center justify-between">
              <h2 class="text-sm font-medium">{{ mcpStore.activeCategory?.icon }} {{ mcpStore.activeCategory?.name }}</h2>
              <span class="text-xs text-muted-foreground">{{ filteredServers.length }} 个服务</span>
            </div>
            <div class="grid grid-cols-2 gap-3">
              <div
                v-for="server in filteredServers"
                :key="server.id"
                class="rounded-lg border p-3 transition-colors hover:bg-foreground/5"
              >
                <div class="mb-1 flex items-center justify-between">
                  <span class="text-sm font-medium">{{ server.name }}</span>
                  <span class="text-xs text-muted-foreground">{{ server.publisher }}</span>
                </div>
                <p class="mb-2 text-xs text-muted-foreground line-clamp-2">{{ server.description }}</p>
                <div class="flex items-center justify-between">
                  <span v-if="server.github_stars" class="text-xs text-muted-foreground">⭐ {{ (server.github_stars / 1000).toFixed(0) }}k</span>
                  <span v-else />
                  <Button
                    v-if="!isInstalled(server.id)"
                    size="sm"
                    class="h-6 text-xs"
                    @click="handleInstall(server)"
                  >
                    添加
                  </Button>
                  <span v-else class="text-xs text-green-500">✓ 已添加</span>
                </div>
              </div>
            </div>
            <div v-if="filteredServers.length === 0" class="flex flex-col items-center py-16 text-muted-foreground">
              <Search class="mb-2 size-8" />
              <p class="text-sm">{{ searchQuery ? '无匹配结果' : '该分类暂无服务' }}</p>
            </div>
          </template>

          <!-- 已安装实例详情 -->
          <template v-else-if="mcpStore.activeInstance">
            <div class="mb-3">
              <h2 class="text-sm font-medium">{{ mcpStore.activeInstance.name }}</h2>
              <p class="text-xs text-muted-foreground mt-1">
                状态：
                <span :class="{
                  'text-green-500': getInstanceStatus(mcpStore.activeInstance) === 'running',
                  'text-yellow-500': getInstanceStatus(mcpStore.activeInstance) === 'starting',
                  'text-muted-foreground': getInstanceStatus(mcpStore.activeInstance) === 'stopped',
                }">
                  {{ getInstanceStatusText(mcpStore.activeInstance) }}
                </span>
                <span v-if="mcpStore.errorMap[mcpStore.activeInstance.id]" class="text-red-500 ml-2">
                  ({{ mcpStore.errorMap[mcpStore.activeInstance.id] }})
                </span>
              </p>
            </div>
            <div class="space-y-3">
              <div>
                <div class="text-xs font-medium text-foreground mb-1">配置</div>
                <div class="rounded-md border p-2 text-xs font-mono text-muted-foreground">
                  <div>传输方式: {{ mcpStore.activeInstance.transport }}</div>
                  <div v-if="mcpStore.activeInstance.command">命令: {{ mcpStore.activeInstance.command }} {{ mcpStore.activeInstance.args?.join(' ') }}</div>
                  <div v-if="mcpStore.activeInstance.url">URL: {{ mcpStore.activeInstance.url }}</div>
                  <div v-if="mcpStore.activeInstance.env">
                    环境变量:
                    <div v-for="(v, k) in mcpStore.activeInstance.env" :key="k" class="ml-2">
                      {{ k }}={{ v }}
                    </div>
                  </div>
                </div>
              </div>
              <div class="flex gap-2">
                <Button
                  size="sm"
                  :variant="getInstanceStatus(mcpStore.activeInstance) === 'running' ? 'outline' : 'default'"
                  :disabled="getInstanceStatus(mcpStore.activeInstance) === 'starting'"
                  @click="handleToggle(mcpStore.activeInstance.id, mcpStore.activeInstance.enabled)"
                >
                  {{ getInstanceStatus(mcpStore.activeInstance) === 'starting' ? '启动中...' :
                     getInstanceStatus(mcpStore.activeInstance) === 'running' ? '暂停' : '启动' }}
                </Button>
                <Button
                  size="sm"
                  variant="outline"
                  :disabled="testingId === mcpStore.activeInstance.id"
                  @click="handleTest(mcpStore.activeInstance.id)"
                >
                  {{ testingId === mcpStore.activeInstance.id ? '测试中...' : '测试连接' }}
                </Button>
                <Button
                  size="sm"
                  variant="destructive"
                  @click="handleUninstall(mcpStore.activeInstance.id)"
                >
                  移除
                </Button>
              </div>
              <div
                v-if="testResult && testResult.id === mcpStore.activeInstance.id"
                :class="cn(
                  'mt-2 rounded-md border px-3 py-2 text-xs',
                  testResult.ok ? 'border-green-500/30 bg-green-500/5 text-green-600' : 'border-red-500/30 bg-red-500/5 text-red-600',
                )"
              >
                {{ testResult.msg }}
              </div>
            </div>
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
    <Teleport to="body">
      <div
        v-if="showInstallDialog && installServer"
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
        @click.self="cancelInstall"
      >
        <div class="w-96 rounded-lg border bg-background p-4 shadow-lg">
          <h3 class="text-sm font-medium mb-3">添加: {{ installServer.name }}</h3>

          <!-- 需要用户填写的参数 -->
          <div v-if="installServer.env_vars && installServer.env_vars.length > 0" class="space-y-2 mb-3">
            <div v-for="v in installServer.env_vars" :key="v.name">
              <label class="text-xs text-muted-foreground">
                {{ v.name }}
                <span v-if="v.required" class="text-red-500">*</span>
                <span v-if="v.description" class="ml-1 text-xs text-muted-foreground">({{ v.description }})</span>
              </label>
              <Input
                v-model="installConfig[v.name]"
                :type="v.secret ? 'password' : 'text'"
                :placeholder="v.default || ''"
                class="h-7 text-sm mt-0.5"
              />
            </div>
          </div>

          <div v-if="installServer.args && installServer.args.length > 0" class="space-y-2 mb-3">
            <div v-for="a in installServer.args" :key="a.valueHint || a.name || ''">
              <label class="text-xs text-muted-foreground">
                {{ a.description }}
                <span v-if="a.required" class="text-red-500">*</span>
              </label>
              <Input
                v-model="installConfig[a.valueHint || a.name || '']"
                :placeholder="a.default || ''"
                class="h-7 text-sm mt-0.5"
              />
            </div>
          </div>

          <div v-if="(!installServer.env_vars || installServer.env_vars.length === 0) && (!installServer.args || installServer.args.length === 0)" class="mb-3 text-xs text-muted-foreground">
            此服务无需额外配置，点击确认即可添加。
          </div>

          <div class="flex justify-end gap-2">
            <Button size="sm" variant="outline" @click="cancelInstall">取消</Button>
            <Button size="sm" @click="confirmInstall">确认添加</Button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- 自定义服务弹窗 -->
    <Teleport to="body">
      <div
        v-if="showCustomDialog"
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
        @click.self="cancelCustom"
      >
        <div class="w-96 rounded-lg border bg-background p-4 shadow-lg">
          <h3 class="text-sm font-medium mb-3">添加自定义 MCP 服务</h3>

          <div class="space-y-2 mb-3">
            <div>
              <label class="text-xs text-muted-foreground">名称</label>
              <Input v-model="customForm.name" placeholder="我的搜索服务" class="h-7 text-sm mt-0.5" />
            </div>
            <div>
              <label class="text-xs text-muted-foreground">命令</label>
              <Input v-model="customForm.command" placeholder="npx" class="h-7 text-sm mt-0.5" />
            </div>
            <div>
              <label class="text-xs text-muted-foreground">参数（空格分隔）</label>
              <Input v-model="customForm.args" placeholder="-y @humansean/mcp-bocha" class="h-7 text-sm mt-0.5" />
            </div>
            <div>
              <label class="text-xs text-muted-foreground">环境变量 Key</label>
              <Input v-model="customForm.envKey" placeholder="BOCHA_API_KEY" class="h-7 text-sm mt-0.5" />
            </div>
            <div>
              <label class="text-xs text-muted-foreground">环境变量 Value</label>
              <Input v-model="customForm.envValue" type="password" placeholder="sk-xxx" class="h-7 text-sm mt-0.5" />
            </div>
          </div>

          <div class="flex justify-end gap-2">
            <Button size="sm" variant="outline" @click="cancelCustom">取消</Button>
            <Button size="sm" @click="confirmCustom">确认添加</Button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>
