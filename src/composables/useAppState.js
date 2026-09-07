import { STORAGE_KEYS } from '@/utils/storage-keys.js'
import { ref, computed } from 'vue'
import { invoke } from '../utils/tauri.js'
import { safeInvoke } from '../utils/invoke.js'
import {
  deleteCredential,
  loadCredential,
  migrateLegacyPlaintextCredential,
  saveCredential,
  stripConnectionSecrets,
} from '../utils/credentials.js'
import { parseHostKeyError } from '../utils/ssh-host-key.js'

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

function persistentTab(tab) {
  return {
    id: tab.id,
    name: tab.name,
    connectionId: tab.connectionId,
    connection: stripConnectionSecrets(tab.connection || {}),
    type: tab.type || 'terminal',
  }
}

function saveTabsState() {
  localStorage.setItem(STORAGE_KEYS.TABS, JSON.stringify(tabs.value.map(persistentTab)))
  localStorage.setItem(STORAGE_KEYS.ACTIVE_TAB, activeTabId.value || '')
}

async function loadConnections() {
  try {
    const saved = await invoke('load_connections')
    if (saved?.length > 0) {
      // One-time migration of the historical plaintext xterminal-pwd_* keys.
      for (const conn of saved) {
        if (!conn?.id || conn.id === 'local') continue
        try {
          await migrateLegacyPlaintextCredential(conn.id)
        } catch (e) {
          console.warn(`[XTerminal] Credential migration failed for ${conn.id}:`, e)
        }
      }

      const localExists = saved.some(c => c.id === 'local')
      connections.value = localExists ? saved : [LOCAL_CONN, ...saved]
    }
  } catch (e) { console.warn('[XTerminal] Load connections error:', e) }
}

function loadTabsState() {
  try {
    const savedTabs = JSON.parse(localStorage.getItem(STORAGE_KEYS.TABS) || '[]')
    if (savedTabs.length > 0) {
      // Strip secrets from legacy tab snapshots immediately and rewrite the safe form.
      tabs.value = savedTabs.map(tab => ({
        ...tab,
        connection: stripConnectionSecrets(tab.connection || {}),
      }))
      activeTabId.value = localStorage.getItem(STORAGE_KEYS.ACTIVE_TAB) || savedTabs[0]?.id
      saveTabsState()
    }
  } catch (e) { console.warn('[XTerminal] Load tabs error:', e) }
}

// ---- Tab management ----

async function onSelectConnection(conn) {
  const safeConnection = stripConnectionSecrets(conn)
  activeConnectionId.value = safeConnection.id
  const existing = tabs.value.find(t => t.connectionId === safeConnection.id)
  if (existing) {
    // Refresh non-secret connection metadata in case the connection was edited.
    existing.name = safeConnection.name
    existing.connection = safeConnection
    activeTabId.value = existing.id
  } else {
    const tab = {
      id: `tab_${Date.now()}`,
      name: safeConnection.name,
      connectionId: safeConnection.id,
      connection: safeConnection,
      type: 'terminal',
    }
    tabs.value.push(tab)
    activeTabId.value = tab.id
  }
  saveTabsState()
}

function closeTab(tabId) {
  const sid = sessionMap.value[tabId]
  if (sid) {
    const tab = tabs.value.find(t => t.id === tabId)
    const isLocal = !tab?.connection || tab.connection.host === 'localhost'
    safeInvoke(isLocal ? 'local_close_shell' : 'ssh_close_shell', { sessionId: sid }).catch(() => {})
    delete sessionMap.value[tabId]
  }
  const idx = tabs.value.findIndex(t => t.id === tabId)
  if (idx < 0) return
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
  const id = editingConn?.id || conn.id || `conn_${Date.now()}`
  const merged = editingConn ? { ...editingConn, ...conn, id } : { ...conn, id }
  const safeConnection = stripConnectionSecrets(merged)

  try {
    if (conn.authType === 'password' && conn.password) {
      await saveCredential(id, conn.password)
    }
    await invoke('save_connection', { conn: safeConnection })
    await loadConnections()
    _toast('连接已保存', 'success')
  } catch (err) {
    _toast('保存失败: ' + err, 'error')
  }
}

async function onDeleteConnection(id) {
  connections.value = connections.value.filter(c => c.id !== id)
  tabs.value.filter(t => t.connectionId === id).forEach(t => closeTab(t.id))
  await deleteCredential(id).catch(() => {})
  await safeInvoke('delete_connection', { id }).catch(() => {})
  _toast('已删除', 'info')
}

function onDuplicateConnection(conn) {
  const dup = {
    ...stripConnectionSecrets(conn),
    id: `conn_${Date.now()}`,
    name: conn.name + ' (副本)',
  }
  connections.value.push(dup)
  safeInvoke('save_connection', { conn: dup }).catch(() => {})
  _toast('已复制（凭证未复制）', 'success')
}

async function onTestConnection(conn) {
  _toast('正在测试连接...', 'info')
  try {
    const password = conn.authType === 'password' && conn.id
      ? await loadCredential(conn.id)
      : ''

    let sid
    if (conn.authType === 'key') {
      sid = await invoke('ssh_connect_key', {
        host: conn.host,
        port: conn.port || 22,
        username: conn.username,
        keyPath: conn.keyPath || '',
        passphrase: conn.passphrase || '',
        trustNewHostKey: false,
      })
    } else {
      sid = await invoke('ssh_connect', {
        host: conn.host,
        port: conn.port || 22,
        username: conn.username,
        password,
        trustNewHostKey: false,
      })
    }
    _toast(`✓ ${conn.name} 连接成功`, 'success')
    await invoke('ssh_disconnect', { sessionId: sid })
  } catch (err) {
    const hostKeyError = parseHostKeyError(err)
    if (hostKeyError?.code === 'HOST_KEY_UNKNOWN') {
      _toast(`首次连接 ${conn.name}：请打开终端确认主机指纹`, 'info')
      return
    }
    if (hostKeyError?.code === 'HOST_KEY_MISMATCH') {
      _toast(`✗ ${conn.name} 主机密钥已变化，已拒绝连接`, 'error')
      return
    }
    _toast(`✗ ${conn.name} 连接失败: ${err}`, 'error')
  }
}

function onToggleFavorite(conn) {
  const idx = connections.value.findIndex(c => c.id === conn.id)
  if (idx >= 0) {
    connections.value[idx].favorite = !connections.value[idx].favorite
    safeInvoke('save_connection', { conn: stripConnectionSecrets(connections.value[idx]) }).catch(() => {})
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
