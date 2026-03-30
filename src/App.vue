<template>
  <div class="h-screen flex flex-col" :class="{ dark: isDark }">
    <TitleBar @toggle-sidebar="sidebarOpen = !sidebarOpen" @toggle-fullscreen="toggleFullscreen" />

    <div class="flex flex-1 overflow-hidden">
      <Sidebar
        v-if="sidebarOpen"
        :connections="connections"
        :active-id="activeConnectionId"
        @select="onSelectConnection"
        @add="showAddConnection = true"
        @delete="onDeleteConnection"
        @edit="onEditConnection"
        @duplicate="onDuplicateConnection"
        @test="onTestConnection"
        @quick-command="onQuickCommand"
      />

      <div class="flex-1 flex flex-col overflow-hidden bg-gray-900">
        <div class="h-9 bg-gray-800 border-b border-gray-700 flex items-center px-2">
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
            <button v-for="vm in viewModes" :key="vm.value" @click="viewMode = vm.value" class="px-2 py-0.5 text-xs rounded" :class="viewMode === vm.value ? 'bg-blue-600 text-white' : 'bg-gray-700 text-gray-400 hover:bg-gray-600'">{{ vm.label }}</button>
          </div>
        </div>

        <div class="flex-1 relative overflow-hidden">
          <template v-if="viewMode === 'terminal'">
            <TerminalPanel v-for="tab in tabs" :key="tab.id" :tab="tab" :active="tab.id === activeTabId" @connected="onSessionConnected(tab.id, $event)" @disconnected="onSessionDisconnected(tab.id)" />
          </template>
          <SftpPanel v-if="viewMode === 'sftp'" :connection="activeConnection" :session-id="activeSessionId" :active="true" />
          <BatchCommand v-if="viewMode === 'batch'" />
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
  </div>
</template>

<script setup>
import { ref, computed, onMounted, inject } from 'vue'
import { invoke } from './utils/tauri.js'
import TitleBar from './components/TitleBar.vue'
import Sidebar from './components/Sidebar.vue'
import TabBar from './components/TabBar.vue'
import TerminalPanel from './components/TerminalPanel.vue'
import SftpPanel from './components/SftpPanel.vue'
import BatchCommand from './components/BatchCommand.vue'
import StatusBar from './components/StatusBar.vue'
import ConnectionDialog from './components/ConnectionDialog.vue'
import Toast from './components/Toast.vue'
import UpdateNotifier from './components/UpdateNotifier.vue'
import ShortcutHelp from './components/ShortcutHelp.vue'

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
]

const connections = ref([
  { id: 'local', name: '本地终端', host: 'localhost', port: 22, username: 'local', authType: 'local', group: '本地', icon: '💻' },
])

const tabs = ref([])
const activeTabId = ref(null)
const sessionMap = ref({})

const activeTab = computed(() => tabs.value.find(t => t.id === activeTabId.value))
const activeConnectionId = ref(null)
const activeConnection = computed(() => connections.value.find(c => c.id === activeConnectionId.value))
const activeSessionId = computed(() => sessionMap.value[activeTabId.value] || null)

function showToast(msg, type = 'info') { toastRef.value?.show(msg, type) }

onMounted(async () => {
  try {
    const saved = await invoke('load_connections')
    if (saved?.length > 0) {
      const localExists = saved.some(c => c.id === 'local')
      connections.value = localExists ? saved : [{ id: 'local', name: '本地终端', host: 'localhost', port: 22, username: 'local', authType: 'local', group: '本地', icon: '💻' }, ...saved]
    }
  } catch {}

  // Restore last session tabs
  try {
    const savedTabs = JSON.parse(localStorage.getItem('xterminal_tabs') || '[]')
    if (savedTabs.length > 0) {
      tabs.value = savedTabs
      activeTabId.value = localStorage.getItem('xterminal_active_tab') || savedTabs[0]?.id
    }
  } catch {}

  // Global keyboard shortcuts
  document.addEventListener('keydown', (e) => {
    if (e.ctrlKey && e.key === 't') { openLocalTerminal(); e.preventDefault() }
    if (e.ctrlKey && e.key === 'w' && activeTabId.value) { closeTab(activeTabId.value); e.preventDefault() }
    if (e.ctrlKey && e.key === 'b') { sidebarOpen.value = !sidebarOpen.value; e.preventDefault() }
    if (e.key === 'F11') { toggleFullscreen(); e.preventDefault() }
    if (e.key === '?' && !e.ctrlKey && !e.altKey) { showShortcuts.value = !showShortcuts.value }
    if (e.ctrlKey && e.key === ',') { /* Settings - not implemented yet */ }
  })
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
  // TODO: Send to active terminal
  showToast(`执行: ${cmd}`, 'info')
}

function toggleFullscreen() {
  if (!document.fullscreenElement) document.documentElement.requestFullscreen()
  else document.exitFullscreen()
}

function onSessionConnected(tabId, sid) { sessionMap.value[tabId] = sid; saveTabsState() }
function onSessionDisconnected(tabId) { delete sessionMap.value[tabId] }
</script>
