<template>
  <div
    v-show="active"
    ref="containerRef"
    class="absolute inset-0"
  />
</template>

<script setup>
import { ref, onMounted, onUnmounted, watch } from 'vue'
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
let term = null
let fitAddon = null
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

async function initTerminal() {
  if (term || !containerRef.value) return

  term = new Terminal({
    cursorBlink: true,
    fontSize: 14,
    fontFamily: "'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace",
    theme,
    allowTransparency: false,
    scrollback: 5000,
  })

  fitAddon = new FitAddon()
  const searchAddon = new SearchAddon()

  term.loadAddon(fitAddon)
  term.loadAddon(searchAddon)

  term.open(containerRef.value)
  fitAddon.fit()

  const conn = props.tab.connection
  const isLocal = !conn || conn.host === 'localhost'

  if (isLocal) {
    term.writeln('\x1b[1;36m╔══════════════════════════════════════╗\x1b[0m')
    term.writeln('\x1b[1;36m║       XTerminal Pro v0.1.0           ║\x1b[0m')
    term.writeln('\x1b[1;36m╚══════════════════════════════════════╝\x1b[0m')
    term.writeln('')
    term.writeln('  模式: \x1b[1;33m本地演示\x1b[0m')
    term.writeln('  输入 \x1b[1;32mhelp\x1b[0m 查看可用命令')
    term.writeln('')
    term.write('\x1b[1;32m$ \x1b[0m')
    isConnected = true
  } else {
    // SSH 连接
    term.writeln(`\x1b[1;33m正在连接 ${conn.username}@${conn.host}:${conn.port || 22}...\x1b[0m`)

    try {
      if (conn.authType === 'key' || conn.keyPath) {
        sessionId = await invoke('ssh_connect_key', {
          host: conn.host,
          port: conn.port || 22,
          username: conn.username,
          keyPath: conn.keyPath || '',
          passphrase: conn.passphrase || '',
        })
      } else {
        sessionId = await invoke('ssh_connect', {
          host: conn.host,
          port: conn.port || 22,
          username: conn.username,
          password: conn.password || '',
        })
      }

      isConnected = true
      term.writeln(`\x1b[1;32m✓ 已连接到 ${conn.host}\x1b[0m`)
      term.writeln('')
      term.write('\x1b[1;32m$ \x1b[0m')
      emit('connected', sessionId)
    } catch (err) {
      term.writeln(`\x1b[1;31m✗ 连接失败: ${err}\x1b[0m`)
      term.writeln('')
      term.writeln('提示: 检查主机地址、端口、用户名和密码')
      term.write('\x1b[1;31m$ \x1b[0m')
    }
  }

  // Handle input
  term.onData((data) => handleInput(data))

  // Resize handling
  resizeObserver = new ResizeObserver(() => {
    if (fitAddon && props.active) {
      fitAddon.fit()
    }
  })
  resizeObserver.observe(containerRef.value)
}

async function handleInput(data) {
  if (data === '\r') {
    // Enter
    term.write('\r\n')
    const cmd = inputBuffer.trim()
    inputBuffer = ''

    if (cmd) {
      await executeCommand(cmd)
    }

    if (isConnected) {
      term.write('\x1b[1;32m$ \x1b[0m')
    }
  } else if (data === '\x7f') {
    // Backspace
    if (inputBuffer.length > 0) {
      inputBuffer = inputBuffer.slice(0, -1)
      term.write('\b \b')
    }
  } else if (data === '\x03') {
    // Ctrl+C
    term.write('^C\r\n')
    inputBuffer = ''
    if (isConnected) {
      term.write('\x1b[1;32m$ \x1b[0m')
    }
  } else if (data === '\x0c') {
    // Ctrl+L (clear)
    term.clear()
    inputBuffer = ''
    if (isConnected) {
      term.write('\x1b[1;32m$ \x1b[0m')
    }
  } else if (data >= ' ') {
    inputBuffer += data
    term.write(data)
  }
}

async function executeCommand(cmd) {
  if (cmd === 'exit' || cmd === 'quit') {
    if (sessionId) {
      await invoke('ssh_disconnect', { sessionId }).catch(() => {})
      sessionId = null
      isConnected = false
      emit('disconnected')
    }
    term.writeln('\x1b[33m会话已断开\x1b[0m')
    return
  }

  if (cmd === 'clear') {
    term.clear()
    return
  }

  try {
    const result = sessionId
      ? await invoke('ssh_execute', { sessionId, command: cmd })
      : await invoke('ssh_execute', { command: cmd })

    if (result && !result.includes('\x1b[2J')) {
      term.write(result.replace(/\n/g, '\r\n'))
    } else if (result && result.includes('\x1b[2J')) {
      term.clear()
    }
  } catch (err) {
    term.write(`\x1b[31m执行错误: ${err}\x1b[0m\r\n`)
  }
}

watch(() => props.active, (active) => {
  if (active && term) {
    setTimeout(() => fitAddon?.fit(), 50)
  }
})

onMounted(() => {
  initTerminal()
})

onUnmounted(() => {
  resizeObserver?.disconnect()
  if (sessionId) {
    invoke('ssh_disconnect', { sessionId }).catch(() => {})
  }
  term?.dispose()
})
</script>
