<template>
  <div class="h-full flex flex-col bg-gray-900">
    <!-- 顶部：录制控制 -->
    <div class="h-9 bg-gray-800 border-b border-gray-700 flex items-center px-3 gap-2">
      <span class="text-sm font-medium text-gray-300">🎯 宏录制</span>
      <div class="flex-1" />
      <span v-if="recording" class="flex items-center gap-1 text-xs text-red-400">
        <span class="w-2 h-2 rounded-full bg-red-500 animate-pulse" />
        录制中 ({{ currentCommands.length }})
      </span>
      <button @click="toggleRecording" class="text-xs px-2 py-0.5 rounded"
        :class="recording ? 'bg-red-600 hover:bg-red-500 text-white' : 'bg-blue-600 hover:bg-blue-500 text-white'">
        {{ recording ? '⏹ 停止' : '⏺ 开始录制' }}
      </button>
    </div>

    <!-- 回放状态 -->
    <div v-if="replaying" class="px-3 py-1.5 bg-yellow-900/40 border-b border-yellow-700/50 flex items-center gap-2">
      <span class="w-2 h-2 rounded-full bg-yellow-400 animate-pulse" />
      <span class="text-xs text-yellow-300">正在回放: {{ replayIndex }}/{{ replayTotal }}</span>
      <button @click="stopReplay" class="ml-auto text-xs px-2 py-0.5 rounded bg-red-600/50 text-red-300 hover:bg-red-600/70">停止</button>
    </div>

    <!-- 录制中：命令输入 -->
    <div v-if="recording" class="px-3 py-2 border-b border-gray-700 flex gap-2">
      <input
        v-model="commandInput"
        @keydown.enter="addCommand"
        class="flex-1 bg-gray-900 border border-gray-600 rounded px-2 py-1 text-sm font-mono focus:outline-none focus:border-red-500"
        placeholder="输入命令后回车添加到录制..."
      />
      <button @click="addCommand" :disabled="!commandInput.trim()" class="text-xs px-2 py-1 rounded bg-red-600/30 text-red-300 hover:bg-red-600/50 disabled:opacity-40">+ 添加</button>
    </div>

    <!-- 宏列表 -->
    <div class="flex-1 overflow-y-auto p-3">
      <div v-for="macro in macros" :key="macro.id" class="bg-gray-800 rounded-lg mb-2 p-3 group">
        <div class="flex items-center justify-between">
          <div class="flex-1 min-w-0">
            <div class="text-sm text-gray-200 font-medium">{{ macro.name }}</div>
            <div class="text-xs text-gray-500 mt-0.5">
              {{ macro.commands.length }} 条命令 · {{ formatDate(macro.createdAt) }}
            </div>
          </div>
          <div class="flex gap-1 ml-2">
            <button @click="replayMacro(macro)" :disabled="recording || replaying"
              class="text-xs px-2 py-0.5 rounded bg-green-600/30 text-green-300 hover:bg-green-600/50 disabled:opacity-40">▶ 回放</button>
            <button @click="deleteMacro(macro.id)" :disabled="recording || replaying"
              class="text-xs px-2 py-0.5 rounded bg-gray-700 text-gray-400 hover:text-red-400 disabled:opacity-40">🗑</button>
          </div>
        </div>
        <!-- 命令预览 -->
        <div class="mt-2 space-y-0.5">
          <div v-for="(cmd, i) in macro.commands.slice(0, 3)" :key="i"
            class="text-xs text-gray-500 font-mono truncate">
            <span class="text-gray-600">{{ i + 1 }}.</span> {{ cmd.command }}
          </div>
          <div v-if="macro.commands.length > 3" class="text-xs text-gray-600">
            ...还有 {{ macro.commands.length - 3 }} 条
          </div>
        </div>
      </div>

      <div v-if="macros.length === 0 && !recording" class="text-center text-gray-500 text-sm mt-10">
        暂无宏<br/><span class="text-xs">点击"开始录制"创建第一个宏</span>
      </div>
    </div>

    <!-- 底部：当前录制的命令预览 -->
    <div v-if="recording && currentCommands.length > 0" class="border-t border-gray-700 bg-gray-850 max-h-40 overflow-y-auto">
      <div class="px-3 py-1.5 text-xs text-gray-500 border-b border-gray-700/50 flex items-center gap-2">
        <span class="w-1.5 h-1.5 rounded-full bg-red-500 animate-pulse" />
        录制预览 ({{ currentCommands.length }})
      </div>
      <div class="px-3 py-1 space-y-0.5">
        <div v-for="(cmd, i) in currentCommands" :key="i" class="text-xs font-mono text-gray-400 truncate flex gap-2">
          <span class="text-gray-600 shrink-0">{{ i + 1 }}.</span>
          <span class="truncate">{{ cmd.command }}</span>
          <span class="text-gray-700 shrink-0">{{ cmd.time }}</span>
        </div>
      </div>
    </div>

    <!-- 保存宏对话框 -->
    <div v-if="showSave" class="fixed inset-0 bg-black/60 flex items-center justify-center z-50" @click.self="cancelSave">
      <div class="bg-gray-800 rounded-lg w-80 border border-gray-600 p-4 space-y-3">
        <h3 class="text-sm font-medium text-gray-200">保存宏</h3>
        <div class="text-xs text-gray-500">{{ currentCommands.length }} 条命令已录制</div>
        <input v-model="macroName" @keydown.enter="saveMacro" ref="nameInputRef"
          class="w-full bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm focus:outline-none focus:border-blue-500"
          placeholder="输入宏名称..." autofocus />
        <div class="flex justify-end gap-2">
          <button @click="cancelSave" class="px-3 py-1 text-sm text-gray-400">取消</button>
          <button @click="saveMacro" :disabled="!macroName.trim()" class="px-3 py-1 text-sm bg-blue-600 rounded text-white disabled:opacity-50">保存</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, nextTick, onMounted, onUnmounted } from 'vue'
import { invoke } from '../utils/tauri.js'
import { secureStore } from '../utils/secure-store-browser'

const STORAGE_KEY = 'xterminal_macros'

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
