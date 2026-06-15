<template>
  <div class="h-full flex flex-col" style="background: var(--bg-base)">
    <!-- 顶部：录制控制 -->
    <div class="h-9 border-b flex items-center px-3 gap-2" style="background: var(--bg-surface); border-color: var(--border)">
      <span class="text-sm font-medium" style="color: var(--fg-secondary)">🎯 宏录制</span>
      <div class="flex-1" />
      <span v-if="recording" class="flex items-center gap-1 text-xs" style="color: var(--danger)">
        <span class="w-2 h-2 rounded-full animate-pulse" style="background: var(--danger)" />
        录制中 ({{ currentCommands.length }})
      </span>
      <button @click="toggleRecording" class="text-xs px-2 py-0.5 rounded"
        :style="recording ? { background: 'var(--danger)', color: 'white' } : { background: 'var(--accent)', color: 'white' }">
        {{ recording ? '⏹ 停止' : '⏺ 开始录制' }}
      </button>
    </div>

    <!-- 回放状态 -->
    <div v-if="replaying" class="px-3 py-1.5 border-b flex items-center gap-2" style="background: color-mix(in srgb, var(--warning) 40%, transparent); border-color: color-mix(in srgb, var(--warning) 50%, transparent)">
      <span class="w-2 h-2 rounded-full animate-pulse" style="background: var(--warning)" />
      <span class="text-xs" style="color: var(--warning)">正在回放: {{ replayIndex }}/{{ replayTotal }}</span>
      <button @click="stopReplay" class="ml-auto text-xs px-2 py-0.5 rounded" style="background: color-mix(in srgb, var(--danger) 50%, transparent); color: var(--danger)">停止</button>
    </div>

    <!-- 录制中：命令输入 -->
    <div v-if="recording" class="px-3 py-2 border-b flex gap-2" style="border-color: var(--border)">
      <input
        v-model="commandInput"
        @keydown.enter="addCommand"
        class="flex-1 rounded px-2 py-1 text-sm font-mono focus:outline-none" style="background: var(--bg-base); border-color: var(--border-subtle)"
        placeholder="输入命令后回车添加到录制..."
      />
      <button @click="addCommand" :disabled="!commandInput.trim()" class="text-xs px-2 py-1 rounded disabled:opacity-40" style="background: color-mix(in srgb, var(--danger) 30%, transparent); color: var(--danger)">+ 添加</button>
    </div>

    <!-- 宏列表 -->
    <div class="flex-1 overflow-y-auto p-3">
      <div v-for="macro in macros" :key="macro.id" class="rounded-lg mb-2 p-3 group" style="background: var(--bg-surface)">
        <div class="flex items-center justify-between">
          <div class="flex-1 min-w-0">
            <div class="text-sm font-medium" style="color: var(--fg-primary)">{{ macro.name }}</div>
            <div class="text-xs mt-0.5" style="color: var(--fg-muted)">
              {{ macro.commands.length }} 条命令 · {{ formatDate(macro.createdAt) }}
            </div>
          </div>
          <div class="flex gap-1 ml-2">
            <button @click="replayMacro(macro)" :disabled="recording || replaying"
              class="text-xs px-2 py-0.5 rounded disabled:opacity-40" style="background: color-mix(in srgb, var(--success) 30%, transparent); color: var(--success)">▶ 回放</button>
            <button @click="deleteMacro(macro.id)" :disabled="recording || replaying"
              class="text-xs px-2 py-0.5 rounded disabled:opacity-40" style="background: var(--bg-elevated); color: var(--fg-muted)">🗑</button>
          </div>
        </div>
        <!-- 命令预览 -->
        <div class="mt-2 space-y-0.5">
          <div v-for="(cmd, i) in macro.commands.slice(0, 3)" :key="i"
            class="text-xs font-mono truncate" style="color: var(--fg-muted)">
            <span style="color: var(--fg-muted)">{{ i + 1 }}.</span> {{ cmd.command }}
          </div>
          <div v-if="macro.commands.length > 3" class="text-xs" style="color: var(--fg-muted)">
            ...还有 {{ macro.commands.length - 3 }} 条
          </div>
        </div>
      </div>

      <div v-if="macros.length === 0 && !recording" class="text-center text-sm mt-10" style="color: var(--fg-muted)">
        暂无宏<br/><span class="text-xs">点击"开始录制"创建第一个宏</span>
      </div>
    </div>

    <!-- 底部：当前录制的命令预览 -->
    <div v-if="recording && currentCommands.length > 0" class="border-t max-h-40 overflow-y-auto" style="background: var(--bg-surface); border-color: var(--border)">
      <div class="px-3 py-1.5 text-xs border-b flex items-center gap-2" style="color: var(--fg-muted); border-color: color-mix(in srgb, var(--border) 50%, transparent)">
        <span class="w-1.5 h-1.5 rounded-full animate-pulse" style="background: var(--danger)" />
        录制预览 ({{ currentCommands.length }})
      </div>
      <div class="px-3 py-1 space-y-0.5">
        <div v-for="(cmd, i) in currentCommands" :key="i" class="text-xs font-mono truncate flex gap-2" style="color: var(--fg-muted)">
          <span class="shrink-0" style="color: var(--fg-muted)">{{ i + 1 }}.</span>
          <span class="truncate">{{ cmd.command }}</span>
          <span class="shrink-0" style="color: var(--bg-elevated)">{{ cmd.time }}</span>
        </div>
      </div>
    </div>

    <!-- 保存宏对话框 -->
    <div v-if="showSave" class="fixed inset-0 flex items-center justify-center z-50" style="background: color-mix(in srgb, #000 60%, transparent)" @click.self="cancelSave">
      <div class="rounded-lg w-80 p-4 space-y-3" style="background: var(--bg-surface); border-color: var(--border-subtle)">
        <h3 class="text-sm font-medium" style="color: var(--fg-primary)">保存宏</h3>
        <div class="text-xs" style="color: var(--fg-muted)">{{ currentCommands.length }} 条命令已录制</div>
        <input v-model="macroName" @keydown.enter="saveMacro" ref="nameInputRef"
          class="w-full rounded px-3 py-1.5 text-sm focus:outline-none" style="background: var(--bg-base); border-color: var(--border-subtle)"
          placeholder="输入宏名称..." autofocus />
        <div class="flex justify-end gap-2">
          <button @click="cancelSave" class="px-3 py-1 text-sm" style="color: var(--fg-muted)">取消</button>
          <button @click="saveMacro" :disabled="!macroName.trim()" class="px-3 py-1 text-sm rounded disabled:opacity-50" style="background: var(--accent); color: white">保存</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, nextTick, onMounted } from 'vue'
import { invoke } from '../utils/tauri.js'
import { secureStore } from '../utils/secure-store-browser'

import { STORAGE_KEYS } from '@/utils/storage-keys'
const STORAGE_KEY = STORAGE_KEYS.MACROS

const props = defineProps({
  sessionId: String,
})

const emit = defineEmits(['status'])

// State
const recording = ref(false)
const currentCommands = ref([])
const commandInput = ref('')
const macros = ref([])
const showSave = ref(false)
const macroName = ref('')
const nameInputRef = ref(null)
const replaying = ref(false)
const replayIndex = ref(0)
const replayTotal = ref(0)
let replayAbort = false

// Load macros from secure storage
async function loadMacros() {
  try {
    const data = await secureStore.get(STORAGE_KEY)
    macros.value = data || []
  } catch {
    macros.value = []
  }
}

// Save macros to secure storage
async function saveMacros() {
  try {
    await secureStore.set(STORAGE_KEY, macros.value)
  } catch (error) {
    console.error('Failed to save macros:', error)
  }
}

// Toggle recording
function toggleRecording() {
  if (recording.value) {
    // Stop recording
    recording.value = false
    if (currentCommands.value.length > 0) {
      showSave.value = true
      macroName.value = `宏 ${new Date().toLocaleString('zh-CN', { month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' })}`
      nextTick(() => nameInputRef.value?.focus())
    }
  } else {
    // Start recording
    currentCommands.value = []
    commandInput.value = ''
    recording.value = true
  }
}

// Add command during recording
function addCommand() {
  const cmd = commandInput.value.trim()
  if (!cmd || !recording.value) return
  currentCommands.value.push({
    command: cmd,
    time: new Date().toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit', second: '2-digit' }),
  })
  commandInput.value = ''
}

// Save macro
function saveMacro() {
  const name = macroName.value.trim()
  if (!name || currentCommands.value.length === 0) return
  macros.value.push({
    id: `macro_${Date.now()}`,
    name,
    commands: [...currentCommands.value],
    createdAt: new Date().toISOString(),
  })
  saveMacros().then(() => { 
    showSave.value = false
  }).catch(err => console.error('Failed to save macro:', err))
  currentCommands.value = []
  macroName.value = ''
}

// Cancel save
function cancelSave() {
  showSave.value = false
  currentCommands.value = []
  macroName.value = ''
}

// Delete macro
async function deleteMacro(id) {
  macros.value = macros.value.filter(m => m.id !== id)
  await saveMacros()
}

// Replay macro
async function replayMacro(macro) {
  if (!props.sessionId) {
    emit('status', '请先连接服务器')
    return
  }
  if (replaying.value || recording.value) return

  replaying.value = true
  replayAbort = false
  replayTotal.value = macro.commands.length
  replayIndex.value = 0

  for (let i = 0; i < macro.commands.length; i++) {
    if (replayAbort) break
    replayIndex.value = i + 1
    try {
      await invoke('ssh_shell_input', { sessionId: props.sessionId, data: macro.commands[i].command + '\r' })
    } catch { /* send failed */ }
    if (i < macro.commands.length - 1 && !replayAbort) {
      await delay(500)
    }
  }

  replaying.value = false
  replayIndex.value = 0
  replayTotal.value = 0
}

// Stop replay
function stopReplay() {
  replayAbort = true
  replaying.value = false
}

// Helpers
function delay(ms) {
  return new Promise(r => setTimeout(r, ms))
}

function formatDate(iso) {
  try {
    return new Date(iso).toLocaleString('zh-CN', { month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' })
  } catch {
    return '-'
  }
}

onMounted(() => loadMacros())
</script>
