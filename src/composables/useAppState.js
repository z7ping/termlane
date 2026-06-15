import { STORAGE_KEYS } from '@/utils/storage-keys.js'
import { ref, computed } from 'vue'
import { invoke } from '../utils/tauri.js'
import { safeInvoke } from '../utils/invoke.js'

// Local terminal connection template (always present)
const LOCAL_CONN = { id: 'local', name: '本地终端', host: 'localhost', port: 22, username: 'local', authType: 'local', group: '本地', icon: '💻' }

// Singleton state — shared across all components that call useAppState()
const connections = ref([LOCAL_CONN])
const latencyMap = ref({})
const tabs = ref([])
const activeTabId = ref(null)
const sessionMap = ref({})
const activeConnectionId = ref(null)

const activeTab = computed(() => tabs.value.find(t => t.id === activeTabId.value))
const activeConnection = computed(() => connections.value.find(c => c.id === activeConnectionId.value))
const activeSessionId = computed(() => sessionMap.value[activeTabId.value] || null)

// ---- Callbacks (set by the host component) ----
let _toastFn = null

/**
 * Set the toast notification callback. Call this from the host component.
 * @param {(msg: string, type?: string) => void} fn
 */
export function setToast(fn) { _toastFn = fn }
function _toast(msg, type = 'info') { _toastFn?.(msg, type) }

// ---- Persistence helpers ----

function saveTabsState() {
  localStorage.setItem(STORAGE_KEYS.TABS, JSON.stringify(tabs.value.map(t => ({ id: t.id, name: t.name, connectionId: t.connectionId, connection: t.connection }))))
  localStorage.setItem(STORAGE_KEYS.ACTIVE_TAB, activeTabId.value || '')
}

async function loadConnections() {
  try {
    const saved = await invoke('load_connections')
    if (saved?.length > 0) {
      const localExists = saved.some(c => c.id === 'local')
      connections.value = localExists ? saved : [LOCAL_CONN, ...saved]
    }
  } catch (e) { console.warn('[XTerminal] Load connections error:', e) }
}

function loadTabsState() {
  try {
    const savedTabs = JSON.parse(localStorage.getItem(STORAGE_KEYS.TABS) || '[]')
    if (savedTabs.length > 0) {
      tabs.value = savedTabs
      activeTabId.value = localStorage.getItem(STORAGE_KEYS.ACTIVE_TAB) || savedTabs[0]?.id
    }
  } catch (e) { console.warn('[XTerminal] Load tabs error:', e) }
}

// ---- Tab management ----

async function onSelectConnection(conn) {
  // Load password from keyring if not present in the connection object
  let password = conn.password
  if (conn.authType === 'password' && !password && conn.id) {
    try { password = await invoke('keyring_load_password', { connId: conn.id }) } catch (e) {}
  }
  const connWithPassword = password ? { ...conn, password } : conn
  activeConnectionId.value = connWithPassword.id
  const existing = tabs.value.find(t => t.connectionId === connWithPassword.id)
  if (existing) {
    activeTabId.value = existing.id
  } else {
    const tab = { id: `tab_${Date.now()}`, name: connWithPassword.name, connectionId: connWithPassword.id, connection: connWithPassword, type: 'terminal' }
    tabs.value.push(tab)
    activeTabId.value = tab.id
  }
  saveTabsState()
}

function closeTab(tabId) {
  const sid = sessionMap.value[tabId]
  if (sid) { safeInvoke('ssh_disconnect', { sessionId: sid }).catch(() => {}); delete sessionMap.value[tabId] }
  const idx = tabs.value.findIndex(t => t.id === tabId)
  tabs.value.splice(idx, 1)
  if (activeTabId.value === tabId) activeTabId.value = tabs.value[Math.min(idx, tabs.value.length - 1)]?.id || null
  saveTabsState()
}

function closeOtherTabs(keepId) {
  tabs.value.filter(t => t.id !== keepId).forEach(t => closeTab(t.id))
}

function closeAllTabs() { [...tabs.value].forEach(t => closeTab(t.id)) }

function openLocalTerminal() {
  const local = connections.value.find(c => c.id === 'local')
  if (local) onSelectConnection(local)
}

// ---- Connection management ----

async function onSaveConnection(conn, editingConn) {
  const newConn = editingConn ? { ...editingConn, ...conn } : { ...conn, id: `conn_${Date.now()}` }
  try {
    if (conn.authType === 'password' && conn.password) {
      await invoke('keyring_save_password', { connId: newConn.id, password: conn.password })
    }
    await invoke('save_connection', { conn: newConn })
    await loadConnections()
    _toast('连接已保存', 'success')
  } catch (err) {
    _toast('保存失败: ' + err, 'error')
  }
}

function onDeleteConnection(id) {
  connections.value = connections.value.filter(c => c.id !== id)
  tabs.value.filter(t => t.connectionId === id).forEach(t => closeTab(t.id))
  safeInvoke('delete_connection', { id }).catch(() => {})
  _toast('已删除', 'info')
}

function onDuplicateConnection(conn) {
  const dup = { ...conn, id: `conn_${Date.now()}`, name: conn.name + ' (副本)' }
  connections.value.push(dup)
  safeInvoke('save_connection', { conn: dup }).catch(() => {})
  _toast('已复制', 'success')
}

async function onTestConnection(conn) {
  _toast('正在测试连接...', 'info')
  try {
    let sid
    if (conn.authType === 'key') {
      sid = await invoke('ssh_connect_key', { host: conn.host, port: conn.port || 22, username: conn.username, keyPath: conn.keyPath || '', passphrase: conn.passphrase || '' })
    } else {
      sid = await invoke('ssh_connect', { host: conn.host, port: conn.port || 22, username: conn.username, password: conn.password || '' })
    }
    _toast(`✓ ${conn.name} 连接成功`, 'success')
    await invoke('ssh_disconnect', { sessionId: sid })
  } catch (err) {
    _toast(`✗ ${conn.name} 连接失败: ${err}`, 'error')
  }
}

function onToggleFavorite(conn) {
  const idx = connections.value.findIndex(c => c.id === conn.id)
  if (idx >= 0) {
    connections.value[idx].favorite = !connections.value[idx].favorite
    safeInvoke('save_connection', { conn: connections.value[idx] }).catch(() => {})
    _toast(connections.value[idx].favorite ? '已收藏' : '已取消收藏', 'success')
  }
}

// ---- Session management ----

function onSessionConnected(tabId, sid) {
  sessionMap.value[tabId] = sid
  saveTabsState()
}

function onSessionDisconnected(tabId) {
  delete sessionMap.value[tabId]
}

function onQuickCommand(cmd) {
  if (activeSessionId.value) {
    safeInvoke('ssh_shell_input', { sessionId: activeSessionId.value, data: cmd + '\r' }).catch(() => {})
  } else {
    _toast('请先连接服务器', 'info')
  }
}

// ---- Latency polling ----

let pingTimer = null

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

function startPingPolling() {
  pingConnections()
  pingTimer = setInterval(pingConnections, 30000)
}

function stopPingPolling() {
  if (pingTimer) { clearInterval(pingTimer); pingTimer = null }
}

// ---- Composable entry point ----

export function useAppState() {
  return {
    // Reactive state
    connections,
    latencyMap,
    tabs,
    activeTabId,
    sessionMap,
    activeConnectionId,
    activeTab,
    activeConnection,
    activeSessionId,

    // Initialization
    loadConnections,
    loadTabsState,
    saveTabsState,
    startPingPolling,
    stopPingPolling,

    // Tab actions
    onSelectConnection,
    closeTab,
    closeOtherTabs,
    closeAllTabs,
    openLocalTerminal,

    // Connection actions
    onSaveConnection,
    onDeleteConnection,
    onDuplicateConnection,
    onTestConnection,
    onToggleFavorite,

    // Session actions
    onSessionConnected,
    onSessionDisconnected,
    onQuickCommand,
  }
}
