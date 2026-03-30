<template>
  <div class="h-full flex flex-col bg-gray-900">
    <div class="h-9 bg-gray-800 border-b border-gray-700 flex items-center px-3 gap-2">
      <span class="text-sm font-medium text-gray-300">⏺ 会话录制</span>
      <div class="flex-1" />
      <span v-if="recording" class="flex items-center gap-1 text-xs text-red-400">
        <span class="w-2 h-2 rounded-full bg-red-500 animate-pulse" />
        {{ formatDuration(recDuration) }}
      </span>
      <button @click="toggleRecording" :disabled="!sessionId && !recording" class="text-xs px-2 py-0.5 rounded"
        :class="recording ? 'bg-red-600 hover:bg-red-500 text-white' : 'bg-blue-600 hover:bg-blue-500 text-white disabled:opacity-50'">
        {{ recording ? '⏹ 停止' : '⏺ 录制' }}
      </button>
    </div>

    <div class="flex-1 overflow-y-auto p-3">
      <div v-for="rec in recordings" :key="rec.id" class="bg-gray-800 rounded-lg mb-2 p-3 hover:bg-gray-750">
        <div class="flex items-center justify-between">
          <div>
            <div class="text-sm text-gray-200">{{ rec.name }}</div>
            <div class="text-xs text-gray-500 mt-0.5">
              {{ rec.connection_name }} · {{ rec.started_at }} · {{ formatDuration(rec.duration_secs) }}
            </div>
          </div>
          <div class="flex gap-1">
            <button @click="playRecording(rec)" class="text-xs px-2 py-0.5 rounded bg-green-600/30 text-green-300 hover:bg-green-600/50">▶</button>
            <button @click="exportRecording(rec)" class="text-xs px-2 py-0.5 rounded bg-blue-600/30 text-blue-300 hover:bg-blue-600/50">📤</button>
            <button @click="delRecording(rec.id)" class="text-xs px-2 py-0.5 rounded bg-gray-700 text-gray-400 hover:text-white">🗑</button>
          </div>
        </div>
        <div v-if="rec.tags?.length" class="flex gap-1 mt-2">
          <span v-for="tag in rec.tags" :key="tag" class="text-[10px] px-1.5 py-0.5 bg-gray-700 rounded text-gray-400">{{ tag }}</span>
        </div>
      </div>
      <div v-if="recordings.length === 0" class="text-center text-gray-500 text-sm mt-10">
        暂无录制<br/><span class="text-xs">连接服务器后点击 ⏺ 开始录制</span>
      </div>
    </div>

    <!-- Playback Modal -->
    <div v-if="playing" class="fixed inset-0 bg-black/80 flex items-center justify-center z-50" @click.self="playing = null">
      <div class="bg-gray-800 rounded-lg w-[720px] border border-gray-600 flex flex-col max-h-[80vh]">
        <div class="p-3 border-b border-gray-700 flex items-center justify-between">
          <span class="text-sm text-gray-200">▶ {{ playing.name }}</span>
          <button @click="playing = null" class="text-gray-400 hover:text-white">✕</button>
        </div>
        <div ref="playbackRef" class="h-80 bg-black overflow-auto font-mono text-sm p-3" style="color: #d4d4d4;">
          <div v-for="(line, i) in playbackLines" :key="i" class="whitespace-pre-wrap break-all" v-html="ansiToHtml(line)" />
        </div>
        <div class="p-3 border-t border-gray-700 flex items-center gap-3">
          <button @click="togglePlayback" class="text-sm px-3 py-1 rounded text-white" :class="playbackActive ? 'bg-red-600' : 'bg-blue-600'">
            {{ playbackActive ? '⏸ 暂停' : '▶ 播放' }}
          </button>
          <input type="range" class="flex-1" min="0" :max="playbackData.length" v-model.number="playbackIdx" @input="renderPlayback" />
          <span class="text-xs text-gray-500">{{ playbackIdx }}/{{ playbackData.length }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, nextTick, watch } from 'vue'
import { invoke, listen } from '../utils/tauri.js'

const props = defineProps({ sessionId: String, connectionName: String })

const recording = ref(false)
const recDuration = ref(0)
const recData = []
const recordings = ref([])
const playing = ref(null)
const playbackRef = ref(null)
const playbackLines = ref([])
const playbackData = ref([])
const playbackIdx = ref(0)
const playbackActive = ref(false)
let recTimer = null
let unlistenOutput = null

function formatDuration(s) {
  if (!s && s !== 0) return '-'
  const m = Math.floor(s / 60)
  return `${m}:${(s % 60).toString().padStart(2, '0')}`
}

async function loadRecordings() {
  try { recordings.value = await invoke('list_recordings') || [] }
  catch { recordings.value = [] }
}

async function toggleRecording() {
  if (recording.value) {
    // Stop recording
    clearInterval(recTimer)
    unlistenOutput?.()
    unlistenOutput = null
    recording.value = false

    // Save as asciinema v2
    const id = `rec_${Date.now()}`
    const filename = `${id}.cast`
    const header = JSON.stringify({
      version: 2, width: 80, height: 24,
      timestamp: Math.floor(Date.now() / 1000),
      duration: recDuration.value,
      title: `录制 ${new Date().toLocaleString('zh-CN')}`,
      env: { SHELL: '/bin/bash', TERM: 'xterm-256color' }
    })
    const body = recData.map(e => JSON.stringify(e)).join('\n')
    const content = header + '\n' + body + '\n'

    try {
      await invoke('save_recording_file', { filename, content })
      await invoke('save_recording_meta', {
        meta: {
          id, name: `录制 ${new Date().toLocaleString('zh-CN')}`,
          connection_name: props.connectionName || '未知',
          started_at: new Date().toISOString(),
          duration_secs: recDuration.value,
          file_path: filename,
          tags: [],
        }
      })
    } catch (e) { console.error('Save recording failed:', e) }

    recData.length = 0
    recDuration.value = 0
    await loadRecordings()
  } else {
    // Start recording
    if (!props.sessionId) return
    recData.length = 0
    recDuration.value = 0
    recording.value = true

    const startTime = Date.now()

    // Listen for terminal output
    unlistenOutput = await listen(`ssh-output:${props.sessionId}`, (event) => {
      if (recording.value) {
        const elapsed = ((Date.now() - startTime) / 1000).toFixed(3)
        recData.push([parseFloat(elapsed), 'o', event.payload])
      }
    })

    recTimer = setInterval(() => { recDuration.value++ }, 1000)
  }
}

async function playRecording(rec) {
  try {
    const content = await invoke('read_recording_file', { filename: rec.file_path })
    const lines = content.split('\n').filter(Boolean)
    // Skip header line
    const dataLines = lines.slice(1)
    playbackData.value = dataLines.map(l => { try { return JSON.parse(l) } catch { return null } }).filter(Boolean)
    playbackIdx.value = 0
    playbackLines.value = []
    playing.value = rec
  } catch (e) { alert('读取录制失败: ' + e) }
}

let playbackTimer = null
function togglePlayback() {
  if (playbackActive.value) {
    clearInterval(playbackTimer)
    playbackActive.value = false
  } else {
    playbackActive.value = true
    playbackTimer = setInterval(() => {
      if (playbackIdx.value >= playbackData.value.length) {
        clearInterval(playbackTimer)
        playbackActive.value = false
        return
      }
      const entry = playbackData.value[playbackIdx.value]
      if (entry && entry[1] === 'o') {
        playbackLines.value.push(entry[2])
        nextTick(() => { if (playbackRef.value) playbackRef.value.scrollTop = playbackRef.value.scrollHeight })
      }
      playbackIdx.value++
    }, 50)
  }
}

function renderPlayback() {
  playbackLines.value = []
  for (let i = 0; i < playbackIdx.value && i < playbackData.value.length; i++) {
    const entry = playbackData.value[i]
    if (entry && entry[1] === 'o') playbackLines.value.push(entry[2])
  }
}

async function exportRecording(rec) {
  try {
    const content = await invoke('read_recording_file', { filename: rec.file_path })
    const blob = new Blob([content], { type: 'application/x-asciicast' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url; a.download = `${rec.name}.cast`; a.click()
    URL.revokeObjectURL(url)
  } catch (e) { alert('导出失败: ' + e) }
}

async function delRecording(id) {
  if (!confirm('确认删除此录制？')) return
  try { await invoke('delete_recording', { id }); await loadRecordings() }
  catch (e) { alert('删除失败: ' + e) }
}

function ansiToHtml(text) {
  if (!text) return ''
  return text
    .replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
    .replace(/\x1b\[1;32m/g, '<span style="color:#0dbc79">')
    .replace(/\x1b\[1;31m/g, '<span style="color:#cd3131">')
    .replace(/\x1b\[1;33m/g, '<span style="color:#e5e510">')
    .replace(/\x1b\[1;36m/g, '<span style="color:#11a8cd">')
    .replace(/\x1b\[0m/g, '</span>')
    .replace(/\r\n/g, '\n').replace(/\r/g, '\n')
}

onMounted(loadRecordings)
onUnmounted(() => { clearInterval(recTimer); unlistenOutput?.(); clearInterval(playbackTimer) })
</script>
