<template>
  <div class="terminal-panel flex flex-col h-full" v-show="active">
    <div v-if="showSearch" class="search-bar">
      <SearchIcon :size="14" :stroke-width="1.8" />
      <input
        ref="searchInput"
        v-model="searchTerm"
        class="search-input"
        placeholder="搜索终端内容"
        @keydown.enter="searchNext"
        @keydown.shift.enter="searchPrev"
      />
      <button type="button" class="icon-button" aria-label="关闭搜索" @click="closeSearch">
        <X :size="14" :stroke-width="1.8" />
      </button>
    </div>

    <div v-if="connectFailed" class="failure-banner">
      <span>连接失败</span>
      <button type="button" @click="retryConnection">重试</button>
    </div>

    <div class="flex-1 overflow-hidden relative" :class="{ 'connection-failed': connectFailed }">
      <div ref="containerRef" class="w-full h-full overflow-hidden" />

      <div v-if="connecting" class="connecting-overlay">
        <div class="flex flex-col items-center gap-3">
          <div class="spinner" />
          <div class="text-sm" style="color: var(--fg-muted);">正在连接...</div>
        </div>
      </div>

      <div v-if="hostKeyPrompt" class="host-key-overlay">
        <div class="host-key-card">
          <div class="text-sm font-semibold mb-1" style="color: var(--fg-primary);">首次连接：确认服务器身份</div>
          <div class="text-xs mb-4" style="color: var(--fg-muted);">
            {{ hostKeyTarget(hostKeyPrompt) }} 尚未记录在 known_hosts。请核对服务器指纹后再继续。
          </div>
          <div class="space-y-2 text-xs">
            <div class="flex gap-3">
              <span class="w-16 shrink-0" style="color: var(--fg-muted);">算法</span>
              <span class="font-mono" style="color: var(--fg-secondary);">{{ hostKeyPrompt.algorithm }}</span>
            </div>
            <div class="flex gap-3 items-start">
              <span class="w-16 shrink-0" style="color: var(--fg-muted);">SHA256</span>
              <code class="font-mono break-all select-text" style="color: var(--fg-primary);">{{ hostKeyPrompt.fingerprint }}</code>
            </div>
          </div>
          <div class="mt-4 text-xs" style="color: var(--warning);">
            无法确认指纹时不要继续。确认后该主机密钥会写入 ~/.ssh/known_hosts。
          </div>
          <div class="mt-5 flex justify-end gap-2">
            <button type="button" class="secondary-button" @click="cancelHostKeyTrust">取消</button>
            <button type="button" class="primary-button" @click="confirmHostKeyTrust">信任并连接</button>
          </div>
        </div>
      </div>
    </div>

    <div v-if="active" class="terminal-actions">
      <button type="button" class="terminal-action" title="搜索 (Ctrl+Shift+F)" aria-label="搜索终端内容" @click="toggleSearch">
        <SearchIcon :size="14" :stroke-width="1.8" />
      </button>
    </div>

    <div v-if="toast.show" class="terminal-toast" :class="toastClass">{{ toast.message }}</div>
  </div>
</template>

<script setup>
import { STORAGE_KEYS } from '@/utils/storage-keys.js'
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { Search as SearchIcon, X } from 'lucide-vue-next'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { SearchAddon } from '@xterm/addon-search'
import { WebLinksAddon } from '@xterm/addon-web-links'
import { invoke, isTauri, listen } from '../utils/tauri.js'
import { loadCredential } from '../utils/credentials.js'
import { hostKeyTarget, parseHostKeyError } from '../utils/ssh-host-key.js'
import { getShortcut, parseShortcut } from '../utils/shortcuts.js'
import { themes } from '@/utils/themes.js'
import '@xterm/xterm/css/xterm.css'

const props = defineProps({ tab: Object, active: Boolean })
const emit = defineEmits(['connected', 'disconnected'])

const containerRef = ref(null)
const searchInput = ref(null)
const showSearch = ref(false)
const searchTerm = ref('')
const connecting = ref(false)
const connectFailed = ref(false)
const hostKeyPrompt = ref(null)
const pendingHostKeyReconnect = ref(false)
const appVersion = ref('')

const toast = ref({ show: false, message: '', type: 'info' })
const toastClass = computed(() => ({
  success: 'success',
  error: 'error',
  info: 'info',
}[toast.value.type]))

function showToast(message, type = 'info', duration = 2000) {
  toast.value = { show: true, message, type }
  setTimeout(() => { toast.value.show = false }, duration)
}

let term = null
let fitAddon = null
let searchAddon = null
let resizeObserver = null
let shellId = null
let sessionKind = null
let isConnected = false
let unlisten = null
let inputDisposable = null
let reconnectAttempts = 0
const MAX_RECONNECT = 3
let idleTimer = null
let lastActivity = Date.now()
const IDLE_TIMEOUT_MS = 30 * 60 * 1000

function getTheme() {
  const themeName = localStorage.getItem(STORAGE_KEYS.THEME) || 'dark'
  return themes[themeName] || themes.dark
}

const theme = ref(getTheme())

function safeFit() {
  fitAddon?.fit()
}

function createTerminal(container) {
  const fontSize = parseInt(localStorage.getItem(STORAGE_KEYS.FONT_SIZE)) || 14
  const scrollback = parseInt(localStorage.getItem(STORAGE_KEYS.SCROLLBACK)) || 10000
  const terminal = new Terminal({
    cursorBlink: true,
    fontSize,
    fontFamily: "'Cascadia Code', 'Cascadia Mono', 'JetBrains Mono', 'Fira Code', 'Consolas', monospace",
    theme: theme.value,
    scrollback,
  })
  const fit = new FitAddon()
  const search = new SearchAddon()
  terminal.loadAddon(fit)
  terminal.loadAddon(search)
  terminal.loadAddon(new WebLinksAddon())
  terminal.open(container)
  fit.fit()
  return { terminal, fit, search }
}

async function initTerminal() {
  if (term || !containerRef.value) return

  const created = createTerminal(containerRef.value)
  term = created.terminal
  fitAddon = created.fit
  searchAddon = created.search

  const conn = props.tab.connection
  const isLocal = !conn || conn.host === 'localhost'

  if (isLocal) {
    connectFailed.value = false
    if (isTauri) {
      await startLocalShell(term)
    } else {
      connecting.value = false
      sessionKind = 'browser'
      showWelcome(term, '浏览器模式')
      term.write('\r\n\x1b[1;32m$ \x1b[0m')
      isConnected = true
      inputDisposable = term.onData(data => handleLocalInput(term, data))
    }
  } else {
    await startPtyShell(term, conn)
  }

  term.onSelectionChange(() => {
    const selected = term.getSelection()
    if (!selected) return
    navigator.clipboard.writeText(selected).then(() => {
      showToast('已复制', 'success', 1000)
    }).catch(() => {})
  })

  containerRef.value.addEventListener('contextmenu', handleContextmenu)

  const searchKey = parseShortcut(getShortcut('搜索'))
  term.attachCustomKeyEventHandler(event => {
    if (searchKey(event)) {
      toggleSearch()
      return false
    }
    if (event.ctrlKey && event.key === 'c' && term.hasSelection()) {
      navigator.clipboard.writeText(term.getSelection()).catch(() => {})
      return false
    }
    if (event.ctrlKey && event.key === 'v') return false
    return true
  })

  resizeObserver = new ResizeObserver(() => {
    if (!fitAddon || !props.active) return
    safeFit()
    if (!shellId) return

    const dims = fitAddon.proposeDimensions()
    if (!dims) return
    const command = sessionKind === 'local' ? 'local_resize' : 'ssh_shell_resize'
    invoke(command, { sessionId: shellId, cols: dims.cols, rows: dims.rows }).catch(() => {})
  })
  resizeObserver.observe(containerRef.value)
}

function showWelcome(terminal, mode) {
  const version = appVersion.value ? ` v${appVersion.value}` : ''
  terminal.writeln('\x1b[1;36m╔══════════════════════════════════════════╗\x1b[0m')
  terminal.writeln(`\x1b[1;36m║              XTerminal Pro${version.padEnd(12)}║\x1b[0m`)
  terminal.writeln('\x1b[1;36m╚══════════════════════════════════════════╝\x1b[0m')
  terminal.writeln('')
  terminal.writeln(`  模式: \x1b[1;33m${mode}\x1b[0m`)
  terminal.writeln('  快捷键: Ctrl+Shift+F 搜索 | Ctrl+L 清屏 | Ctrl+C 中断')
  terminal.writeln('')
}

async function startLocalShell(terminal) {
  terminal.writeln('\x1b[1;33m启动本地 Shell...\x1b[0m')
  connecting.value = true

  try {
    const dims = fitAddon?.proposeDimensions() || { cols: 80, rows: 24 }
    shellId = await invoke('local_start_shell', { cols: dims.cols, rows: dims.rows })
    sessionKind = 'local'
    isConnected = true
    reconnectAttempts = 0
    connecting.value = false
    connectFailed.value = false
    terminal.writeln('\x1b[1;32m✓ 本地 Shell 已启动\x1b[0m')
    showToast('本地终端已启动', 'success')
    emit('connected', shellId)

    unlisten?.()
    unlisten = await listen(`local-output:${shellId}`, event => {
      if (term && !term.disposed) term.write(event.payload)
      if (String(event.payload).includes('[Shell 已退出]')) {
        isConnected = false
        emit('disconnected')
      }
    })

    inputDisposable?.dispose()
    inputDisposable = terminal.onData(async data => {
      lastActivity = Date.now()
      if (!shellId || !isConnected) return
      try { await invoke('local_input', { sessionId: shellId, data }) } catch {}
    })
  } catch (error) {
    connecting.value = false
    terminal.writeln(`\x1b[1;31m✗ 本地 Shell 启动失败: ${error}\x1b[0m`)
    showToast('本地终端启动失败', 'error')
    connectFailed.value = true
    isConnected = false
    sessionKind = null
  }
}

async function startPtyShell(terminal, conn, isReconnect = false, trustNewHostKey = false) {
  if (!isReconnect && !trustNewHostKey) {
    terminal.writeln(`\x1b[1;33m正在连接 ${conn.username}@${conn.host}:${conn.port || 22}...\x1b[0m`)
  }
  connecting.value = true
  hostKeyPrompt.value = null

  try {
    const dims = fitAddon?.proposeDimensions() || { cols: 80, rows: 24 }
    const secret = conn.id ? await loadCredential(conn.id) : ''

    shellId = await invoke('ssh_start_shell', {
      host: conn.host,
      port: conn.port || 22,
      username: conn.username,
      password: conn.authType === 'password' ? secret : '',
      keyPath: conn.keyPath || null,
      passphrase: conn.authType === 'key' ? secret : null,
      trustNewHostKey,
      cols: dims.cols,
      rows: dims.rows,
    })

    sessionKind = 'ssh'
    isConnected = true
    reconnectAttempts = 0
    connecting.value = false
    connectFailed.value = false
    terminal.writeln(`\x1b[1;32m✓ 已连接到 ${conn.host}\x1b[0m`)
    showToast('连接成功', 'success')
    emit('connected', shellId)

    unlisten?.()
    unlisten = await listen(`ssh-output:${shellId}`, event => {
      if (!term || term.disposed) return

      const payload = String(event.payload)
      term.write(payload)
      if (payload.includes('[Shell 已退出]') || payload.includes('[连接断开]')) {
        isConnected = false
        emit('disconnected')
        if (reconnectAttempts < MAX_RECONNECT) {
          reconnectAttempts++
          term.writeln(`\r\n\x1b[1;33m正在重连 (${reconnectAttempts}/${MAX_RECONNECT})...\x1b[0m`)
          setTimeout(() => startPtyShell(term, conn, true, false), 2000 * reconnectAttempts)
        } else {
          term.writeln('\r\n\x1b[1;31m重连失败，请手动重新连接\x1b[0m')
          connectFailed.value = true
        }
      }
      lastActivity = Date.now()
    })

    inputDisposable?.dispose()
    inputDisposable = terminal.onData(async data => {
      lastActivity = Date.now()
      if (!shellId || !isConnected) return
      try { await invoke('ssh_shell_input', { sessionId: shellId, data }) } catch {}
    })
  } catch (error) {
    connecting.value = false
    isConnected = false
    sessionKind = null

    const hostKeyError = parseHostKeyError(error)
    if (hostKeyError?.code === 'HOST_KEY_UNKNOWN') {
      hostKeyPrompt.value = hostKeyError
      pendingHostKeyReconnect.value = isReconnect
      connectFailed.value = false
      return
    }

    if (hostKeyError?.code === 'HOST_KEY_MISMATCH') {
      terminal.writeln(`\x1b[1;31m✗ 主机密钥已变化，已拒绝连接：${hostKeyTarget(hostKeyError)}\x1b[0m`)
      terminal.writeln(`\x1b[1;31m  当前指纹：${hostKeyError.fingerprint}\x1b[0m`)
      showToast('主机密钥已变化，连接已拒绝', 'error', 4000)
      connectFailed.value = true
      return
    }

    terminal.writeln(`\x1b[1;31m✗ 连接失败: ${error}\x1b[0m`)
    showToast('连接失败', 'error')
    connectFailed.value = true
  }
}

async function confirmHostKeyTrust() {
  const conn = props.tab?.connection
  if (!conn || !term) return
  const isReconnect = pendingHostKeyReconnect.value
  hostKeyPrompt.value = null
  await startPtyShell(term, conn, isReconnect, true)
}

function cancelHostKeyTrust() {
  if (hostKeyPrompt.value && term) {
    term.writeln(`\x1b[1;33m未信任 ${hostKeyTarget(hostKeyPrompt.value)}，连接已取消。\x1b[0m`)
  }
  hostKeyPrompt.value = null
  pendingHostKeyReconnect.value = false
  connectFailed.value = true
}

let localBuffer = ''
const commandHistory = []
let historyIndex = -1

function handleLocalInput(terminal, data) {
  if (data === '\r') {
    terminal.write('\r\n')
    const command = localBuffer.trim()
    if (command) {
      commandHistory.push(command)
      historyIndex = commandHistory.length
      executeLocalCommand(terminal, command)
    }
    localBuffer = ''
    if (isConnected) terminal.write('\x1b[1;32m$ \x1b[0m')
  } else if (data === '\x7f') {
    if (localBuffer.length > 0) {
      localBuffer = localBuffer.slice(0, -1)
      terminal.write('\b \b')
    }
  } else if (data === '\x03') {
    terminal.write('^C\r\n')
    localBuffer = ''
    if (isConnected) terminal.write('\x1b[1;32m$ \x1b[0m')
  } else if (data === '\x0c') {
    terminal.clear()
    localBuffer = ''
    if (isConnected) terminal.write('\x1b[1;32m$ \x1b[0m')
  } else if (data === '\x1b[A') {
    if (commandHistory.length > 0 && historyIndex > 0) {
      historyIndex--
      while (localBuffer.length > 0) {
        terminal.write('\b \b')
        localBuffer = localBuffer.slice(0, -1)
      }
      localBuffer = commandHistory[historyIndex]
      terminal.write(localBuffer)
    }
  } else if (data === '\x1b[B') {
    if (historyIndex < commandHistory.length - 1) {
      historyIndex++
      while (localBuffer.length > 0) {
        terminal.write('\b \b')
        localBuffer = localBuffer.slice(0, -1)
      }
      localBuffer = commandHistory[historyIndex]
      terminal.write(localBuffer)
    } else {
      historyIndex = commandHistory.length
      while (localBuffer.length > 0) {
        terminal.write('\b \b')
        localBuffer = localBuffer.slice(0, -1)
      }
    }
  } else if (data >= ' ') {
    localBuffer += data
    terminal.write(data)
  }
}

function executeLocalCommand(terminal, command) {
  if (command === 'help') {
    terminal.writeln('  可用命令: help, clear, exit, version')
    terminal.writeln('  提示: 连接远程服务器以使用完整终端功能')
  } else if (command === 'version') {
    terminal.writeln(`XTerminal Pro${appVersion.value ? ` v${appVersion.value}` : ''}`)
  } else if (command === 'clear') {
    terminal.clear()
  } else if (command === 'exit' || command === 'quit') {
    terminal.writeln('\x1b[33m浏览器模式无法退出\x1b[0m')
  } else {
    terminal.writeln(`\x1b[33m"${command}": 浏览器演示模式不支持真实命令执行\x1b[0m`)
  }
}

function toggleSearch() {
  showSearch.value = !showSearch.value
  if (showSearch.value) nextTick(() => searchInput.value?.focus())
}

function closeSearch() {
  showSearch.value = false
  searchTerm.value = ''
  searchAddon?.clearDecorations()
}

function searchNext() {
  if (searchTerm.value && searchAddon) searchAddon.findNext(searchTerm.value)
}

function searchPrev() {
  if (searchTerm.value && searchAddon) searchAddon.findPrevious(searchTerm.value)
}

watch(() => props.active, active => {
  if (active && term) setTimeout(safeFit, 50)
})

function retryConnection() {
  connectFailed.value = false
  hostKeyPrompt.value = null
  const conn = props.tab?.connection
  if (!term || !conn) return
  if (conn.host === 'localhost') startLocalShell(term)
  else startPtyShell(term, conn)
}

async function handleContextmenu(event) {
  event.preventDefault()
  try {
    const text = await navigator.clipboard.readText()
    if (!text || !shellId) return
    const command = sessionKind === 'local' ? 'local_input' : 'ssh_shell_input'
    await invoke(command, { sessionId: shellId, data: text })
  } catch {}
}

function applyCurrentTheme() {
  theme.value = getTheme()
  if (term) term.options.theme = theme.value
}

function handleStorage(event) {
  if (event.key === STORAGE_KEYS.THEME) applyCurrentTheme()
}

function handleThemeChanged() {
  applyCurrentTheme()
}

onMounted(async () => {
  try {
    appVersion.value = await invoke('get_app_version')
  } catch {
    appVersion.value = ''
  }

  await initTerminal()

  idleTimer = setInterval(() => {
    if (!isConnected || !shellId || Date.now() - lastActivity <= IDLE_TIMEOUT_MS) return

    term?.writeln('\r\n\x1b[1;33m[空闲超时 30 分钟，自动断开]\x1b[0m')
    const command = sessionKind === 'local' ? 'local_close_shell' : 'ssh_close_shell'
    invoke(command, { sessionId: shellId }).catch(() => {})
    isConnected = false
    shellId = null
    sessionKind = null
    emit('disconnected')
  }, 60_000)

  window.addEventListener('storage', handleStorage)
  window.addEventListener('xterminal-theme-changed', handleThemeChanged)
})

onUnmounted(async () => {
  clearInterval(idleTimer)
  resizeObserver?.disconnect()
  unlisten?.()
  inputDisposable?.dispose()
  containerRef.value?.removeEventListener('contextmenu', handleContextmenu)
  window.removeEventListener('storage', handleStorage)
  window.removeEventListener('xterminal-theme-changed', handleThemeChanged)

  if (shellId) {
    const command = sessionKind === 'local' ? 'local_close_shell' : 'ssh_close_shell'
    await invoke(command, { sessionId: shellId }).catch(() => {})
  }
  term?.dispose()
})
</script>

<style scoped>
.terminal-panel {
  position: relative;
  background: var(--bg-base);
}

.search-bar {
  height: 34px;
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 0 7px 0 10px;
  flex-shrink: 0;
  border-bottom: 1px solid var(--border-subtle);
  background: var(--bg-surface);
  color: var(--fg-muted);
}

.search-input {
  flex: 1;
  min-width: 0;
  height: 26px;
  padding: 0 8px;
  border: 1px solid var(--border);
  border-radius: 5px;
  outline: none;
  background: var(--bg-base);
  color: var(--fg-primary);
  font-size: 12px;
}

.search-input:focus {
  border-color: var(--accent);
}

.icon-button,
.terminal-action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: 5px;
  background: transparent;
  color: var(--fg-muted);
  transition: background-color var(--transition-fast), color var(--transition-fast);
}

.icon-button {
  width: 26px;
  height: 26px;
}

.icon-button:hover,
.terminal-action:hover {
  background: var(--bg-hover);
  color: var(--fg-primary);
}

.failure-banner {
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-shrink: 0;
  padding: 0 10px;
  border-bottom: 1px solid color-mix(in srgb, var(--danger) 45%, transparent);
  background: color-mix(in srgb, var(--danger) 10%, var(--bg-surface));
  color: var(--danger);
  font-size: 12px;
  font-weight: 500;
}

.failure-banner button {
  padding: 4px 8px;
  border: 0;
  border-radius: 5px;
  background: var(--danger);
  color: white;
  font-size: 11px;
}

.connection-failed {
  box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--danger) 60%, transparent);
}

.connecting-overlay,
.host-key-overlay {
  position: absolute;
  inset: 0;
  z-index: 20;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
  background: color-mix(in srgb, var(--bg-base) 88%, transparent);
}

.host-key-overlay {
  z-index: 30;
  background: color-mix(in srgb, var(--bg-base) 94%, transparent);
}

.spinner {
  width: 28px;
  height: 28px;
  border: 2px solid var(--border);
  border-top-color: var(--accent);
  border-radius: 999px;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.host-key-card {
  width: min(520px, 100%);
  padding: 18px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--bg-elevated);
  box-shadow: var(--shadow-lg);
}

.secondary-button,
.primary-button {
  height: 30px;
  padding: 0 11px;
  border: 0;
  border-radius: 6px;
  font-size: 12px;
}

.secondary-button {
  background: var(--bg-hover);
  color: var(--fg-secondary);
}

.primary-button {
  background: var(--accent);
  color: white;
}

.terminal-actions {
  position: absolute;
  right: 8px;
  bottom: 8px;
  z-index: 10;
}

.terminal-action {
  width: 28px;
  height: 28px;
  border: 1px solid var(--border-subtle);
  background: var(--bg-elevated);
  box-shadow: var(--shadow-sm);
}

.terminal-toast {
  position: absolute;
  top: 8px;
  right: 8px;
  z-index: 25;
  padding: 6px 9px;
  border-radius: 6px;
  font-size: 11px;
  box-shadow: var(--shadow-sm);
}

.terminal-toast.success {
  background: color-mix(in srgb, var(--success) 18%, var(--bg-elevated));
  color: var(--success);
}

.terminal-toast.error {
  background: color-mix(in srgb, var(--danger) 18%, var(--bg-elevated));
  color: var(--danger);
}

.terminal-toast.info {
  background: var(--bg-elevated);
  color: var(--fg-secondary);
}
</style>
