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
import 'xterm/css/xterm.css'

const props = defineProps({
  tab: Object,
  active: Boolean,
})

const containerRef = ref(null)
let term = null
let fitAddon = null
let resizeObserver = null

function initTerminal() {
  if (term || !containerRef.value) return

  term = new Terminal({
    cursorBlink: true,
    fontSize: 14,
    fontFamily: "'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace",
    theme: {
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
    },
    allowTransparency: false,
    scrollback: 5000,
  })

  fitAddon = new FitAddon()
  const searchAddon = new SearchAddon()

  term.loadAddon(fitAddon)
  term.loadAddon(searchAddon)

  term.open(containerRef.value)
  fitAddon.fit()

  // Welcome message
  term.writeln('\x1b[1;36m╔══════════════════════════════════════╗\x1b[0m')
  term.writeln('\x1b[1;36m║       XTerminal Pro v0.1.0           ║\x1b[0m')
  term.writeln('\x1b[1;36m╚══════════════════════════════════════╝\x1b[0m')
  term.writeln('')
  term.writeln(`  连接: \x1b[1;33m${props.tab.name}\x1b[0m`)
  term.writeln(`  类型: ${props.tab.connection?.host === 'localhost' ? '本地终端' : 'SSH'}`)
  term.writeln('')
  term.write('$ ')

  // Handle input (local shell simulation)
  let inputBuffer = ''
  term.onData((data) => {
    if (data === '\r') {
      term.write('\r\n')
      if (inputBuffer.trim()) {
        handleCommand(inputBuffer.trim())
      }
      inputBuffer = ''
      term.write('$ ')
    } else if (data === '\x7f') {
      if (inputBuffer.length > 0) {
        inputBuffer = inputBuffer.slice(0, -1)
        term.write('\b \b')
      }
    } else if (data >= ' ') {
      inputBuffer += data
      term.write(data)
    }
  })

  // Resize handling
  resizeObserver = new ResizeObserver(() => {
    if (fitAddon && props.active) {
      fitAddon.fit()
    }
  })
  resizeObserver.observe(containerRef.value)
}

function handleCommand(cmd) {
  // Basic local command simulation
  const parts = cmd.split(' ')
  const bin = parts[0]

  switch (bin) {
    case 'help':
      term.writeln('可用命令: help, clear, echo, date, whoami, pwd, ls, exit')
      break
    case 'clear':
      term.clear()
      break
    case 'echo':
      term.writeln(parts.slice(1).join(' '))
      break
    case 'date':
      term.writeln(new Date().toString())
      break
    case 'whoami':
      term.writeln('user')
      break
    case 'pwd':
      term.writeln('/home/user')
      break
    case 'ls':
      term.writeln('Desktop  Documents  Downloads  Music  Pictures  Videos')
      break
    case 'exit':
      term.writeln('\x1b[33m提示: 本地终端不支持 exit，使用标签页关闭\x1b[0m')
      break
    default:
      term.writeln(`\x1b[31m${bin}: command not found\x1b[0m`)
      term.writeln('提示: 这是前端演示模式，实际连接需要 Tauri 后端')
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
  term?.dispose()
})
</script>
