<template>
  <div class="flex flex-col h-full" v-show="active">
    <!-- Search Bar -->
    <div v-if="showSearch" class="h-8 flex items-center px-2 gap-2" style="background: var(--bg-surface); border-bottom: 1px solid var(--border-subtle);">
      <input ref="searchInput" v-model="searchTerm" @keydown.enter="searchNext" @keydown.shift.enter="searchPrev" class="flex-1 text-sm px-2 py-1 rounded border focus:outline-none" style="background: var(--bg-base); color: var(--fg-primary); border-color: var(--border););" placeholder="搜索... (Enter下一个, Shift+Enter上一个)" />
      <button @click="closeSearch" style="color: var(--fg-muted);" px-1>✕</button>
    </div>

    <!-- Connection Failed Banner -->
    <div v-if="connectFailed" class="h-8 flex items-center justify-between px-3" style="background: rgba(220, 38, 38, 0.1); border-bottom: 2px solid var(--danger);">
      <span class="text-xs font-medium" style="color: var(--danger);">🔴 连接失败</span>
      <button @click="retryConnection" class="px-3 py-0.5 text-xs rounded" style="background: var(--danger); color: white;">重试</button>
    </div>

    <!-- Terminal Container -->
    <div class="flex-1 flex overflow-hidden" :style="connectFailed ? 'border: 2px solid var(--danger);' : ''">
      <div ref="containerRef" class="flex-1 overflow-hidden" :style="splitMode ? { width: splitLeftWidth + '%' } : {}" />
      <div v-if="splitMode" class="w-1 cursor-col-resize hover:bg-blue-500/50 transition-colors flex-shrink-0" @mousedown="startSplitResize" />
      <div v-if="splitMode" ref="splitContainerRef" class="flex-1 overflow-hidden" style="border-left: 1px solid var(--border-subtle);" />
    </div>

    <!-- Quick Actions -->
    <div v-if="active" class="absolute bottom-8 right-2 flex gap-1 z-10">
      <button @click="toggleSearch" class="w-7 h-7 rounded text-xs flex items-center justify-center" style="background: var(--bg-elevated); color: var(--fg-secondary); hover:background: var(--bg-hover);" title="搜索 (Ctrl+Shift+F)">🔍</button>
      <button @click="toggleSplit" class="w-7 h-7 rounded text-xs flex items-center justify-center" :style="splitMode ? 'background: var(--accent); color: white;' : 'background: var(--bg-elevated); color: var(--fg-secondary); hover:background: var(--bg-hover);'" title="分屏">⊞</button>
    </div>

    <!-- Toast -->
    <div v-if="toast.show" class="absolute top-2 right-2 px-3 py-1.5 rounded text-xs z-20 transition-opacity" :class="toastClass">{{ toast.message }}</div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { Terminal } from 'xterm'
import { FitAddon } from 'xterm-addon-fit'
import { SearchAddon } from 'xterm-addon-search'
import { WebLinksAddon } from 'xterm-addon-web-links'
import { invoke, listen, isTauri } from '../utils/tauri.js'
import 'xterm/css/xterm.css'

const props = defineProps({ tab: Object, active: Boolean })
const emit = defineEmits(['connected', 'disconnected'])

const containerRef = ref(null)
const splitContainerRef = ref(null)
const searchInput = ref(null)
const showSearch = ref(false)
const searchTerm = ref('')
const splitMode = ref(false)
const splitLeftWidth = ref(50) // percentage for resizable split

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
let isConnected = false
let connectFailed = ref(false)
let unlisten = null
let reconnectAttempts = 0
const MAX_RECONNECT = 3
let idleTimer = null
let lastActivity = Date.now()
const IDLE_TIMEOUT_MS = 30 * 60 * 1000 // 30 minutes

const theme = {
  background: '#1e1e1e', foreground: '#d4d4d4', cursor: '#aeafad', selection: '#264f78',
  black: '#000000', red: '#cd3131', green: '#0dbc79', yellow: '#e5e510',
  blue: '#2472c8', magenta: '#bc3fbc', cyan: '#11a8cd', white: '#e5e5e5',
  brightBlack: '#666666', brightRed: '#f14c4c', brightGreen: '#23d18b', brightYellow: '#f5f543',
  brightBlue: '#3b8eea', brightMagenta: '#d670d6', brightCyan: '#29b8db', brightWhite: '#e5e5e5',
}

function createTerminal(container) {
  const fontSize = parseInt(localStorage.getItem('xterminal-fontSize')) || 14
  const scrollback = parseInt(localStorage.getItem('xterminal-scrollback')) || 10000
  const t = new Terminal({
    cursorBlink: true, fontSize,
    fontFamily: "'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace",
    theme, scrollback,
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
      // Real local shell via portable-pty
      await startLocalShell(term)
    } else {
      // Browser fallback
      showWelcome(term, '浏览器模式')
      term.write('\r\n\x1b[1;32m$ \x1b[0m')
      isConnected = true
      term.onData((data) => handleLocalInput(term, data))
    }
  } else {
    await startPtyShell(term, conn)
  }

  // Auto-copy on selection
  term.onSelectionChange(() => {
    const selected = term.getSelection()
    if (selected) {
      navigator.clipboard.writeText(selected).then(() => {
        showToast('已复制', 'success', 1000)
      }).catch(() => {})
    }
  })

  // Right-click paste
  containerRef.value.addEventListener('contextmenu', async (e) => {
    e.preventDefault()
    try {
      const text = await navigator.clipboard.readText()
      if (text && shellId) {
        await invoke('ssh_shell_input', { sessionId: shellId, data: text })
      }
    } catch {}
  })

  // Keyboard shortcuts
  term.attachCustomKeyEventHandler((e) => {
    if (e.ctrlKey && e.shiftKey && e.key === 'F') { toggleSearch(); return false }
    if (e.ctrlKey && e.key === 'c' && term.hasSelection()) {
      // Ctrl+C with selection → copy, not interrupt
      navigator.clipboard.writeText(term.getSelection()).catch(() => {})
      return false
    }
    if (e.ctrlKey && e.key === 'v') return false // Let paste handler deal with it
    return true
  })

  resizeObserver = new ResizeObserver(() => {
    if (fitAddon && props.active) {
      fitAddon.fit()
      if (shellId) {
        const dims = fitAddon.proposeDimensions()
        if (dims) {
          invoke('ssh_shell_resize', { sessionId: shellId, cols: dims.cols, rows: dims.rows }).catch(() => {})
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

// ─── Local Shell (real local PTY via portable-pty) ───

async function startLocalShell(t) {
  t.writeln('\x1b[1;33m启动本地 Shell...\x1b[0m')
  try {
    const dims = fitAddon?.proposeDimensions() || { cols: 80, rows: 24 }
    shellId = await invoke('local_start_shell', { cols: dims.cols, rows: dims.rows })
    isConnected = true
    reconnectAttempts = 0
    t.writeln('\x1b[1;32m✓ 本地 Shell 已启动\x1b[0m')
    showToast('本地终端已启动', 'success')
    emit('connected', shellId)

    // Listen for output
    unlisten = await listen(`local-output:${shellId}`, (event) => {
      if (term && !term.disposed) {
        term.write(event.payload)
      }
      if (event.payload.includes('[Shell 已退出]')) {
        isConnected = false
        emit('disconnected')
      }
    })

    // Send input
    t.onData(async (data) => {
      lastActivity = Date.now()
      if (shellId && isConnected) {
        try { await invoke('local_shell_input', { sessionId: shellId, data }) }
        catch { /* input failed silently */ }
      }
    })
  } catch (err) {
    t.writeln(`\x1b[1;31m✗ 本地 Shell 启动失败: ${err}\x1b[0m`)
    showToast('本地终端启动失败', 'error')
    connectFailed.value = true
    isConnected = false
  }
}

// ─── PTY Shell (real interactive terminal) ───

async function startPtyShell(t, conn, isReconnect = false) {
  if (!isReconnect) {
    t.writeln(`\x1b[1;33m正在连接 ${conn.username}@${conn.host}:${conn.port || 22}...\x1b[0m`)
  }

  try {
    // Get terminal dimensions
    const dims = fitAddon?.proposeDimensions() || { cols: 80, rows: 24 }

    // Start PTY shell
    shellId = await invoke('ssh_start_shell', {
      host: conn.host,
      port: conn.port || 22,
      username: conn.username,
      password: conn.password || '',
      keyPath: conn.keyPath || null,
      passphrase: conn.passphrase || null,
      cols: dims.cols,
      rows: dims.rows,
    })

    isConnected = true
    reconnectAttempts = 0
    t.writeln(`\x1b[1;32m✓ 已连接到 ${conn.host}\x1b[0m`)
    showToast('连接成功', 'success')
    emit('connected', shellId)

    // Listen for shell output events
    unlisten = await listen(`ssh-output:${shellId}`, (event) => {
      if (term && !term.disposed) {
        term.write(event.payload)
        // Check for disconnect message
        if (event.payload.includes('[Shell 已退出]') || event.payload.includes('[连接断开]')) {
          isConnected = false
          emit('disconnected')
          // Auto-reconnect
          if (reconnectAttempts < MAX_RECONNECT) {
            reconnectAttempts++
            term.writeln(`\r\n\x1b[1;33m正在重连 (${reconnectAttempts}/${MAX_RECONNECT})...\x1b[0m`)
            setTimeout(() => startPtyShell(term, conn, true), 2000 * reconnectAttempts)
          } else {
            term.writeln('\r\n\x1b[1;31m重连失败，请手动重新连接\x1b[0m')
          }
        }
      }
      // Reset idle timer on any output
      lastActivity = Date.now()
    })

    // Send user input to PTY shell
    t.onData(async (data) => {
      lastActivity = Date.now() // Reset idle timer
      if (shellId && isConnected) {
        try {
          await invoke('ssh_shell_input', { sessionId: shellId, data })
        } catch {
          // input failed silently
        }
      }
    })

  } catch (err) {
    t.writeln(`\x1b[1;31m✗ 连接失败: ${err}\x1b[0m`)
    showToast('连接失败', 'error')
    connectFailed.value = true
    // Fallback to local mode
    isConnected = false
  }
}

// ─── Local/Fallback Input Handler ───

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
    // Up arrow - history
    if (commandHistory.length > 0 && historyIndex > 0) {
      historyIndex--
      while (localBuffer.length > 0) { t.write('\b \b'); localBuffer = localBuffer.slice(0, -1) }
      localBuffer = commandHistory[historyIndex]
      t.write(localBuffer)
    }
  } else if (data === '\x1b[B') {
    // Down arrow - history
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
    localBuffer += data; t.write(data)
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
        splitTerm = st; splitFitAddon = sf
        showWelcome(splitTerm, '分屏')
        splitTerm.write('\x1b[1;32m$ \x1b[0m')
        splitTerm.onData((data) => handleLocalInput(splitTerm, data))
      }
      setTimeout(() => { fitAddon?.fit(); splitFitAddon?.fit() }, 50)
    })
  } else {
    splitTerm?.dispose(); splitTerm = null; splitFitAddon = null
    nextTick(() => fitAddon?.fit())
  }
}

// ─── Split Resize ───
function startSplitResize(e) {
  e.preventDefault()
  const container = e.target.parentElement
  const rect = container.getBoundingClientRect()
  const startX = e.clientX
  const startW = splitLeftWidth.value
  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'
  const onMove = (ev) => {
    const pct = startW + ((ev.clientX - startX) / rect.width) * 100
    splitLeftWidth.value = Math.max(20, Math.min(80, pct))
    fitAddon?.fit(); splitFitAddon?.fit()
  }
  const onUp = () => {
    document.removeEventListener('mousemove', onMove)
    document.removeEventListener('mouseup', onUp)
    document.body.style.cursor = ''
    document.body.style.userSelect = ''
    fitAddon?.fit(); splitFitAddon?.fit()
  }
  document.addEventListener('mousemove', onMove)
  document.addEventListener('mouseup', onUp)
}

// ─── Lifecycle ───

watch(() => props.active, (active) => {
  if (active && term) setTimeout(() => { fitAddon?.fit(); if (splitMode.value) splitFitAddon?.fit() }, 50)
})

function retryConnection() {
  connectFailed.value = false
  term?.reset()
  initTerminal()
}

onMounted(() => {
  initTerminal()
  // Idle timeout check every minute
  idleTimer = setInterval(() => {
    if (isConnected && shellId && Date.now() - lastActivity > IDLE_TIMEOUT_MS) {
      term?.writeln('\r\n\x1b[1;33m[空闲超时 30 分钟，自动断开]\x1b[0m')
      invoke('ssh_close_shell', { sessionId: shellId }).catch(() => {})
      isConnected = false
      shellId = null
      emit('disconnected')
    }
  }, 60000)
})

onUnmounted(async () => {
  clearInterval(idleTimer)
  resizeObserver?.disconnect()
  unlisten?.()
  if (shellId) {
    // Try both SSH and local close
    await invoke('ssh_close_shell', { sessionId: shellId }).catch(() => {})
    await invoke('local_close_shell', { sessionId: shellId }).catch(() => {})
  }
  splitTerm?.dispose(); term?.dispose()
})
</script>
