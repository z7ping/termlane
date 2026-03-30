<template>
  <div class="flex flex-col h-full">
    <!-- Search Bar -->
    <div v-if="showSearch" class="h-8 bg-gray-800 border-b border-gray-700 flex items-center px-2 gap-2">
      <input
        ref="searchInput"
        v-model="searchTerm"
        @keydown.enter="searchNext"
        @keydown.shift.enter="searchPrev"
        class="flex-1 bg-gray-900 text-sm text-gray-200 px-2 py-1 rounded border border-gray-600 focus:outline-none focus:border-blue-500"
        placeholder="搜索... (Enter下一个, Shift+Enter上一个)"
      />
      <span class="text-xs text-gray-500">{{ searchIndex }}/{{ searchTotal }}</span>
      <button @click="searchPrev" class="text-gray-400 hover:text-white px-1">▲</button>
      <button @click="searchNext" class="text-gray-400 hover:text-white px-1">▼</button>
      <button @click="closeSearch" class="text-gray-400 hover:text-white px-1">✕</button>
    </div>

    <!-- Terminal Container -->
    <div class="flex-1 flex overflow-hidden">
      <!-- Main Terminal -->
      <div
        ref="containerRef"
        class="flex-1"
        :style="splitMode ? { width: '50%' } : {}"
      />

      <!-- Split Pane -->
      <div
        v-if="splitMode"
        ref="splitContainerRef"
        class="flex-1 border-l border-gray-700"
      />
    </div>

    <!-- Split Toggle -->
    <div v-if="active" class="absolute bottom-8 right-2 flex gap-1 z-10">
      <button
        @click="toggleSearch"
        class="w-7 h-7 bg-gray-700 hover:bg-gray-600 rounded text-xs text-gray-300 flex items-center justify-center"
        title="搜索 (Ctrl+Shift+F)"
      >🔍</button>
      <button
        @click="toggleSplit"
        class="w-7 h-7 rounded text-xs flex items-center justify-center"
        :class="splitMode ? 'bg-blue-600 text-white' : 'bg-gray-700 hover:bg-gray-600 text-gray-300'"
        title="分屏"
      >⊞</button>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { Terminal } from 'xterm'
import { FitAddon } from 'xterm-addon-fit'
import { SearchAddon } from 'xterm-addon-search'
import { invoke } from '../utils/tauri.js'
import 'xterm/css/xterm.css'

const props = defineProps({
  tab: Object,
  active: Boolean,
})

const emit = defineEmits(['connected', 'disconnected'])

const containerRef = ref(null)
const splitContainerRef = ref(null)
const searchInput = ref(null)
const showSearch = ref(false)
const searchTerm = ref('')
const searchIndex = ref(0)
const searchTotal = ref(0)
const splitMode = ref(false)

let term = null
let splitTerm = null
let fitAddon = null
let splitFitAddon = null
let searchAddon = null
let splitSearchAddon = null
let resizeObserver = null
let sessionId = null
let isConnected = false
let inputBuffer = ''

const theme = {
  background: '#1e1e1e',
  foreground: '#d4d4d4',
  cursor: '#aeafad',
  selection: '#264f78',
  black: '#000000',
  red: '#cd3131',
  green: '#0dbc79',
  yellow: '#e5e510',
  blue: '#2472c8',
  magenta: '#bc3fbc',
  cyan: '#11a8cd',
  white: '#e5e5e5',
  brightBlack: '#666666',
  brightRed: '#f14c4c',
  brightGreen: '#23d18b',
  brightYellow: '#f5f543',
  brightBlue: '#3b8eea',
  brightMagenta: '#d670d6',
  brightCyan: '#29b8db',
  brightWhite: '#e5e5e5',
}

function createTerminal(container, isPrimary = true) {
  const t = new Terminal({
    cursorBlink: true,
    fontSize: 14,
    fontFamily: "'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace",
    theme,
    allowTransparency: false,
    scrollback: 10000,
  })

  const fit = new FitAddon()
  const search = new SearchAddon()
  t.loadAddon(fit)
  t.loadAddon(search)

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
    showWelcome(term, '本地演示')
    term.write('\x1b[1;32m$ \x1b[0m')
    isConnected = true
  } else {
    await connectSSH(term, conn)
  }

  term.onData((data) => handleInput(term, data))

  // Keyboard shortcuts
  term.attachCustomKeyEventHandler((e) => {
    if (e.ctrlKey && e.shiftKey && e.key === 'F') {
      toggleSearch()
      return false
    }
    return true
  })

  resizeObserver = new ResizeObserver(() => {
    if (fitAddon && props.active) fitAddon.fit()
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
  t.writeln('  输入 \x1b[1;32mhelp\x1b[0m 查看可用命令')
  t.writeln('')
}

async function connectSSH(t, conn) {
  t.writeln(`\x1b[1;33m正在连接 ${conn.username}@${conn.host}:${conn.port || 22}...\x1b[0m`)
  try {
    // 检查是否有跳板配置
    if (conn.useJumpHost && conn.jumpHost) {
      t.writeln(`\x1b[1;33m通过跳板机 ${conn.jumpHost} 中转...\x1b[0m`)
      sessionId = await invoke('ssh_connect_jump', {
        targetHost: conn.host,
        targetPort: conn.port || 22,
        targetUser: conn.username,
        targetPass: conn.password || '',
        jumpHost: conn.jumpHost,
        jumpPort: conn.jumpPort || 22,
        jumpUser: conn.jumpUsername || conn.username,
        jumpPass: conn.jumpPassword || conn.password || '',
      })
    } else if (conn.authType === 'key' || conn.keyPath) {
      sessionId = await invoke('ssh_connect_key', {
        host: conn.host, port: conn.port || 22, username: conn.username,
        keyPath: conn.keyPath || '', passphrase: conn.passphrase || '',
      })
    } else {
      sessionId = await invoke('ssh_connect', {
        host: conn.host, port: conn.port || 22, username: conn.username,
        password: conn.password || '',
      })
    }
    isConnected = true
    t.writeln(`\x1b[1;32m✓ 已连接到 ${conn.host}\x1b[0m`)
    t.writeln('')
    t.write('\x1b[1;32m$ \x1b[0m')
    emit('connected', sessionId)
  } catch (err) {
    t.writeln(`\x1b[1;31m✗ 连接失败: ${err}\x1b[0m`)
    t.write('\x1b[1;31m$ \x1b[0m')
  }
}

function handleInput(t, data) {
  if (data === '\r') {
    t.write('\r\n')
    const cmd = inputBuffer.trim()
    inputBuffer = ''
    if (cmd) executeCommand(t, cmd)
    if (isConnected) t.write('\x1b[1;32m$ \x1b[0m')
  } else if (data === '\x7f') {
    if (inputBuffer.length > 0) {
      inputBuffer = inputBuffer.slice(0, -1)
      t.write('\b \b')
    }
  } else if (data === '\x03') {
    t.write('^C\r\n')
    inputBuffer = ''
    if (isConnected) t.write('\x1b[1;32m$ \x1b[0m')
  } else if (data === '\x0c') {
    t.clear()
    inputBuffer = ''
    if (isConnected) t.write('\x1b[1;32m$ \x1b[0m')
  } else if (data >= ' ') {
    inputBuffer += data
    t.write(data)
  }
}

async function executeCommand(t, cmd) {
  if (cmd === 'exit' || cmd === 'quit') {
    if (sessionId) {
      await invoke('ssh_disconnect', { sessionId }).catch(() => {})
      sessionId = null
      isConnected = false
      emit('disconnected')
    }
    t.writeln('\x1b[33m会话已断开\x1b[0m')
    return
  }
  if (cmd === 'clear') { t.clear(); return }

  try {
    const result = sessionId
      ? await invoke('ssh_execute', { sessionId, command: cmd })
      : await invoke('ssh_execute', { command: cmd })
    if (result) t.write(result.replace(/\n/g, '\r\n'))
  } catch (err) {
    t.write(`\x1b[31m执行错误: ${err}\x1b[0m\r\n`)
  }
}

// Search
function toggleSearch() {
  showSearch.value = !showSearch.value
  if (showSearch.value) {
    nextTick(() => searchInput.value?.focus())
  }
}

function closeSearch() {
  showSearch.value = false
  searchTerm.value = ''
  searchAddon?.clearDecorations()
}

function searchNext() {
  if (!searchTerm.value || !searchAddon) return
  searchAddon.findNext(searchTerm.value)
  updateSearchCount()
}

function searchPrev() {
  if (!searchTerm.value || !searchAddon) return
  searchAddon.findPrevious(searchTerm.value)
  updateSearchCount()
}

function updateSearchCount() {
  // xterm-addon-search doesn't expose count directly
  searchIndex.value = '?'
  searchTotal.value = '?'
}

// Split pane
function toggleSplit() {
  splitMode.value = !splitMode.value
  if (splitMode.value) {
    nextTick(() => {
      if (splitContainerRef.value && !splitTerm) {
        const { term: st, fit: sf, search: ss } = createTerminal(splitContainerRef.value, false)
        splitTerm = st
        splitFitAddon = sf
        splitSearchAddon = ss
        showWelcome(splitTerm, '分屏')
        splitTerm.write('\x1b[1;32m$ \x1b[0m')

        // Mirror input to split pane
        splitTerm.onData((data) => handleInput(splitTerm, data))
      }
      setTimeout(() => {
        fitAddon?.fit()
        splitFitAddon?.fit()
      }, 50)
    })
  } else {
    splitTerm?.dispose()
    splitTerm = null
    splitFitAddon = null
    nextTick(() => fitAddon?.fit())
  }
}

watch(() => props.active, (active) => {
  if (active && term) {
    setTimeout(() => {
      fitAddon?.fit()
      if (splitMode.value) splitFitAddon?.fit()
    }, 50)
  }
})

onMounted(() => { initTerminal() })

onUnmounted(() => {
  resizeObserver?.disconnect()
  if (sessionId) invoke('ssh_disconnect', { sessionId }).catch(() => {})
  splitTerm?.dispose()
  term?.dispose()
})
</script>
