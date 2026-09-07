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

const LOCAL_CONN = { id: 'local', name: '本地终端', host: 'localhost', port: 22, username: 'local', authType: 'local', group: '本地', icon: '💻' }
const connections = ref([LOCAL_CONN])
const latencyMap = ref({})
const tabs = ref([])
const activeTabId = ref(null)
const sessionMap = ref({})

const activeTab = computed(() => tabs.value.find(tab => tab.id === activeTabId.value) || null)
const activeConnectionId = computed(() => activeTab.value?.connectionId || null)
const activeConnection = computed(() => {
  const connectionId = activeConnectionId.value
  if (!connectionId) return null
  return connections.value.find(connection => connection.id === connectionId)
    || activeTab.value?.connection
    || null
})
const activeSessionId = computed(() => sessionMap.value[activeTabId.value] || null)

let _toastFn = null

export function setToast(fn) { _toastFn = fn }
function _toast(msg, type = 'info') { _toastFn?.(msg, type) }

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
      for (const conn of saved) {
        if (!conn?.id || conn.id === 'local' || conn.authType !== 'password') continue
        try {
          await migrateLegacyPlaintextCredential(conn.id)
        } catch (e) {
          console.warn(`[Termlane] Credential migration failed for ${conn.id}:`, e)
        }
      }

      const localExists = saved.some(c => c.id === 'local')
      connections.value = localExists ? saved : [LOCAL_CONN, ...saved]
    }
  } catch (e) { console.warn('[Termlane] Load connections error:', e) }
}

function loadTabsState() {
  try {
    const savedTabs = JSON.parse(localStorage.getItem(STORAGE_KEYS.TABS) || '[]')
    if (savedTabs.length > 0) {
      tabs.value = savedTabs.map(tab => ({
        ...tab,
        connection: stripConnectionSecrets(tab.connection || {}),
      }))
      const savedActiveTabId = localStorage.getItem(STORAGE_KEYS.ACTIVE_TAB)
      activeTabId.value = tabs.value.some(tab => tab.id === savedActiveTabId)
        ? savedActiveTabId
        : savedTabs[0]?.id
      saveTabsState()
    }
  } catch (e) { console.warn('[Termlane] Load tabs error:', e) }
}

async function onSelectConnection(conn) {
  const safeConnection = stripConnectionSecrets(conn)
  const existing = tabs.value.find(tab => tab.connectionId === safeConnection.id)
  if (existing) {
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
    const isLocal = !tab?.connection || tab.connection.host === 'localhost' || tab.connection.host === '127.0.0.1'
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

async function onSaveConnection(conn, editingConn) {
  const id = editingConn?.id || conn.id || `conn_${Date.now()}`
  const merged = editingConn ? { ...editingConn, ...conn, id } : { ...conn, id }
  const safeConnection = stripConnectionSecrets(merged)
  const credential = conn.authType === 'password'
    ? conn.password
    : conn.authType === 'key'
      ? conn.passphrase
      : ''

  try {
    if (conn.authType === 'password') {
      if (!credential) throw new Error('密码不能为空')
      await saveCredential(id, credential)
    } else if (conn.authType === 'key') {
      if (credential) await saveCredential(id, credential)
      else await deleteCredential(id)
    } else {
      await deleteCredential(id)
    }

    await invoke('save_connection', { conn: safeConnection })
    await loadConnections()

    const openTab = tabs.value.find(tab => tab.connectionId === id)
    if (openTab) {
      openTab.name = safeConnection.name
      openTab.connection = safeConnection
      saveTabsState()
    }

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
    const credential = conn.id ? await loadCredential(conn.id) : ''

    let sid
    if (conn.authType === 'key') {
      sid = await invoke('ssh_connect_key', {
        host: conn.host,
        port: conn.port || 22,
        username: conn.username,
        keyPath: conn.keyPath || '',
        passphrase: credential,
        trustNewHostKey: false,
      })
    } else {
      sid = await invoke('ssh_connect', {
        host: conn.host,
        port: conn.port || 22,
        username: conn.username,
        password: credential,
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

function onSessionConnected(tabId, sid) {
  sessionMap.value[tabId] = sid
  saveTabsState()
}

function onSessionDisconnected(tabId) {
  delete sessionMap.value[tabId]
}

async function onQuickCommand(cmd) {
  if (!activeSessionId.value || !activeTab.value) {
    _toast('请先打开一个终端会话', 'info')
    return
  }

  const connection = activeConnection.value || activeTab.value.connection
  const isLocal = !connection || connection.host === 'localhost' || connection.host === '127.0.0.1'
  try {
    await invoke(isLocal ? 'local_input' : 'ssh_shell_input', {
      sessionId: activeSessionId.value,
      data: cmd + '\r',
    })
  } catch (error) {
    _toast(`发送命令失败: ${error}`, 'error')
  }
}

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

export function useAppState() {
  return {
    connections,
    latencyMap,
    tabs,
    activeTabId,
    sessionMap,
    activeConnectionId,
    activeTab,
    activeConnection,
    activeSessionId,
    loadConnections,
    loadTabsState,
    saveTabsState,
    startPingPolling,
    stopPingPolling,
    onSelectConnection,
    closeTab,
    closeOtherTabs,
    closeAllTabs,
    openLocalTerminal,
    onSaveConnection,
    onDeleteConnection,
    onDuplicateConnection,
    onTestConnection,
    onToggleFavorite,
    onSessionConnected,
    onSessionDisconnected,
    onQuickCommand,
  }
}
