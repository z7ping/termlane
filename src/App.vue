<template>
  <div class="h-screen flex flex-col">
    <TitleBar
      @toggle-sidebar="sidebarOpen = !sidebarOpen"
      @toggle-fullscreen="toggleFullscreen"
      @open-settings="viewMode = 'settings'"
    />

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
        <div class="workspace-bar h-9 flex items-center px-2">
          <TabBar
            :tabs="tabs"
            :active-id="activeTabId"
            @select="activeTabId = $event"
            @close="closeTab"
            @close-others="closeOtherTabs"
            @close-all="closeAllTabs"
            @new="openLocalTerminal"
            class="flex-1 min-w-0"
          />
          <ViewSwitcher v-model="viewMode" />
        </div>

        <div class="flex-1 relative overflow-hidden">
          <ErrorBoundary>
            <template v-if="viewMode === 'terminal'">
              <keep-alive>
                <TerminalPanel
                  v-if="activeTab"
                  :key="activeTabId"
                  :tab="activeTab"
                  :active="true"
                  @connected="onSessionConnected(activeTabId, $event)"
                  @disconnected="onSessionDisconnected(activeTabId)"
                />
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
            <Settings v-if="viewMode === 'settings'" @open-proxy-settings="viewMode = 'proxy'" />
          </ErrorBoundary>

          <div v-if="tabs.length === 0 && viewMode === 'terminal'" class="h-full flex items-center justify-center" style="color: var(--fg-muted);">
            <div class="text-center">
              <TerminalIcon :size="52" :stroke-width="1.2" class="mx-auto mb-4" />
              <div class="text-lg" style="color: var(--fg-secondary);">XTerminal Pro</div>
              <div class="text-sm mt-2">从左侧选择一个连接，或按 + 打开本地终端</div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <StatusBar :connection="activeConnection" :session-id="activeSessionId" />
    <Toast ref="toastRef" />
    <UpdateNotifier />

    <ConnectionDialog
      v-if="showAddConnection"
      :editing="editingConnection"
      @save="(conn) => onSaveConnection(conn, editingConnection)"
      @close="showAddConnection = false; editingConnection = null"
    />
    <ShortcutHelp :visible="showShortcuts" @close="showShortcuts = false" />
    <Onboarding />
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, defineAsyncComponent, defineComponent } from 'vue'
import { Terminal as TerminalIcon } from 'lucide-vue-next'
import { parseShortcut, getShortcut } from './utils/shortcuts.js'
import { useAppState, setToast } from './composables/useAppState.js'

// Critical - 首屏必需（同步加载）
import TitleBar from './components/TitleBar.vue'
import Sidebar from './components/Sidebar.vue'
import TabBar from './components/TabBar.vue'
import ViewSwitcher from './components/ViewSwitcher.vue'
import TerminalPanel from './components/TerminalPanel.vue'
import StatusBar from './components/StatusBar.vue'
import Toast from './components/Toast.vue'

// Lazy - 按需加载（异步分包）
const asyncLoadingComponent = defineComponent({ template: '<div class="flex items-center justify-center p-4" style="color: var(--fg-muted);">加载中...</div>' })
const asyncOpts = { loadingComponent: asyncLoadingComponent }
const SftpPanel = defineAsyncComponent(() => import('./components/SftpPanel.vue'), asyncOpts)
const BatchCommand = defineAsyncComponent(() => import('./components/BatchCommand.vue'), asyncOpts)
const ConnectionMonitor = defineAsyncComponent(() => import('./components/ConnectionMonitor.vue'), asyncOpts)
const SpeedTest = defineAsyncComponent(() => import('./components/SpeedTest.vue'), asyncOpts)
const SessionRecorder = defineAsyncComponent(() => import('./components/SessionRecorder.vue'), asyncOpts)
const Notes = defineAsyncComponent(() => import('./components/Notes.vue'), asyncOpts)
const Bookmarks = defineAsyncComponent(() => import('./components/Bookmarks.vue'), asyncOpts)
const ProxyConfig = defineAsyncComponent(() => import('./components/ProxyConfig.vue'), asyncOpts)
const QuickCommands = defineAsyncComponent(() => import('./components/QuickCommands.vue'), asyncOpts)
const PortForward = defineAsyncComponent(() => import('./components/PortForward.vue'), asyncOpts)
const ScheduledTasks = defineAsyncComponent(() => import('./components/ScheduledTasks.vue'), asyncOpts)
const MacroRecorder = defineAsyncComponent(() => import('./components/MacroRecorder.vue'), asyncOpts)
const Settings = defineAsyncComponent(() => import('./components/Settings.vue'), asyncOpts)
const ConnectionDialog = defineAsyncComponent(() => import('./components/ConnectionDialog.vue'), asyncOpts)
const UpdateNotifier = defineAsyncComponent(() => import('./components/UpdateNotifier.vue'), asyncOpts)
const ShortcutHelp = defineAsyncComponent(() => import('./components/ShortcutHelp.vue'), asyncOpts)
const ErrorBoundary = defineAsyncComponent(() => import('./components/ErrorBoundary.vue'), asyncOpts)
const Onboarding = defineAsyncComponent(() => import('./components/Onboarding.vue'), asyncOpts)

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
const sidebarOpen = ref(true)
const showAddConnection = ref(false)
const showShortcuts = ref(false)
const editingConnection = ref(null)
const viewMode = ref('terminal')
const toastRef = ref(null)

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
let _cleanupKeydown = null
let _cleanupShortcutChanged = null
let _cleanupBeforeUnload = null

onMounted(async () => {
  setToast(showToast)
  document.getElementById('app')?.classList.add('ready')

  const savedTheme = localStorage.getItem('xterminal-theme') || 'dark'
  document.documentElement.setAttribute('data-theme', savedTheme)

  await loadConnections()
  loadTabsState()

  const handleBeforeUnload = () => saveTabsState()
  window.addEventListener('beforeunload', handleBeforeUnload)
  _cleanupBeforeUnload = handleBeforeUnload

  const setupGlobalShortcuts = () => {
    const handlers = []

    const newTabKey = parseShortcut(getShortcut('新建标签'))
    handlers.push((e) => { if (newTabKey(e)) { openLocalTerminal(); e.preventDefault() } })

    const closeTabKey = parseShortcut(getShortcut('关闭标签'))
    handlers.push((e) => { if (closeTabKey(e) && activeTabId.value) { closeTab(activeTabId.value); e.preventDefault() } })

    const toggleSidebarKey = parseShortcut(getShortcut('切换侧边栏') || 'Ctrl+B')
    handlers.push((e) => { if (toggleSidebarKey(e)) { sidebarOpen.value = !sidebarOpen.value; e.preventDefault() } })

    const fullscreenKey = parseShortcut(getShortcut('全屏'))
    handlers.push((e) => { if (fullscreenKey(e)) { toggleFullscreen(); e.preventDefault() } })

    const helpKey = parseShortcut(getShortcut('帮助') || '?')
    handlers.push((e) => {
      if (helpKey(e) && !e.ctrlKey && !e.altKey && !['INPUT', 'TEXTAREA'].includes(e.target.tagName)) {
        showShortcuts.value = !showShortcuts.value
      }
    })

    return handlers
  }

  let shortcutHandlers = setupGlobalShortcuts()

  const handleKeydown = (e) => {
    shortcutHandlers.forEach(handler => handler(e))
  }
  document.addEventListener('keydown', handleKeydown)

  const handleShortcutChanged = () => {
    shortcutHandlers = setupGlobalShortcuts()
  }
  window.addEventListener('shortcut-changed', handleShortcutChanged)

  _cleanupKeydown = handleKeydown
  _cleanupShortcutChanged = handleShortcutChanged

  startPingPolling()
})

onUnmounted(() => {
  stopPingPolling()
  if (_cleanupKeydown) document.removeEventListener('keydown', _cleanupKeydown)
  if (_cleanupShortcutChanged) window.removeEventListener('shortcut-changed', _cleanupShortcutChanged)
  if (_cleanupBeforeUnload) window.removeEventListener('beforeunload', _cleanupBeforeUnload)
})
</script>

<style scoped>
.workspace-bar {
  min-width: 0;
  background: var(--bg-surface);
  border-bottom: 1px solid var(--border-subtle);
}
</style>
