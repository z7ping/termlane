<template>
  <div class="h-screen flex flex-col" :class="{ dark: isDark }">
    <TitleBar @toggle-sidebar="sidebarOpen = !sidebarOpen" @toggle-fullscreen="toggleFullscreen" />

    <div class="flex flex-1 overflow-hidden">
      <Sidebar
        v-if="sidebarOpen"
        :connections="connections"
        :active-id="activeConnectionId"
        :latency-map="latencyMap"
        @select="onSelectConnection"
        @add="showAddConnection = true"
        @delete="onDeleteConnection"
        @edit="onEditConnection"
        @duplicate="onDuplicateConnection"
        @test="onTestConnection"
        @quick-command="onQuickCommand"
          @favorite="onToggleFavorite"
      />

      <div class="flex-1 flex flex-col overflow-hidden" style="background: var(--bg-base);">
        <div class="h-9 flex items-center px-2" style="background: var(--bg-surface); border-bottom: 1px solid var(--border-subtle);">
          <TabBar
            :tabs="tabs"
            :active-id="activeTabId"
            @select="activeTabId = $event"
            @close="closeTab"
            @close-others="closeOtherTabs"
            @close-all="closeAllTabs"
            @new="openLocalTerminal"
            class="flex-1"
          />
          <div v-if="activeTab" class="flex gap-1 ml-2">
            <button v-for="vm in viewModes" :key="vm.value" @click="viewMode = vm.value" class="px-2 py-0.5 text-xs rounded-md transition-colors" :style="viewMode === vm.value ? 'background: var(--accent); color: white;' : 'color: var(--fg-muted); hover: background: var(--bg-hover);'" :class="viewMode === vm.value ? '' : 'hover:bg-white/5'">{{ vm.label }}</button>
          </div>
        </div>

        <div class="flex-1 relative overflow-hidden">
          <ErrorBoundary>
            <template v-if="viewMode === 'terminal'">
              <keep-alive>
                <TerminalPanel v-if="activeTab" :key="activeTabId" :tab="activeTab" :active="true" @connected="onSessionConnected(activeTabId, $event)" @disconnected="onSessionDisconnected(activeTabId)" />
              </keep-alive>
            </template>
            <SftpPanel v-if="viewMode === 'sftp'" :connection="activeConnection" :session-id="activeSessionId" :active="true" />
            <BatchCommand v-if="viewMode === 'batch'" />
            <ConnectionMonitor v-if="viewMode === 'monitor'" :connections="connections" :active-session-id="activeSessionId" />
            <SpeedTest v-if="viewMode === 'speed'" />
            <SessionRecorder v-if="viewMode === 'recorder'" :session-id="activeSessionId" :connection-name="activeConnection?.name" />
            <Notes v-if="viewMode === 'notes'" :connection-name="activeConnection?.name" />
            <Bookmarks v-if="viewMode === 'bookmarks'" @navigate="onBookmarkNav" />
            <ProxyConfig v-if="viewMode === 'proxy'" />
            <QuickCommands v-if="viewMode === 'commands'" @run="onQuickCommand" />
            <PortForward v-if="viewMode === 'forward'" />
          <ScheduledTasks v-if="viewMode === 'tasks'" />
          <MacroRecorder v-if="viewMode === 'macro'" :session-id="activeSessionId" />
          </ErrorBoundary>
          <div v-if="tabs.length === 0 && viewMode === 'terminal'" class="h-full flex items-center justify-center text-gray-500">
            <div class="text-center">
              <div class="text-6xl mb-4">⌨️</div>
              <div class="text-lg">XTerminal Pro</div>
              <div class="text-sm mt-2">从左侧选择一个连接，或按 + 打开本地终端</div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <StatusBar :connection="activeConnection" :session-id="activeSessionId" />
    <Toast ref="toastRef" />
    <UpdateNotifier />

    <ConnectionDialog v-if="showAddConnection" :editing="editingConnection" @save="onSaveConnection" @close="showAddConnection = false; editingConnection = null" />
    <ShortcutHelp v-if="showShortcuts" @close="showShortcuts = false" />
    <Onboarding />
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, defineAsyncComponent } from 'vue'
import { invoke } from './utils/tauri.js'
import { parseShortcut, getShortcut } from './utils/shortcuts.js'
import { useAppState, setToast } from './composables/useAppState.js'

// Critical - 首屏必需（同步加载）
import TitleBar from './components/TitleBar.vue'
import Sidebar from './components/Sidebar.vue'
import TabBar from './components/TabBar.vue'
import TerminalPanel from './components/TerminalPanel.vue'
import StatusBar from './components/StatusBar.vue'
import Toast from './components/Toast.vue'

// Lazy - 按需加载（异步分包）
const SftpPanel = defineAsyncComponent(() => import('./components/SftpPanel.vue'))
const BatchCommand = defineAsyncComponent(() => import('./components/BatchCommand.vue'))
const ConnectionMonitor = defineAsyncComponent(() => import('./components/ConnectionMonitor.vue'))
const SpeedTest = defineAsyncComponent(() => import('./components/SpeedTest.vue'))
const SessionRecorder = defineAsyncComponent(() => import('./components/SessionRecorder.vue'))
const Notes = defineAsyncComponent(() => import('./components/Notes.vue'))
const Bookmarks = defineAsyncComponent(() => import('./components/Bookmarks.vue'))
const ProxyConfig = defineAsyncComponent(() => import('./components/ProxyConfig.vue'))
const QuickCommands = defineAsyncComponent(() => import('./components/QuickCommands.vue'))
const PortForward = defineAsyncComponent(() => import('./components/PortForward.vue'))
const ScheduledTasks = defineAsyncComponent(() => import('./components/ScheduledTasks.vue'))
const MacroRecorder = defineAsyncComponent(() => import('./components/MacroRecorder.vue'))
const Settings = defineAsyncComponent(() => import('./components/Settings.vue'))
const ConnectionDialog = defineAsyncComponent(() => import('./components/ConnectionDialog.vue'))
const UpdateNotifier = defineAsyncComponent(() => import('./components/UpdateNotifier.vue'))
const ShortcutHelp = defineAsyncComponent(() => import('./components/ShortcutHelp.vue'))
const ErrorBoundary = defineAsyncComponent(() => import('./components/ErrorBoundary.vue'))
const Onboarding = defineAsyncComponent(() => import('./components/Onboarding.vue'))

// ── Shared state from composable ──
const {
  connections, latencyMap, tabs, activeTabId, sessionMap,
  activeConnectionId, activeTab, activeConnection, activeSessionId,
  loadConnections, loadTabsState, saveTabsState,
  startPingPolling, stopPingPolling,
  onSelectConnection, closeTab, closeOtherTabs, closeAllTabs, openLocalTerminal,
  onSaveConnection, onDeleteConnection, onDuplicateConnection, onTestConnection, onToggleFavorite,
  onSessionConnected, onSessionDisconnected, onQuickCommand,
} = useAppState()

// ── View-only state (stays in App.vue) ──
const isDark = ref(true)
const sidebarOpen = ref(true)
const showAddConnection = ref(false)
const showShortcuts = ref(false)
const editingConnection = ref(null)
const viewMode = ref('terminal')
const toastRef = ref(null)

const viewModes = [
  { value: 'terminal', label: '⌨️ 终端' },
  { value: 'sftp', label: '📁 文件' },
  { value: 'batch', label: '⚡ 批量' },
  { value: 'monitor', label: '📊 监控' },
  { value: 'speed', label: '🚀 测速' },
  { value: 'recorder', label: '⏺ 录制' },
  { value: 'notes', label: '📝 笔记' },
  { value: 'bookmarks', label: '🔖 书签' },
  { value: 'proxy', label: '🌐 代理' },
  { value: 'commands', label: '⚡ 命令' },
  { value: 'forward', label: '🔗 转发' },
  { value: 'tasks', label: '⏰ 定时' },
  { value: 'macro', label: '🎯 宏' },
]

function showToast(msg, type = 'info') { toastRef.value?.show(msg, type) }

function onEditConnection(conn) { editingConnection.value = conn; showAddConnection.value = true }

function onBookmarkNav(bm) {
  viewMode.value = 'sftp'
  showToast(`跳转到: ${bm.path}`, 'info')
}

function toggleFullscreen() {
  if (!document.fullscreenElement) document.documentElement.requestFullscreen()
  else document.exitFullscreen()
}

// ── Lifecycle ──

onMounted(async () => {
  // Wire up toast callback so the composable can show notifications
  setToast(showToast)

  // Mark app as ready to show (prevent FOUC)
  document.getElementById('app')?.classList.add('ready')

  // Restore persisted state
  await loadConnections()
  loadTabsState()

  // Restore window state
  try {
    const ws = await invoke('load_window_state')
    if (ws) {
      // Window state available
    }
  } catch (e) { console.warn("[XTerminal] Load error:", e) }

  // Save window state on close
  window.addEventListener('beforeunload', () => {
    saveTabsState()
    try {
      invoke('save_window_state', {
        x: window.screenX,
        y: window.screenY,
        width: window.outerWidth,
        height: window.outerHeight,
        maximized: false,
      })
    } catch (e) { console.warn("[XTerminal] Load error:", e) }
  })

  // Global keyboard shortcuts (支持自定义快捷键)
  const setupGlobalShortcuts = () => {
    const handlers = []

    // 新建标签
    const newTabKey = parseShortcut(getShortcut('新建标签'))
    handlers.push((e) => { if (newTabKey(e)) { openLocalTerminal(); e.preventDefault() } })

    // 关闭标签
    const closeTabKey = parseShortcut(getShortcut('关闭标签'))
    handlers.push((e) => { if (closeTabKey(e) && activeTabId.value) { closeTab(activeTabId.value); e.preventDefault() } })

    // 切换侧边栏
    const toggleSidebarKey = parseShortcut(getShortcut('切换侧边栏') || 'Ctrl+B')
    handlers.push((e) => { if (toggleSidebarKey(e)) { sidebarOpen.value = !sidebarOpen.value; e.preventDefault() } })

    // 全屏
    const fullscreenKey = parseShortcut(getShortcut('全屏'))
    handlers.push((e) => { if (fullscreenKey(e)) { toggleFullscreen(); e.preventDefault() } })

    // 帮助
    const helpKey = parseShortcut(getShortcut('帮助') || '?')
    handlers.push((e) => { if (helpKey(e) && !e.ctrlKey && !e.altKey && !['INPUT','TEXTAREA'].includes(e.target.tagName)) { showShortcuts.value = !showShortcuts.value } })

    return handlers
  }

  let shortcutHandlers = setupGlobalShortcuts()
  document.addEventListener('keydown', (e) => {
    shortcutHandlers.forEach(handler => handler(e))
  })

  // 监听快捷键变化
  window.addEventListener('shortcut-changed', () => {
    shortcutHandlers = setupGlobalShortcuts()
  })

  // Start latency polling
  startPingPolling()
})

onUnmounted(() => {
  stopPingPolling()
})
</script>
