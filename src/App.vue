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
            class="flex-1 min-w-0"
            @select="activeTabId = $event"
            @close="closeTab"
            @close-others="closeOtherTabs"
            @close-all="closeAllTabs"
            @new="openLocalTerminal"
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
            <SftpPanel
              v-if="viewMode === 'sftp'"
              :connection="activeConnection"
              :session-id="activeSessionId"
              :active="true"
              :navigation-target="bookmarkTarget"
            />
            <SpeedTest v-if="viewMode === 'speed'" />
            <SessionRecorder
              v-if="viewMode === 'recorder'"
              :session-id="activeSessionId"
              :connection-name="activeConnection?.name"
              :is-local="isActiveLocal"
            />
            <Notes v-if="viewMode === 'notes'" :connection-name="activeConnection?.name" />
            <Bookmarks v-if="viewMode === 'bookmarks'" @navigate="onBookmarkNav" />
            <QuickCommands v-if="viewMode === 'commands'" @run="onQuickCommand" />
            <MacroRecorder
              v-if="viewMode === 'macro'"
              :session-id="activeSessionId"
              :is-local="isActiveLocal"
              @status="showToast($event, 'info')"
            />
            <Settings v-if="viewMode === 'settings'" />
          </ErrorBoundary>

          <div v-if="tabs.length === 0 && viewMode === 'terminal'" class="h-full flex items-center justify-center" style="color: var(--fg-muted);">
            <div class="text-center">
              <TerminalIcon :size="52" :stroke-width="1.2" class="mx-auto mb-4" />
              <div class="text-lg" style="color: var(--fg-secondary);">Termlane</div>
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
      @save="conn => onSaveConnection(conn, editingConnection)"
      @close="showAddConnection = false; editingConnection = null"
    />
    <ShortcutHelp :visible="showShortcuts" @close="showShortcuts = false" />
    <Onboarding />
  </div>
</template>

<script setup>
import { computed, defineAsyncComponent, defineComponent, onMounted, onUnmounted, ref } from 'vue'
import { Terminal as TerminalIcon } from 'lucide-vue-next'
import { getShortcut, parseShortcut } from './utils/shortcuts.js'
import { applyTheme, getStoredTheme } from './utils/theme-state'
import { setToast, useAppState } from './composables/useAppState.js'

import TitleBar from './components/TitleBar.vue'
import Sidebar from './components/Sidebar.vue'
import TabBar from './components/TabBar.vue'
import ViewSwitcher from './components/ViewSwitcher.vue'
import TerminalPanel from './components/TerminalPanel.vue'
import StatusBar from './components/StatusBar.vue'
import Toast from './components/Toast.vue'

const asyncLoadingComponent = defineComponent({ template: '<div class="flex items-center justify-center p-4" style="color: var(--fg-muted);">加载中...</div>' })
const asyncOpts = { loadingComponent: asyncLoadingComponent }
const SftpPanel = defineAsyncComponent(() => import('./components/SftpPanel.vue'), asyncOpts)
const SpeedTest = defineAsyncComponent(() => import('./components/SpeedTest.vue'), asyncOpts)
const SessionRecorder = defineAsyncComponent(() => import('./components/SessionRecorder.vue'), asyncOpts)
const Notes = defineAsyncComponent(() => import('./components/Notes.vue'), asyncOpts)
const Bookmarks = defineAsyncComponent(() => import('./components/Bookmarks.vue'), asyncOpts)
const QuickCommands = defineAsyncComponent(() => import('./components/QuickCommands.vue'), asyncOpts)
const MacroRecorder = defineAsyncComponent(() => import('./components/MacroRecorder.vue'), asyncOpts)
const Settings = defineAsyncComponent(() => import('./components/Settings.vue'), asyncOpts)
const ConnectionDialog = defineAsyncComponent(() => import('./components/ConnectionDialog.vue'), asyncOpts)
const UpdateNotifier = defineAsyncComponent(() => import('./components/UpdateNotifier.vue'), asyncOpts)
const ShortcutHelp = defineAsyncComponent(() => import('./components/ShortcutHelp.vue'), asyncOpts)
const ErrorBoundary = defineAsyncComponent(() => import('./components/ErrorBoundary.vue'), asyncOpts)
const Onboarding = defineAsyncComponent(() => import('./components/Onboarding.vue'), asyncOpts)

const {
  connections, latencyMap, tabs, activeTabId,
  activeConnectionId, activeTab, activeConnection, activeSessionId,
  loadConnections, loadTabsState, saveTabsState,
  startPingPolling, stopPingPolling,
  onSelectConnection, closeTab, closeOtherTabs, closeAllTabs, openLocalTerminal,
  onSaveConnection, onDeleteConnection, onDuplicateConnection, onTestConnection, onToggleFavorite,
  onSessionConnected, onSessionDisconnected, onQuickCommand,
} = useAppState()

const sidebarOpen = ref(true)
const showAddConnection = ref(false)
const showShortcuts = ref(false)
const editingConnection = ref(null)
const viewMode = ref('terminal')
const toastRef = ref(null)
const bookmarkTarget = ref(null)

const isActiveLocal = computed(() => {
  const connection = activeConnection.value
  return !connection || connection.host === 'localhost' || connection.host === '127.0.0.1'
})

function showToast(message, type = 'info') {
  toastRef.value?.show(message, type)
}

function onEditConnection(connection) {
  editingConnection.value = connection
  showAddConnection.value = true
}

function onBookmarkNav(bookmark) {
  if (!bookmark?.path) return

  const targetHost = bookmark.host?.trim()
  if (targetHost) {
    const targetConnection = connections.value.find(connection =>
      connection.name === targetHost || connection.host === targetHost,
    )
    if (!targetConnection) {
      showToast(`未找到书签对应连接：${targetHost}`, 'error')
      return
    }
    if (activeConnectionId.value !== targetConnection.id) {
      onSelectConnection(targetConnection)
    }
  }

  bookmarkTarget.value = {
    path: bookmark.path,
    requestId: `${Date.now()}-${Math.random()}`,
  }
  viewMode.value = 'sftp'
}

function toggleFullscreen() {
  if (!document.fullscreenElement) document.documentElement.requestFullscreen()
  else document.exitFullscreen()
}

let cleanupKeydown = null
let cleanupShortcutChanged = null
let cleanupBeforeUnload = null

onMounted(async () => {
  setToast(showToast)
  document.getElementById('app')?.classList.add('ready')
  applyTheme(getStoredTheme())

  await loadConnections()
  loadTabsState()

  const handleBeforeUnload = () => saveTabsState()
  window.addEventListener('beforeunload', handleBeforeUnload)
  cleanupBeforeUnload = handleBeforeUnload

  const setupGlobalShortcuts = () => {
    const handlers = []

    const newTabKey = parseShortcut(getShortcut('新建标签'))
    handlers.push(event => {
      if (!newTabKey(event)) return
      openLocalTerminal()
      event.preventDefault()
    })

    const closeTabKey = parseShortcut(getShortcut('关闭标签'))
    handlers.push(event => {
      if (!closeTabKey(event) || !activeTabId.value) return
      closeTab(activeTabId.value)
      event.preventDefault()
    })

    const toggleSidebarKey = parseShortcut(getShortcut('切换侧边栏') || 'Ctrl+B')
    handlers.push(event => {
      if (!toggleSidebarKey(event)) return
      sidebarOpen.value = !sidebarOpen.value
      event.preventDefault()
    })

    const settingsKey = parseShortcut(getShortcut('设置'))
    handlers.push(event => {
      if (!settingsKey(event)) return
      viewMode.value = 'settings'
      event.preventDefault()
    })

    const fullscreenKey = parseShortcut(getShortcut('全屏'))
    handlers.push(event => {
      if (!fullscreenKey(event)) return
      toggleFullscreen()
      event.preventDefault()
    })

    const helpKey = parseShortcut(getShortcut('帮助') || '?')
    handlers.push(event => {
      const tagName = event.target?.tagName
      if (helpKey(event) && !event.ctrlKey && !event.altKey && !['INPUT', 'TEXTAREA'].includes(tagName)) {
        showShortcuts.value = !showShortcuts.value
      }
    })

    return handlers
  }

  let shortcutHandlers = setupGlobalShortcuts()

  const handleKeydown = event => {
    shortcutHandlers.forEach(handler => handler(event))
  }
  document.addEventListener('keydown', handleKeydown)

  const handleShortcutChanged = () => {
    shortcutHandlers = setupGlobalShortcuts()
  }
  window.addEventListener('shortcut-changed', handleShortcutChanged)

  cleanupKeydown = handleKeydown
  cleanupShortcutChanged = handleShortcutChanged

  startPingPolling()
})

onUnmounted(() => {
  stopPingPolling()
  if (cleanupKeydown) document.removeEventListener('keydown', cleanupKeydown)
  if (cleanupShortcutChanged) window.removeEventListener('shortcut-changed', cleanupShortcutChanged)
  if (cleanupBeforeUnload) window.removeEventListener('beforeunload', cleanupBeforeUnload)
})
</script>

<style scoped>
.workspace-bar {
  min-width: 0;
  background: var(--bg-surface);
  border-bottom: 1px solid var(--border-subtle);
}
</style>
