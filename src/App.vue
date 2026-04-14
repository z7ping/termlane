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
            <button v-for="vm in viewModes" :key="vm.value" @click="viewMode = vm.value" class="px-2 py-0.5 text-[10px] rounded-md transition-colors" :style="viewMode === vm.value ? 'background: var(--accent); color: white;' : 'color: var(--fg-muted); hover: background: var(--bg-hover);'" :class="viewMode === vm.value ? '' : 'hover:bg-white/5'">{{ vm.label }}</button>
          </div>
        </div>

        <div class="flex-1 relative overflow-hidden">
          <ErrorBoundary>
            <template v-if="viewMode === 'terminal'">
              <TerminalPanel v-for="tab in tabs" :key="tab.id" :tab="tab" :active="tab.id === activeTabId" @connected="onSessionConnected(tab.id, $event)" @disconnected="onSessionDisconnected(tab.id)" />
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
import { ref, computed, onMounted, inject, defineAsyncComponent } from 'vue'
import { invoke } from './utils/tauri.js'

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

const connections = ref([
  { id: 'local', name: '本地终端', host: 'localhost', port: 22, username: 'local', authType: 'local', group: '本地', icon: '💻' },
])

const latencyMap = ref({})

const tabs = ref([])
const activeTabId = ref(null)
const sessionMap = ref({})

const activeTab = computed(() => tabs.value.find(t => t.id === activeTabId.value))
const activeConnectionId = ref(null)
const activeConnection = computed(() => connections.value.find(c => c.id === activeConnectionId.value))
const activeSessionId = computed(() => sessionMap.value[activeTabId.value] || null)

function showToast(msg, type = 'info') { toastRef.value?.show(msg, type) }

onMounted(async () => {
  // Mark app as ready to show (prevent FOUC)
  document.getElementById('app')?.classList.add('ready')

  // Restore connections
  try {
    const saved = await invoke('load_connections')
    if (saved?.length > 0) {
      const localExists = saved.some(c => c.id === 'local')
      connections.value = localExists ? saved : [{ id: 'local', name: '本地终端', host: 'localhost', port: 22, username: 'local', authType: 'local', group: '本地', icon: '💻' }, ...saved]
    }
  } catch {}

  // Restore tabs
  try {
    const savedTabs = JSON.parse(localStorage.getItem('xterminal_tabs') || '[]')
    if (savedTabs.length > 0) {
      tabs.value = savedTabs
      activeTabId.value = localStorage.getItem('xterminal_active_tab') || savedTabs[0]?.id
    }
  } catch {}

  // Restore window state
  try {
    const ws = await invoke('load_window_state')
    if (ws) {
      // Window state available
    }
  } catch {}

  // Save window state on close
  window.addEventListener('beforeunload', () => {
    saveTabsState()
    // Save window state (only works in Tauri)
    try {
      invoke('save_window_state', {
        x: window.screenX,
        y: window.screenY,
        width: window.outerWidth,
        height: window.outerHeight,
        maximized: false,
      })
    } catch {}
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

function saveTabsState() {
  localStorage.setItem('xterminal_tabs', JSON.stringify(tabs.value.map(t => ({ id: t.id, name: t.name, connectionId: t.connectionId, connection: t.connection }))))
  localStorage.setItem('xterminal_active_tab', activeTabId.value || '')
}

function onSelectConnection(conn) {
  activeConnectionId.value = conn.id
  const existing = tabs.value.find(t => t.connectionId === conn.id)
  if (existing) {
    activeTabId.value = existing.id
  } else {
    const tab = { id: `tab_${Date.now()}`, name: conn.name, connectionId: conn.id, connection: conn, type: 'terminal' }
    tabs.value.push(tab)
    activeTabId.value = tab.id
  }
  saveTabsState()
}

function closeTab(tabId) {
  const sid = sessionMap.value[tabId]
  if (sid) { invoke('ssh_disconnect', { sessionId: sid }).catch(() => {}); delete sessionMap.value[tabId] }
  const idx = tabs.value.findIndex(t => t.id === tabId)
  tabs.value.splice(idx, 1)
  if (activeTabId.value === tabId) activeTabId.value = tabs.value[Math.min(idx, tabs.value.length - 1)]?.id || null
  saveTabsState()
}

function closeOtherTabs(keepId) {
  tabs.value.filter(t => t.id !== keepId).forEach(t => closeTab(t.id))
}

function closeAllTabs() { [...tabs.value].forEach(t => closeTab(t.id)) }

function openLocalTerminal() { const local = connections.value.find(c => c.id === 'local'); if (local) onSelectConnection(local) }

function onSaveConnection(conn) {
  const newConn = editingConnection.value ? { ...editingConnection.value, ...conn } : { ...conn, id: `conn_${Date.now()}` }
  if (editingConnection.value) {
    const idx = connections.value.findIndex(c => c.id === editingConnection.value.id)
    if (idx >= 0) connections.value[idx] = newConn
  } else {
    connections.value.push(newConn)
  }
  showAddConnection.value = false; editingConnection.value = null
  invoke('save_connection', { conn: newConn }).catch(() => {})
  showToast('连接已保存', 'success')
}

function onDeleteConnection(id) {
  connections.value = connections.value.filter(c => c.id !== id)
  tabs.value.filter(t => t.connectionId === id).forEach(t => closeTab(t.id))
  invoke('delete_connection', { id }).catch(() => {})
  showToast('已删除', 'info')
}

function onEditConnection(conn) { editingConnection.value = conn; showAddConnection.value = true }

function onDuplicateConnection(conn) {
  const dup = { ...conn, id: `conn_${Date.now()}`, name: conn.name + ' (副本)' }
  connections.value.push(dup)
  invoke('save_connection', { conn: dup }).catch(() => {})
  showToast('已复制', 'success')
}

async function onTestConnection(conn) {
  showToast('正在测试连接...', 'info')
  try {
    let sid
    if (conn.authType === 'key') {
      sid = await invoke('ssh_connect_key', { host: conn.host, port: conn.port || 22, username: conn.username, keyPath: conn.keyPath || '', passphrase: conn.passphrase || '' })
    } else {
      sid = await invoke('ssh_connect', { host: conn.host, port: conn.port || 22, username: conn.username, password: conn.password || '' })
    }
    showToast(`✓ ${conn.name} 连接成功`, 'success')
    await invoke('ssh_disconnect', { sessionId: sid })
  } catch (err) {
    showToast(`✗ ${conn.name} 连接失败: ${err}`, 'error')
  }
}

function onQuickCommand(cmd) {
  // Send to active terminal session
  if (activeSessionId.value) {
    invoke('ssh_shell_input', { sessionId: activeSessionId.value, data: cmd + '\r' }).catch(() => {})
    viewMode.value = 'terminal'
  } else {
    showToast(`请先连接服务器`, 'info')
  }
}

function onBookmarkNav(bm) {
  viewMode.value = 'sftp'
  showToast(`跳转到: ${bm.path}`, 'info')
}

function onToggleFavorite(conn) {
  const idx = connections.value.findIndex(c => c.id === conn.id)
  if (idx >= 0) {
    connections.value[idx].favorite = !connections.value[idx].favorite
    invoke('save_connection', { conn: connections.value[idx] }).catch(() => {})
    showToast(connections.value[idx].favorite ? '已收藏' : '已取消收藏', 'success')
  }
}

function toggleFullscreen() {
  if (!document.fullscreenElement) document.documentElement.requestFullscreen()
  else document.exitFullscreen()
}

function onSessionConnected(tabId, sid) { sessionMap.value[tabId] = sid; saveTabsState() }
function onSessionDisconnected(tabId) { delete sessionMap.value[tabId] }

// 快捷键工具函数
function getShortcut(actionName) {
  return localStorage.getItem(`shortcut_${actionName}`) || {
    '搜索': 'Ctrl+Shift+F',
    '清屏': 'Ctrl+L',
    '中断': 'Ctrl+C',
    '新建标签': 'Ctrl+T',
    '关闭标签': 'Ctrl+W',
    '分屏': 'Ctrl+Shift+D',
    '全屏': 'F11',
  }[actionName]
}

// 解析快捷键字符串为按键检测函数
function parseShortcut(keyStr) {
  const parts = keyStr.toLowerCase().split('+')
  const ctrl = parts.includes('ctrl')
  const alt = parts.includes('alt')
  const shift = parts.includes('shift')
  const meta = parts.includes('meta')
  const key = parts.filter(p => !['ctrl', 'alt', 'shift', 'meta'].includes(p)).pop()?.toUpperCase() || ''
  return (e) => {
    if (e.ctrlKey !== ctrl) return false
    if (e.altKey !== alt) return false
    if (e.shiftKey !== shift) return false
    if (e.metaKey !== meta) return false
    return e.key.toUpperCase() === key
  }
}

// Latency polling
async function pingConnections() {
  const targets = connections.value.filter(c => c.host && c.host !== 'localhost' && c.host !== '127.0.0.1')
  for (const conn of targets) {
    try {
      const ms = await invoke('tcp_ping', { host: conn.host, port: conn.port || 22 })
      latencyMap.value[conn.id] = ms
    } catch {
      latencyMap.value[conn.id] = null
    }
  }
}
let pingTimer = null
function startPingPolling() {
  pingConnections()
  pingTimer = setInterval(pingConnections, 30000)
}
</script>
