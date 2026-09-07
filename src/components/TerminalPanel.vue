<template>
  <div class="flex flex-col h-full" v-show="active">
    <!-- Search Bar -->
    <div v-if="showSearch" class="h-8 flex items-center px-2 gap-2" style="background: var(--bg-surface); border-bottom: 1px solid var(--border-subtle);">
      <input ref="searchInput" v-model="searchTerm" @keydown.enter="searchNext" @keydown.shift.enter="searchPrev" class="flex-1 text-sm px-2 py-1 rounded border focus:outline-none" style="background: var(--bg-base); color: var(--fg-primary); border-color: var(--border);" placeholder="搜索... (Enter下一个, Shift+Enter上一个)" />
      <button @click="closeSearch" class="px-1" style="color: var(--fg-muted);">✕</button>
    </div>

    <!-- Connection Failed Banner -->
    <div v-if="connectFailed" class="h-8 flex items-center justify-between px-3" style="background: rgba(220, 38, 38, 0.1); border-bottom: 2px solid var(--danger);">
      <span class="text-xs font-medium" style="color: var(--danger);">连接失败</span>
      <button @click="retryConnection" class="px-3 py-0.5 text-xs rounded" style="background: var(--danger); color: white;">重试</button>
    </div>

    <!-- Terminal Container -->
    <div class="flex-1 flex overflow-hidden relative" :style="connectFailed ? 'border: 2px solid var(--danger);' : ''">
      <div ref="containerRef" class="flex-1 overflow-hidden" :style="splitMode ? { width: splitLeftWidth + '%' } : {}" />
      <div v-if="splitMode" class="w-1 cursor-col-resize hover:bg-blue-500/50 transition-colors flex-shrink-0" @mousedown="startSplitResize" />
      <div v-if="splitMode" ref="splitContainerRef" class="flex-1 overflow-hidden" style="border-left: 1px solid var(--border-subtle);" />

      <!-- Connecting Overlay -->
      <div v-if="connecting" class="absolute inset-0 flex items-center justify-center z-20" style="background: color-mix(in srgb, var(--bg-base) 80%, transparent);">
        <div class="flex flex-col items-center gap-3">
          <div class="w-8 h-8 border-2 rounded-full animate-spin" style="border-color: var(--border); border-top-color: var(--accent);"></div>
          <div class="text-sm" style="color: var(--fg-muted);">正在连接...</div>
        </div>
      </div>

      <!-- First-use Host Key Confirmation -->
      <div v-if="hostKeyPrompt" class="absolute inset-0 z-30 flex items-center justify-center p-6" style="background: color-mix(in srgb, var(--bg-base) 92%, transparent);">
        <div class="w-full max-w-lg rounded-xl p-5 shadow-2xl" style="background: var(--bg-elevated); border: 1px solid var(--border);">
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
            <button @click="cancelHostKeyTrust" class="px-3 py-1.5 text-xs rounded" style="background: var(--bg-hover); color: var(--fg-secondary);">取消</button>
            <button @click="confirmHostKeyTrust" class="px-3 py-1.5 text-xs rounded" style="background: var(--accent); color: white;">信任并连接</button>
          </div>
        </div>
      </div>
    </div>

    <!-- Quick Actions -->
    <div v-if="active" class="absolute bottom-8 right-2 flex gap-1 z-10">
      <button @click="toggleSearch" class="w-7 h-7 rounded text-xs flex items-center justify-center hover:bg-white/5" style="background: var(--bg-elevated); color: var(--fg-secondary);" title="搜索 (Ctrl+Shift+F)">🔍</button>
      <button @click="toggleSplit" class="w-7 h-7 rounded text-xs flex items-center justify-center hover:bg-white/5" :style="splitMode ? 'background: var(--accent); color: white;' : 'background: var(--bg-elevated); color: var(--fg-secondary);'" title="分屏">⊞</button>
    </div>

    <!-- Toast -->
    <div v-if="toast.show" class="absolute top-2 right-2 px-3 py-1.5 rounded text-xs z-20 transition-opacity" :class="toastClass">{{ toast.message }}</div>
  </div>
</template>

<script setup>
import { STORAGE_KEYS } from '@/utils/storage-keys.js'
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { SearchAddon } from '@xterm/addon-search'
import { WebLinksAddon } from '@xterm/addon-web-links'
import { invoke, listen, isTauri } from '../utils/tauri.js'
import { loadCredential } from '../utils/credentials.js'
import { hostKeyTarget, parseHostKeyError } from '../utils/ssh-host-key.js'
import { parseShortcut, getShortcut } from '../utils/shortcuts.js'
import { useSplitResize } from '../composables/useSplitResize.js'
import { themes } from '@/utils/themes.js'
import '@xterm/xterm/css/xterm.css'

const props = defineProps({ tab: Object, active: Boolean })
const emit = defineEmits(['connected', 'disconnected'])

const { startResize } = useSplitResize()

const containerRef = ref(null)
const splitContainerRef = ref(null)
const searchInput = ref(null)
const showSearch = ref(false)
const searchTerm = ref('')
const splitMode = ref(false)
const splitLeftWidth = ref(50)
const connecting = ref(false)
const connectFailed = ref(false)
const hostKeyPrompt = ref(null)
const pendingHostKeyReconnect = ref(false)

// Toast
const toast = ref({ show: false, message: '', type: 'info' })
const toastClass = computed(() => ({
  success: 'bg-green-600/90 text-white',
  error: 'bg-red-600/90 text-white',
  info: 'bg-gray-700/90 text-gray-200',
}[toast.value.type]))

function showToast(message, type = 'info', duration = 2000) {
  toast.value = { show: true, message, type }
  setTimeout(() => { toast.value = false }, duration)
}

let term = null
let splitTerm = null
let fitAddon = null
let splitFitAddon = null
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

function safeFit() { fitAddon?.fit() }

function createTerminal(container) {
  const fontSize = parseInt(localStorage.getItem(STORAGE_KEYS.FONT_SIZE)) || 14
  const scrollback = parseInt(localStorage.getItem(STORAGE_KEYS.SCROLLBACK)) || 10000
  const t = new Terminal({
    cursorBlink: true,
    fontSize,
    fontFamily: "'Cascadia Code', 'Cascadia Mono', 'JetBrains Mono', 'Fira Code', 'Consolas', monospace",
    theme: theme.value,
    scrollback,
  })
  const fit = new FitAddon()
  const search = new SearchAddon()
  t.loadAddon(fit)
  t.loadAddon(search)
  t.loadAddon(new WebLinksAddon())
  t.open(container)
  fit.fit()
  return { term: t, fit, search }
}

async function initTerminal() {
  if (term || !containerRef.value) return
  const { term: t, fit, search } = createTerminal(containerRef.value)
  term = t
  fitAddon = fit
  searchAddon = search

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
      inputDisposable = term.onData((data) => handleLocalInput(term, data))
    }
  } else {
    await startPtyShell(term, conn)
  }

  term.onSelectionChange(() => {
    const selected = term.getSelection()
    if (selected) {
      navigator.clipboard.writeText(selected).then(() => {
        showToast('已复制', 'success', 1000)
      }).catch(() => {})
    }
  })

  containerRef.value.addEventListener('contextmenu', handleContextmenu)

  const searchKey = parseShortcut(getShortcut('搜索'))
  term.attachCustomKeyEventHandler((e) => {
    if (searchKey(e)) { toggleSearch(); return false }
    if (e.ctrlKey && e.key === 'c' && term.hasSelection()) {
      navigator.clipboard.writeText(term.getSelection()).catch(() => {})
      return false
    }
    if (e.ctrlKey && e.key === 'v') return false
    return true
  })

  resizeObserver = new ResizeObserver(() => {
    if (fitAddon && props.active) {
      safeFit()
      if (shellId) {
        const dims = fitAddon.proposeDimensions()
        if (dims) {
          const command = sessionKind === 'local' ? 'local_resize' : 'ssh_shell_resize'
          invoke(command, { sessionId: shellId, cols: dims.cols, rows: dims.rows }).catch(() => {})
        }
      }
    }
    if (splitFitAddon && splitMode.value) splitFitAddon.fit()
  })
  resizeObserver.observe(containerRef.value)
}

function showWelcome(t, mode) {
  t.writeln('\x1b[1;36m╔══════════════════════════════════════════╗\x1b[0m')
  t.writeln('\x1b[1;36m║          XTerminal Pro v0.1.0            ║\x1b[0m')
  t.writeln('\x1b[1;36m╚══════════════════════════════════════════╝\x1b[0m')
  t.writeln('')
  t.writeln(`  模式: \x1b[1;33m${mode}\x1b[0m`)
  t.writeln('  快捷键: Ctrl+Shift+F 搜索 | Ctrl+L 清屏 | Ctrl+C 中断')
  t.writeln('')
}

// ─── Local Shell ───

async function startLocalShell(t) {
  t.writeln('\x1b[1;33m启动本地 Shell...\x1b[0m')
  connecting.value = true
  try {
    const dims = fitAddon?.proposeDimensions() || { cols: 80, rows: 24 }
    shellId = await invoke('local_start_shell', { cols: dims.cols, rows: dims.rows })
    sessionKind = 'local'
    isConnected = true
    reconnectAttempts = 0
    connecting.value = false
    connectFailed.value = false
    t.writeln('\x1b[1;32m✓ 本地 Shell 已启动\x1b[0m')
    showToast('本地终端已启动', 'success')
    emit('connected', shellId)

    unlisten?.()
    unlisten = await listen(`local-output:${shellId}`, (event) => {
      if (term && !term.disposed) term.write(event.payload)
      if (String(event.payload).includes('[Shell 已退出]')) {
        isConnected = false
        emit('disconnected')
      }
    })

    inputDisposable?.dispose()
    inputDisposable = t.onData(async (data) => {
      lastActivity = Date.now()
      if (shellId && isConnected) {
        try { await invoke('local_input', { sessionId: shellId, data }) } catch {}
      }
    })
  } catch (err) {
    connecting.value = false
    t.writeln(`\x1b[1;31m✗ 本地 Shell 启动失败: ${err}\x1b[0m`)
    showToast('本地终端启动失败', 'error')
    connectFailed.value = true
    isConnected = false
    sessionKind = null
  }
}

// ─── PTY Shell ───

async function startPtyShell(t, conn, isReconnect = false, trustNewHostKey = false) {
  if (!isReconnect && !trustNewHostKey) {
    t.writeln(`\x1b[1;33m正在连接 ${conn.username}@${conn.host}:${conn.port || 22}...\x1b[0m`)
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
    t.writeln(`\x1b[1;32m✓ 已连接到 ${conn.host}\x1b[0m`)
    showToast('连接成功', 'success')
    emit('connected', shellId)

    unlisten?.()
    unlisten = await listen(`ssh-output:${shellId}`, (event) => {
      if (term && !term.disposed) {
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
      }
      lastActivity = Date.now()
    })

    inputDisposable?.dispose()
    inputDisposable = t.onData(async (data) => {
      lastActivity = Date.now()
      if (shellId && isConnected) {
        try { await invoke('ssh_shell_input', { sessionId: shellId, data }) } catch {}
      }
    })
  } catch (err) {
    connecting.value = false
    isConnected = false
    sessionKind = null

    const hostKeyError = parseHostKeyError(err)
    if (hostKeyError?.code === 'HOST_KEY_UNKNOWN') {
      hostKeyPrompt.value = hostKeyError
      pendingHostKeyReconnect.value = isReconnect
      connectFailed.value = false
      return
    }

    if (hostKeyError?.code === 'HOST_KEY_MISMATCH') {
      t.writeln(`\x1b[1;31m✗ 主机密钥已变化，已拒绝连接：${hostKeyTarget(hostKeyError)}\x1b[0m`)
      t.writeln(`\x1b[1;31m  当前指纹：${hostKeyError.fingerprint}\x1b[0m`)
      showToast('主机密钥已变化，连接已拒绝', 'error', 4000)
      connectFailed.value = true
      return
    }

    t.writeln(`\x1b[1;31m✗ 连接失败: ${err}\x1b[0m`)
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

// ─── Browser fallback input ───

let localBuffer = ''
let commandHistory = []
let historyIndex = -1

function handleLocalInput(t, data) {
  if (data === '\r') {
    t.write('\r\n')
    const cmd = localBuffer.trim()
    if (cmd) {
      commandHistory.push(cmd)
      historyIndex = commandHistory.length
      executeLocalCommand(t, cmd)
    }
    localBuffer = ''
    if (isConnected) t.write('\x1b[1;32m$ \x1b[0m')
  } else if (data === '\x7f') {
    if (localBuffer.length > 0) { localBuffer = localBuffer.slice(0, -1); t.write('\b \b') }
  } else if (data === '\x03') {
    t.write('^C\r\n'); localBuffer = ''
    if (isConnected) t.write('\x1b[1;32m$ \x1b[0m')
  } else if (data === '\x0c') {
    t.clear(); localBuffer = ''
    if (isConnected) t.write('\x1b[1;32m$ \x1b[0m')
  } else if (data === '\x1b[A') {
    if (commandHistory.length > 0 && historyIndex > 0) {
      historyIndex--
      while (localBuffer.length > 0) { t.write('\b \b'); localBuffer = localBuffer.slice(0, -1) }
      localBuffer = commandHistory[historyIndex]
      t.write(localBuffer)
    }
  } else if (data === '\x1b[B') {
    if (historyIndex < commandHistory.length - 1) {
      historyIndex++
      while (localBuffer.length > 0) { t.write('\b \b'); localBuffer = localBuffer.slice(0, -1) }
      localBuffer = commandHistory[historyIndex]
      t.write(localBuffer)
    } else {
      historyIndex = commandHistory.length
      while (localBuffer.length > 0) { t.write('\b \b'); localBuffer = localBuffer.slice(0, -1) }
    }
  } else if (data >= ' ') {
    localBuffer += data
    t.write(data)
  }
}

function executeLocalCommand(t, cmd) {
  if (cmd === 'help') {
    t.writeln('  可用命令: help, clear, exit, version')
    t.writeln('  提示: 连接远程服务器以使用完整终端功能')
  } else if (cmd === 'version') {
    t.writeln('XTerminal Pro v0.1.0')
  } else if (cmd === 'clear') {
    t.clear()
  } else if (cmd === 'exit' || cmd === 'quit') {
    t.writeln('\x1b[33m本地模式无法退出\x1b[0m')
  } else {
    t.writeln(`\x1b[33m"${cmd}": 本地模式不支持命令执行，请连接远程服务器\x1b[0m`)
  }
}

// ─── Search ───

function toggleSearch() {
  showSearch.value = !showSearch.value
  if (showSearch.value) nextTick(() => searchInput.value?.focus())
}

function closeSearch() { showSearch.value = false; searchTerm.value = ''; searchAddon?.clearDecorations() }
function searchNext() { if (searchTerm.value && searchAddon) searchAddon.findNext(searchTerm.value) }
function searchPrev() { if (searchTerm.value && searchAddon) searchAddon.findPrevious(searchTerm.value) }

// ─── Split ───

function toggleSplit() {
  splitMode.value = !splitMode.value
  if (splitMode.value) {
    nextTick(() => {
      if (splitContainerRef.value && !splitTerm) {
        const { term: st, fit: sf } = createTerminal(splitContainerRef.value)
        splitTerm = st
        splitFitAddon = sf
        showWelcome(splitTerm, '分屏')
        splitTerm.write('\x1b[1;32m$ \x1b[0m')
        splitTerm.onData((data) => handleLocalInput(splitTerm, data))
      }
      setTimeout(() => { safeFit(); splitFitAddon?.fit() }, 50)
    })
  } else {
    splitTerm?.dispose()
    splitTerm = null
    splitFitAddon = null
    nextTick(() => safeFit())
  }
}

function startSplitResize(e) {
  const container = e.target.parentElement
  startResize(e, container, splitLeftWidth, () => {
    safeFit()
    splitFitAddon?.fit()
  })
}

// ─── Lifecycle ───

watch(() => props.active, (active) => {
  if (active && term) setTimeout(() => { safeFit(); if (splitMode.value) splitFitAddon?.fit() }, 50)
})

function retryConnection() {
  connectFailed.value = false
  hostKeyPrompt.value = null
  const conn = props.tab?.connection
  if (!term || !conn) return
  if (conn.host === 'localhost') startLocalShell(term)
  else startPtyShell(term, conn)
}

async function handleContextmenu(e) {
  e.preventDefault()
  try {
    const text = await navigator.clipboard.readText()
    if (!text || !shellId) return
    const command = sessionKind === 'local' ? 'local_input' : 'ssh_shell_input'
    await invoke(command, { sessionId: shellId, data: text })
  } catch {}
}

function handleStorage(e) {
  if (e.key === STORAGE_KEYS.THEME) {
    theme.value = getTheme()
    if (term) term.options.theme = theme.value
    if (splitTerm) splitTerm.options.theme = theme.value
  }
}

onMounted(() => {
  initTerminal()
  idleTimer = setInterval(() => {
    if (isConnected && shellId && Date.now() - lastActivity > IDLE_TIMEOUT_MS) {
      term?.writeln('\r\n\x1b[1;33m[空闲超时 30 分钟，自动断开]\x1b[0m')
      const command = sessionKind === 'local' ? 'local_close_shell' : 'ssh_close_shell'
      invoke(command, { sessionId: shellId }).catch(() => {})
      isConnected = false
      shellId = null
      sessionKind = null
      emit('disconnected')
    }
  }, 60000)

  window.addEventListener('storage', handleStorage)
})

onUnmounted(async () => {
  clearInterval(idleTimer)
  resizeObserver?.disconnect()
  unlisten?.()
  inputDisposable?.dispose()
  containerRef.value?.removeEventListener('contextmenu', handleContextmenu)
  window.removeEventListener('storage', handleStorage)
  if (shellId) {
    const command = sessionKind === 'local' ? 'local_close_shell' : 'ssh_close_shell'
    await invoke(command, { sessionId: shellId }).catch(() => {})
  }
  splitTerm?.dispose()
  term?.dispose()
})
</script>
